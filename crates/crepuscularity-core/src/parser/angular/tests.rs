use std::path::Path;

use crate::ast::*;
use crate::parser::parse_template_with_path;

fn parse(src: &str) -> Vec<Node> {
    parse_template_with_path(src, Some(Path::new("app.component.html")))
        .expect("angular template should parse")
}

fn parse_err(src: &str) -> String {
    parse_template_with_path(src, Some(Path::new("app.component.html")))
        .expect_err("expected angular parse error")
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
fn mustache_interpolation_becomes_text_parts() {
    let nodes = parse(r#"<p>Hello {{ name }}!</p>"#);
    let Node::Text(parts) = &element(&nodes[0]).children[0] else {
        panic!("expected text");
    };
    assert!(matches!(&parts[0], TextPart::Literal(s) if s == "Hello "));
    assert!(matches!(&parts[1], TextPart::Expr(e) if e == "name"));
    assert!(matches!(&parts[2], TextPart::Literal(s) if s == "!"));
}

#[test]
fn ng_if_wraps_the_element_in_an_if_block() {
    let nodes = parse(r#"<div><p *ngIf="visible">hi</p></div>"#);
    let Node::If(block) = &element(&nodes[0]).children[0] else {
        panic!("expected if");
    };
    assert_eq!(block.condition, "visible");
    assert_eq!(element(&block.then_children[0]).tag, "p");
    assert!(block.else_children.is_none());
}

#[test]
fn ng_for_wraps_the_element_in_a_for_block() {
    let nodes = parse(r#"<li *ngFor="let item of items; trackBy: byId">{{ item }}</li>"#);
    let Node::For(block) = &nodes[0] else {
        panic!("expected for");
    };
    assert_eq!(block.pattern, "item");
    assert_eq!(block.iterator, "items");
    assert_eq!(text_of(&element(&block.body[0]).children[0]), "{item}");
}

#[test]
fn property_and_event_bindings() {
    let nodes = parse(r#"<button [disabled]="busy" (click)="save()" title="go">+</button>"#);
    let el = element(&nodes[0]);
    assert_eq!(el.bindings[0].prop, "disabled");
    assert_eq!(el.bindings[0].value, "busy");
    assert_eq!(el.bindings[1].prop, "title");
    assert_eq!(el.bindings[1].value, "\"go\"");
    assert_eq!(el.event_handlers[0].event, "click");
    assert_eq!(el.event_handlers[0].handler, "save()");
    assert!(el.event_handlers[0].modifiers.is_empty());
}

#[test]
fn two_way_ng_model_becomes_a_bind_binding() {
    let nodes = parse(r#"<input type="text" [(ngModel)]="query" />"#);
    let el = element(&nodes[0]);
    assert!(el
        .bindings
        .iter()
        .any(|b| b.prop == "bind" && b.value == "query"));
}

#[test]
fn class_bindings_lower_to_conditional_and_dynamic_classes() {
    let nodes = parse(
        r#"<div class="base" [class.active]="isOn"></div>
<div [ngClass]="{ danger: bad, 'a b': other }"></div>
<div [ngClass]="['x', dyn]"></div>"#,
    );
    let first = element(&nodes[0]);
    assert_eq!(first.classes, vec!["base".to_string()]);
    assert_eq!(first.conditional_classes[0].class, "active");
    assert_eq!(first.conditional_classes[0].condition, "isOn");

    let obj = element(&nodes[1]);
    assert_eq!(obj.conditional_classes.len(), 3);
    assert_eq!(obj.conditional_classes[0].class, "danger");
    assert_eq!(obj.conditional_classes[0].condition, "bad");
    assert_eq!(obj.conditional_classes[1].class, "a");

    let arr = element(&nodes[2]);
    assert_eq!(arr.classes, vec!["x".to_string(), "{dyn}".to_string()]);
}

#[test]
fn attribute_and_inner_html_bindings() {
    let nodes = parse(r#"<div [attr.aria-label]="label" [innerHTML]="body">dropped</div>"#);
    let el = element(&nodes[0]);
    assert_eq!(el.bindings[0].prop, "aria-label");
    assert_eq!(el.bindings[0].value, "label");
    assert!(matches!(&el.children[0], Node::RawHtml(e) if e == "body"));
}

#[test]
fn interpolated_attribute_value_becomes_a_binding() {
    let nodes = parse(r#"<a href="{{ url }}">x</a>"#);
    let el = element(&nodes[0]);
    assert_eq!(el.bindings[0].prop, "href");
    assert_eq!(el.bindings[0].value, "url");
}

#[test]
fn ng_container_contributes_children_only() {
    let nodes = parse(r#"<div><ng-container *ngIf="ok"><b>a</b><i>b</i></ng-container></div>"#);
    let Node::If(block) = &element(&nodes[0]).children[0] else {
        panic!("expected if");
    };
    assert_eq!(block.then_children.len(), 2);
    assert_eq!(element(&block.then_children[0]).tag, "b");
}

#[test]
fn control_flow_if_else_if_else_blocks() {
    let nodes = parse(
        r#"<div>
  @if (a) { <p>A</p> } @else if (b) { <p>B</p> } @else { <p>C</p> }
</div>"#,
    );
    let Node::If(outer) = &element(&nodes[0]).children[0] else {
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
fn control_flow_for_block() {
    let nodes = parse(r#"<ul>@for (item of items; track item.id) { <li>{{ item }}</li> }</ul>"#);
    let Node::For(block) = &element(&nodes[0]).children[0] else {
        panic!("expected for");
    };
    assert_eq!(block.pattern, "item");
    assert_eq!(block.iterator, "items");
    assert_eq!(element(&block.body[0]).tag, "li");
}

#[test]
fn nested_control_flow_blocks() {
    let nodes = parse(r#"@for (x of xs; track x) { @if (x.on) { <b>{{ x.n }}</b> } }"#);
    let Node::For(block) = &nodes[0] else {
        panic!("expected for");
    };
    let Node::If(inner) = &block.body[0] else {
        panic!("expected nested if, got {:?}", block.body[0]);
    };
    assert_eq!(inner.condition, "x.on");
    assert_eq!(element(&inner.then_children[0]).tag, "b");
}

#[test]
fn void_elements_and_comments_are_handled() {
    let nodes = parse(
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
fn unsupported_constructs_report_errors() {
    for (src, needle) in [
        (r#"<ng-template><p>x</p></ng-template>"#, "<ng-template>"),
        (r#"<ng-content></ng-content>"#, "<ng-content>"),
        (r#"<div *ngSwitch="v"></div>"#, "ngSwitch"),
        (r#"<p *ngIf="a; else other">x</p>"#, "else"),
        (r#"<div [ngStyle]="s"></div>"#, "ngStyle"),
        (r#"<div [style.width]="w"></div>"#, "style.width"),
        (r#"<input #ref />"#, "template reference variable"),
        (r#"<button (keyup.enter)="go()"></button>"#, "pseudo-event"),
        (r#"<div [(foo)]="v"></div>"#, "two-way binding"),
        (
            r#"<li *ngFor="let item of items; let i = index"></li>"#,
            "for-loop head",
        ),
        (r#"<a href="a-{{ b }}-c">x</a>"#, "interpolation mixed"),
        (r#"@switch (v) { }"#, "@switch"),
        (
            r#"@for (x of xs; track x) { <b>a</b> } @empty { <i>none</i> }"#,
            "@empty",
        ),
        (r#"@else { <p>x</p> }"#, "without a preceding"),
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
fn dispatcher_keys_on_the_angular_naming_conventions() {
    for name in ["app.component.html", "app.ng.html", "app.ng"] {
        let nodes = parse_template_with_path(r#"<p>{{ x }}</p>"#, Some(Path::new(name)))
            .unwrap_or_else(|e| panic!("{name} should use the angular frontend: {e}"));
        assert_eq!(text_of(&element(&nodes[0]).children[0]), "{x}");
    }
}

#[test]
fn plain_html_paths_do_not_use_the_angular_frontend() {
    let err = parse_template_with_path(r#"<div *ngIf="a"></div>"#, Some(Path::new("index.html")));
    // The JSX frontend accepts `*ngIf` as an ordinary attribute, so a plain
    // `.html` file is not claimed by this frontend.
    let nodes = err.expect("jsx frontend should handle plain html");
    assert_eq!(element(&nodes[0]).tag, "div");
}
