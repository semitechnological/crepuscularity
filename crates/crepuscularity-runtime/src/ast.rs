/// Runtime AST — mirrors the compile-time macro AST but operates on owned strings.

#[derive(Debug, Clone)]
pub enum Node {
    Element(Element),
    Text(Vec<TextPart>),
    If(IfBlock),
    For(ForBlock),
    Match(MatchBlock),
    LetDecl(LetDecl),
    RawText(String),
}

#[derive(Debug, Clone)]
pub struct Element {
    pub tag: String,
    pub classes: Vec<String>,
    pub conditional_classes: Vec<ConditionalClass>,
    pub event_handlers: Vec<EventHandler>,
    pub bindings: Vec<Binding>,
    pub children: Vec<Node>,
}

#[derive(Debug, Clone)]
pub struct ConditionalClass {
    pub class: String,
    pub condition: String,
}

#[derive(Debug, Clone)]
pub struct EventHandler {
    pub event: String,
    pub modifiers: Vec<String>,
    pub handler: String,
}

#[derive(Debug, Clone)]
pub struct Binding {
    pub prop: String,
    pub value: String,
}

#[derive(Debug, Clone)]
pub enum TextPart {
    Literal(String),
    Expr(String),
}

#[derive(Debug, Clone)]
pub struct IfBlock {
    pub condition: String,
    pub then_children: Vec<Node>,
    pub else_children: Option<Vec<Node>>,
}

#[derive(Debug, Clone)]
pub struct ForBlock {
    pub pattern: String,
    pub iterator: String,
    pub body: Vec<Node>,
}

#[derive(Debug, Clone)]
pub struct MatchBlock {
    pub expr: String,
    pub arms: Vec<MatchArm>,
}

#[derive(Debug, Clone)]
pub struct MatchArm {
    pub pattern: String,
    pub body: Vec<Node>,
}

#[derive(Debug, Clone)]
pub struct LetDecl {
    pub name: String,
    pub expr: String,
}
