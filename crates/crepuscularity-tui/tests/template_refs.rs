use ratatui::{backend::TestBackend, Terminal};

fn rows(terminal: &Terminal<TestBackend>) -> Vec<String> {
    let buf = terminal.backend().buffer();
    let w = buf.area.width as usize;
    let h = buf.area.height as usize;
    (0..h)
        .map(|y| {
            buf.content[y * w..(y + 1) * w]
                .iter()
                .map(|c| c.symbol())
                .collect::<String>()
        })
        .collect()
}

#[test]
fn template_refs_generate_jquery_like_handles() {
    let mut ui = crepuscularity_tui::template_refs!("tests/fixtures/ui.crepus").unwrap();
    ui.title.text("My App");
    ui.input.content = "input contents".to_string();
    ui.find("#input").unwrap().val("changed input");

    let backend = TestBackend::new(40, 4);
    let mut terminal = Terminal::new(backend).unwrap();
    terminal.draw(|frame| ui.draw_full(frame).unwrap()).unwrap();

    let text = rows(&terminal).join("\n");
    assert!(text.contains("My App"), "{text}");
    assert!(text.contains("changed input"), "{text}");
}
