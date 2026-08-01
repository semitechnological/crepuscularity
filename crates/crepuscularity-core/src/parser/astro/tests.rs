use std::path::Path;

use crate::ast::*;
use crate::parser::parse_template_with_path;

fn parse(src: &str) -> Vec<Node> {
    parse_template_with_path(src, Some(Path::new("Page.astro"))).expect("astro should parse")
}

fn parse_err(src: &str) -> String {
    parse_template_with_path(src, Some(Path::new("Page.astro")))
        .expect_err("expected astro parse error")
        .to_string()
}

fn element(node: &Node) -> &Element {
    match node {
        Node::Element(el) => el,
        other => panic!("expected element, got {other:?}"),
    }
}

fn text_of(node: &Node) -> String {
    match node {
        Node::Text(parts) => parts
            .iter()
            .map(|p| match p {
                TextPart::Literal(s) => s.clone(),
                TextPart::Expr(e) => format!("{{{e}}}"),
            })
            .collect(),
        other => panic!("expected text, got {other:?}"),
    }
}

#[test]
fn frontmatter_is_skipped_and_never_parsed_as_markup() {
    let nodes = parse(
        r#"---
import Card from "../Card.astro";
const markup = "<div class='nope'></div>";
---
<section class="p-4"><p>ok</p></section>"#,
    );
    assert_eq!(nodes.len(), 1);
    let section = element(&nodes[0]);
    assert_eq!(section.tag, "section");
    assert_eq!(section.classes, vec!["p-4".to_string()]);
    assert_eq!(element(&section.children[0]).tag, "p");
}

#[test]
fn elements_attributes_and_self_closing_tags() {
    let nodes = parse(r#"<img src="/a.png" alt="A" width={w} /><br /><hr>"#);
    let img = element(&nodes[0]);
    assert_eq!(img.tag, "img");
    assert_eq!(img.bindings[0].prop, "src");
    assert_eq!(img.bindings[0].value, "\"/a.png\"");
    assert_eq!(img.bindings[2].prop, "width");
    assert_eq!(img.bindings[2].value, "w");
    assert_eq!(element(&nodes[1]).tag, "br");
    assert_eq!(element(&nodes[2]).tag, "hr");
}

#[test]
fn interpolation_stays_inside_the_text_run() {
    let nodes = parse(r#"<p>Hello {name}!</p>"#);
    let p = element(&nodes[0]);
    assert_eq!(text_of(&p.children[0]), "Hello {name}!");
}

#[test]
fn shorthand_attribute_expands_to_a_named_binding() {
    let nodes = parse(r#"<a {href}>x</a>"#);
    let a = element(&nodes[0]);
    assert_eq!(a.bindings[0].prop, "href");
    assert_eq!(a.bindings[0].value, "href");
}

#[test]
fn logical_and_expression_becomes_an_if_block() {
    let nodes = parse(r#"<div>{show && <span>yes</span>}</div>"#);
    let div = element(&nodes[0]);
    let Node::If(block) = &div.children[0] else {
        panic!("expected if, got {:?}", div.children[0]);
    };
    assert_eq!(block.condition, "show");
    assert_eq!(element(&block.then_children[0]).tag, "span");
    assert!(block.else_children.is_none());
}

#[test]
fn ternary_expression_becomes_an_if_else_block() {
    let nodes = parse(r#"<div>{ok ? <b>y</b> : <i>n</i>}</div>"#);
    let div = element(&nodes[0]);
    let Node::If(block) = &div.children[0] else {
        panic!("expected if");
    };
    assert_eq!(block.condition, "ok");
    assert_eq!(element(&block.then_children[0]).tag, "b");
    let Some(else_nodes) = block.else_children.as_deref() else {
        panic!("expected else branch");
    };
    assert_eq!(element(&else_nodes[0]).tag, "i");
}

#[test]
fn map_call_becomes_a_for_block() {
    let nodes = parse(r#"<ul>{items.map((item) => (<li>{item.name}</li>))}</ul>"#);
    let ul = element(&nodes[0]);
    let Node::For(block) = &ul.children[0] else {
        panic!("expected for, got {:?}", ul.children[0]);
    };
    assert_eq!(block.pattern, "item");
    assert_eq!(block.iterator, "items");
    assert_eq!(element(&block.body[0]).tag, "li");
}

#[test]
fn map_call_without_parens_around_the_body() {
    let nodes = parse(r#"<ul>{posts.map(post => <li>{post.title}</li>)}</ul>"#);
    let Node::For(block) = &element(&nodes[0]).children[0] else {
        panic!("expected for");
    };
    assert_eq!(block.pattern, "post");
    assert_eq!(block.iterator, "posts");
}

#[test]
fn plain_expression_stays_an_interpolation() {
    let nodes = parse(r#"<p>{items.map(i => i.name).join(", ")}</p>"#);
    let p = element(&nodes[0]);
    assert_eq!(
        text_of(&p.children[0]),
        r#"{items.map(i => i.name).join(", ")}"#
    );
}

#[test]
fn comments_are_dropped() {
    let nodes = parse(
        r#"<div>
  <!-- an html comment with <span> inside -->
  {/* a javascript comment */}
  <b>x</b>
</div>"#,
    );
    let div = element(&nodes[0]);
    assert_eq!(div.children.len(), 1);
    assert_eq!(element(&div.children[0]).tag, "b");
}

#[test]
fn fragment_contributes_children_only() {
    let nodes = parse(r#"<div><Fragment><b>a</b><i>b</i></Fragment></div>"#);
    let div = element(&nodes[0]);
    assert_eq!(div.children.len(), 2);
    assert_eq!(element(&div.children[0]).tag, "b");
    assert_eq!(element(&div.children[1]).tag, "i");
}

#[test]
fn class_list_lowers_to_classes_and_conditional_classes() {
    let nodes = parse(
        r#"<div class="base" class:list={["a", { active: isOn, "text-red-500 font-bold": danger }, dyn]}></div>"#,
    );
    let el = element(&nodes[0]);
    assert_eq!(
        el.classes,
        vec!["base".to_string(), "a".to_string(), "{dyn}".to_string()]
    );
    assert_eq!(el.conditional_classes.len(), 3);
    assert_eq!(el.conditional_classes[0].class, "active");
    assert_eq!(el.conditional_classes[0].condition, "isOn");
    assert_eq!(el.conditional_classes[1].class, "text-red-500");
    assert_eq!(el.conditional_classes[1].condition, "danger");
    assert_eq!(el.conditional_classes[2].class, "font-bold");
}

#[test]
fn set_html_and_set_text_replace_children() {
    let nodes = parse(r#"<div set:html={body}>dropped</div><span set:text={label}></span>"#);
    let html = element(&nodes[0]);
    assert!(matches!(&html.children[0], Node::RawHtml(e) if e == "body"));
    let text = element(&nodes[1]);
    assert_eq!(text_of(&text.children[0]), "{label}");
}

#[test]
fn client_directives_are_recorded_as_bindings() {
    let nodes = parse(r#"<div client:load client:media="(max-width: 50em)"></div>"#);
    let el = element(&nodes[0]);
    assert_eq!(el.bindings[0].prop, "client:load");
    assert_eq!(el.bindings[0].value, "true");
    assert_eq!(el.bindings[1].prop, "client:media");
    assert_eq!(el.bindings[1].value, "\"(max-width: 50em)\"");
}

#[test]
fn unsupported_constructs_report_errors() {
    for (src, needle) in [
        (r#"<slot />"#, "<slot />"),
        (r#"<Card title="x" />"#, "component tag"),
        (r#"<div {...props}></div>"#, "spread attributes"),
        (r#"<div transition:name="hero"></div>"#, "transition:"),
        (r#"<div define:vars={{a: 1}}></div>"#, "define:vars"),
        (
            r#"<ul>{items.map((item, i) => <li>{i}</li>)}</ul>"#,
            "index bindings",
        ),
        (
            r#"<ul>{items.map(({ a }) => <li>{a}</li>)}</ul>"#,
            "destructuring",
        ),
        (
            r#"<ul>{items.map(i => { return <li /> })}</ul>"#,
            "block-bodied",
        ),
        (r#"<div class:active={on}></div>"#, "class:list"),
        (r#"<div><b>x</b></div"#, "expected </div>"),
    ] {
        let message = parse_err(src);
        assert!(
            message.contains(needle),
            "error for {src:?} should mention {needle:?}, got: {message}"
        );
    }
}

#[test]
fn non_astro_paths_do_not_use_the_astro_frontend() {
    let nodes =
        parse_template_with_path("div flex\n  \"hi\"", Some(Path::new("page.crepus"))).unwrap();
    assert_eq!(element(&nodes[0]).tag, "div");
}
