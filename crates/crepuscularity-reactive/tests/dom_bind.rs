#[cfg(all(target_arch = "wasm32", feature = "dom"))]
mod tests {
    use crepuscularity_reactive::{bind_class, Signal};
    use wasm_bindgen_test::*;
    use web_sys::window;

    // We can test all of them together to be efficient.

    #[wasm_bindgen_test]
    fn test_bind_class() {
        let win = window();
        if win.is_none() {
            return;
        }
        let document = win.unwrap().document().expect("document not found");
        let el = document.create_element("div").unwrap();
        el.set_id("test-class");

        let body = document.body().unwrap();
        body.append_child(&el).unwrap();

        let sig = Signal::new(false);

        let sig_clone = sig.clone();
        let _effect = bind_class("test-class", "active", move || sig_clone.get());

        // Initially false, so should not have the class
        assert_eq!(el.class_list().contains("active"), false);

        // Set to true
        sig.set(true);
        assert_eq!(el.class_list().contains("active"), true);

        // Set back to false
        sig.set(false);
        assert_eq!(el.class_list().contains("active"), false);

        // Let's clean up
        body.remove_child(&el).unwrap();
    }

    #[wasm_bindgen_test]
    fn test_bind_class_missing_el() {
        // Missing element shouldn't panic
        let sig = Signal::new(false);
        let sig_clone = sig.clone();
        let _effect = bind_class("non-existent-class-el", "active", move || sig_clone.get());
        sig.set(true);
        sig.set(false);
    }
}
