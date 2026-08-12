use crepuscularity_tui::{Template, TemplateContext, TemplateValue};
use crepuscularity_tui::ratatui::backend::TestBackend;
use crepuscularity_tui::ratatui::Terminal;

fn main() {
    let src = r#"div w-full h-full flex-col
  div h-[1] border-b border-zinc-800
    "Header"
  div flex-1 flex-col overflow-y-scroll
    for msg in {messages}
      div h-fit flex-col
        span text-xs
          "{role}"
        span text-sm
          "{text}"
  div h-[1] border-t border-zinc-800
    "Footer"
"#;
    let mut tpl = Template::from_source(src);
    let msgs = vec![
        { let mut c = TemplateContext::new(); c.set("role", "user"); c.set("text", "Hello"); c },
        { let mut c = TemplateContext::new(); c.set("role", "assistant"); c.set("text", "Hi there"); c },
        { let mut c = TemplateContext::new(); c.set("role", "user"); c.set("text", "How are you?"); c },
    ];
    tpl.set("messages", TemplateValue::List(msgs));

    let backend = TestBackend::new(40, 15);
    let mut terminal = Terminal::new(backend).unwrap();
    terminal.draw(|f| { tpl.draw(f, f.area()).unwrap(); }).unwrap();
    let buffer = terminal.backend().buffer();
    for y in 0..15 {
        let mut line = String::new();
        for x in 0..40 {
            line.push_str(&buffer[(x, y)].symbol());
        }
        println!("{y:2}|{line}|");
    }
}
