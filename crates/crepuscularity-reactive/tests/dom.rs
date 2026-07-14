#[cfg(all(target_arch = "wasm32", feature = "dom"))]
mod tests {
    use crepuscularity_reactive::{bind_attr, bind_class, bind_text, Signal};
    use wasm_bindgen_test::*;
    use web_sys::{window, Element};

    wasm_bindgen_test_configure!(run_in_browser);

    fn create_test_element(id: &str) -> Element {
        let doc = window().unwrap().document().unwrap();
        let el = doc.create_element("div").unwrap();
        el.set_id(id);
        doc.body().unwrap().append_child(&el).unwrap();
        el
    }

    #[wasm_bindgen_test]
    fn test_bind_text_reactive() {
        let el = create_test_element("test-bind-text");
        let sig = Signal::new("initial".to_string());

        let sig_clone = sig.clone();
        let _effect = bind_text("test-bind-text", move || sig_clone.get());

        assert_eq!(el.text_content().unwrap(), "initial");

        sig.set("updated".to_string());

        assert_eq!(el.text_content().unwrap(), "updated");
    }

    #[wasm_bindgen_test]
    fn test_bind_class_reactive() {
        let el = create_test_element("test-bind-class");
        let sig = Signal::new(false);
        let sig_clone = sig.clone();

        let _effect = bind_class("test-bind-class", "active", move || sig_clone.get());

        assert!(!el.class_list().contains("active"));

        sig.set(true);
        assert!(el.class_list().contains("active"));

        sig.set(false);
        assert!(!el.class_list().contains("active"));
    }

    #[wasm_bindgen_test]
    fn test_bind_attr_reactive() {
        let el = create_test_element("test-bind-attr");
        let sig = Signal::new("foo".to_string());
        let sig_clone = sig.clone();

        let _effect = bind_attr("test-bind-attr", "data-test", move || sig_clone.get());

        assert_eq!(el.get_attribute("data-test").unwrap(), "foo");

        sig.set("bar".to_string());

        assert_eq!(el.get_attribute("data-test").unwrap(), "bar");
    }
}
