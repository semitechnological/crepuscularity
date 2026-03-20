/// Runtime AST — mirrors the compile-time macro AST but operates on owned strings.

#[derive(Debug, Clone)]
pub enum Node {
    Element(Element),
    Text(Vec<TextPart>),
    If(IfBlock),
    For(ForBlock),
    Match(MatchBlock),
    LetDecl(LetDecl),
    Include(IncludeNode),
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
    /// If true, only sets the variable when it is not already present in the context.
    /// Used for component prop defaults: `$: default name = value`
    pub is_default: bool,
}

/// An `include` directive — embeds another `.crepus` file as a component.
///
/// ```text
/// include components/button.crepus label="Click me" count={total}
///     div p-4
///         "slot content"
/// ```
#[derive(Debug, Clone)]
pub struct IncludeNode {
    /// Relative path to the included `.crepus` file.
    pub path: String,
    /// Props passed to the component: (key, expr_string) pairs.
    /// The expr_string is evaluated against the parent context.
    pub props: Vec<(String, String)>,
    /// Children of the include directive — become the component's slot content.
    pub slot: Vec<Node>,
}
