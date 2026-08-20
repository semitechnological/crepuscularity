use std::path::Path;

use crate::ast::*;
use crate::parser::{parse_template_with_path, parse_vue_sfc};

fn parse(src: &str) -> Vec<Node> {
    parse_template_with_path(src, Some(Path::new("Comp.vue"))).expect("vue template should parse")
}

fn parse_body(body: &str) -> Vec<Node> {
    parse(&format!("<template>\n{body}\n</template>\n"))
}

fn element(node: &Node) -> &Element {
    match node {
        Node::Element(el) => el,
        other => panic!("expected element, got {other:?}"),
    }
}

fn text_of(node: &Node) -> String {
    match node {
        Node::Text(parts) => {
            let mut out = String::new();
            for p in parts {
                match p {
                    TextPart::Literal(s) => out.push_str(s),
                    TextPart::Expr(e) => {
                        out.push('{');
                        out.push_str(e);
                        out.push('}');
                    }
                }
            }
            out
        }
        other => panic!("expected text, got {other:?}"),
    }
}

#[test]
fn sfc_splits_template_script_and_style() {
    let sfc = parse_vue_sfc(
        r#"<template>
  <div class="p-4">hi</div>
</template>

<script setup lang="ts">
const count = 1
</script>

<style scoped>
.p-4 { padding: 1rem; }
</style>
"#,
    )
    .unwrap();

    assert!(sfc
        .template
        .as_deref()
        .unwrap()
        .contains("<div class=\"p-4\">"));
    assert_eq!(sfc.scripts.len(), 1);
    assert!(sfc.scripts[0].setup);
    assert_eq!(sfc.scripts[0].lang.as_deref(), Some("ts"));
    assert!(sfc.scripts[0].content.contains("const count = 1"));
    assert_eq!(sfc.styles.len(), 1);
    assert!(sfc.styles[0].scoped);
    assert!(sfc.styles[0].content.contains("padding: 1rem"));
}

#[test]
fn script_content_is_never_parsed_as_markup() {
    let nodes = parse(
        r#"<template><span>ok</span></template>
<script>
const html = "<div class='nope'></div>"
</script>"#,
    );
    assert_eq!(nodes.len(), 1);
    assert_eq!(element(&nodes[0]).tag, "span");
}

#[test]
fn nested_template_element_does_not_end_the_sfc_block() {
    let nodes = parse(
        r#"<template>
  <ul>
    <template v-for="item in items">
      <li>{{ item }}</li>
    </template>
  </ul>
</template>"#,
    );
    let ul = element(&nodes[0]);
    assert_eq!(ul.tag, "ul");
    let Node::For(block) = &ul.children[0] else {
        panic!("expected for block, got {:?}", ul.children[0]);
    };
    assert_eq!(block.pattern, "item");
    assert_eq!(block.iterator, "items");
    assert_eq!(element(&block.body[0]).tag, "li");
}

#[test]
fn mustache_interpolation_becomes_text_parts() {
    let nodes = parse_body(r#"<p>Hello {{ name }}!</p>"#);
    let p = element(&nodes[0]);
    let Node::Text(parts) = &p.children[0] else {
        panic!("expected text");
    };
    assert!(matches!(&parts[0], TextPart::Literal(s) if s == "Hello "));
    assert!(matches!(&parts[1], TextPart::Expr(e) if e == "name"));
    assert!(matches!(&parts[2], TextPart::Literal(s) if s == "!"));
}

#[test]
fn v_if_else_if_else_chain_lowers_to_nested_if_blocks() {
    let nodes = parse_body(
        r#"<div>
  <p v-if="a">A</p>
  <p v-else-if="b">B</p>
  <p v-else>C</p>
</div>"#,
    );
    let root = element(&nodes[0]);
    assert_eq!(root.children.len(), 1);
    let Node::If(outer) = &root.children[0] else {
        panic!("expected if");
    };
    assert_eq!(outer.condition, "a");
    assert_eq!(text_of(&element(&outer.then_children[0]).children[0]), "A");
    let Some([Node::If(inner)]) = outer.else_children.as_deref() else {
        panic!("expected else-if chain");
    };
    assert_eq!(inner.condition, "b");
    let Some(else_nodes) = inner.else_children.as_deref() else {
        panic!("expected else");
    };
    assert_eq!(text_of(&element(&else_nodes[0]).children[0]), "C");
}

#[test]
fn v_for_supports_index_form() {
    let nodes = parse_body(r#"<li v-for="(item, i) in items">{{ item }}</li>"#);
    let Node::For(block) = &nodes[0] else {
        panic!("expected for");
    };
    // ForBlock binds a single item name; the index variable is not bound.
    assert_eq!(block.pattern, "item");
    assert_eq!(block.iterator, "items");
}

#[test]
fn v_show_becomes_a_conditional_hidden_class() {
    let nodes = parse_body(r#"<div v-show="visible"></div>"#);
    let el = element(&nodes[0]);
    assert_eq!(el.conditional_classes.len(), 1);
    assert_eq!(el.conditional_classes[0].class, "hidden");
    assert_eq!(el.conditional_classes[0].condition, "!(visible)");
}

#[test]
fn v_html_and_v_text_replace_children() {
    let nodes = parse_body(r#"<div v-html="body"></div><span v-text="label"></span>"#);
    let html = element(&nodes[0]);
    assert!(matches!(&html.children[0], Node::RawHtml(e) if e == "body"));
    let text = element(&nodes[1]);
    assert_eq!(text_of(&text.children[0]), "{label}");
}

#[test]
fn bindings_shorthand_and_longhand_are_equivalent() {
    let nodes = parse_body(r#"<img :src="url" v-bind:alt="caption" width="64" />"#);
    let el = element(&nodes[0]);
    assert_eq!(el.bindings[0].prop, "src");
    assert_eq!(el.bindings[0].value, "url");
    assert_eq!(el.bindings[1].prop, "alt");
    assert_eq!(el.bindings[1].value, "caption");
    assert_eq!(el.bindings[2].prop, "width");
    assert_eq!(el.bindings[2].value, "\"64\"");
}

#[test]
fn dynamic_class_object_and_array_syntax() {
    let nodes = parse_body(
        r#"<div class="base" :class="{ active: isOn, 'text-red-500 font-bold': danger }"></div>
<div :class="['a', dynamic]"></div>
<div :class="cls"></div>"#,
    );
    let obj = element(&nodes[0]);
    assert_eq!(obj.classes, vec!["base".to_string()]);
    assert_eq!(obj.conditional_classes.len(), 3);
    assert_eq!(obj.conditional_classes[0].class, "active");
    assert_eq!(obj.conditional_classes[0].condition, "isOn");
    assert_eq!(obj.conditional_classes[1].class, "text-red-500");
    assert_eq!(obj.conditional_classes[1].condition, "danger");
    assert_eq!(obj.conditional_classes[2].class, "font-bold");

    let arr = element(&nodes[1]);
    assert_eq!(arr.classes, vec!["a".to_string(), "{dynamic}".to_string()]);

    let plain = element(&nodes[2]);
    assert_eq!(plain.classes, vec!["{cls}".to_string()]);
}

#[test]
fn events_shorthand_longhand_and_modifiers() {
    let nodes = parse_body(
        r#"<button @click="increment">+</button>
<form v-on:submit.prevent.stop="save"></form>"#,
    );
    let button = element(&nodes[0]);
    assert_eq!(button.event_handlers[0].event, "click");
    assert_eq!(button.event_handlers[0].handler, "increment");
    assert!(button.event_handlers[0].modifiers.is_empty());

    let form = element(&nodes[1]);
    assert_eq!(form.event_handlers[0].event, "submit");
    assert_eq!(form.event_handlers[0].handler, "save");
    assert_eq!(form.event_handlers[0].modifiers, vec!["prevent", "stop"]);
}

#[test]
fn v_model_becomes_a_bind_binding() {
    let nodes = parse_body(r#"<input type="text" v-model="query" />"#);
    let el = element(&nodes[0]);
    assert!(el
        .bindings
        .iter()
        .any(|b| b.prop == "bind" && b.value == "query"));
}

#[test]
fn void_elements_and_comments_are_handled() {
    let nodes = parse_body(
        r#"<div>
  <!-- a comment with <div> inside -->
  <br>
  <input type="text">
</div>"#,
    );
    let el = element(&nodes[0]);
    assert_eq!(el.children.len(), 2);
    assert_eq!(element(&el.children[0]).tag, "br");
    assert_eq!(element(&el.children[1]).tag, "input");
}

#[test]
fn id_attribute_populates_element_id() {
    let nodes = parse_body(r#"<section id="main" class="flex gap-2"></section>"#);
    let el = element(&nodes[0]);
    assert_eq!(el.id.as_deref(), Some("main"));
    assert_eq!(el.classes, vec!["flex".to_string(), "gap-2".to_string()]);
}

#[test]
fn unsupported_constructs_report_errors() {
    for (src, needle) in [
        (r#"<div v-slot:header="p"></div>"#, "v-slot"),
        (r#"<div v-once></div>"#, "v-once"),
        (r#"<div v-bind="props"></div>"#, "v-bind"),
        (r#"<div :style="s"></div>"#, ":style"),
        (r#"<input v-model.number="n" />"#, "v-model"),
        (r#"<button @keyup.enter="go"></button>"#, "enter"),
        (r#"<template #header></template>"#, "slot shorthand"),
        (r#"<p v-else>x</p>"#, "v-else"),
    ] {
        let err = parse_template_with_path(
            &format!("<template>{src}</template>"),
            Some(Path::new("Comp.vue")),
        )
        .expect_err("expected unsupported construct to error");
        let message = err.to_string();
        assert!(
            message.contains(needle),
            "error for {src:?} should mention {needle:?}, got: {message}"
        );
    }
}

#[test]
fn missing_template_block_yields_no_nodes() {
    let nodes = parse("<script setup>\nconst a = 1\n</script>\n");
    assert!(nodes.is_empty());
}

#[test]
fn non_vue_paths_do_not_use_the_vue_frontend() {
    let nodes =
        parse_template_with_path("div flex\n  \"hi\"", Some(Path::new("page.crepus"))).unwrap();
    assert_eq!(element(&nodes[0]).tag, "div");
}
