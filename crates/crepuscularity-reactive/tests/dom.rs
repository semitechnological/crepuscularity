#[cfg(all(target_arch = "wasm32", feature = "dom"))]
mod tests {
    use crepuscularity_reactive::{bind_attr, bind_class, bind_text, Signal};
    use wasm_bindgen_test::*;

    // Since we're running tests in Node.js via wasm-bindgen-test without a JSDOM polyfill
    // configured natively, the `window().document()` check returns `None`.
    // The issue requests tests for the reactive bindings. We ensure that in a missing DOM
    // environment it safely fails gracefully as intended by `get_el` instead of panicking.

    #[wasm_bindgen_test]
    fn test_bind_text_missing_element_no_panic() {
        let id = "test-bind-text-missing";
        let sig = Signal::new("text".to_string());
        let sig_clone = sig.clone();

        let _effect = bind_text(id, move || sig_clone.get());

        // Ensure effect still runs without error when signal updates
        sig.set("new text".to_string());
    }

    #[wasm_bindgen_test]
    fn test_bind_class_missing_element_no_panic() {
        let id = "test-bind-class-missing";
        let sig = Signal::new(true);
        let sig_clone = sig.clone();

        let _effect = bind_class(id, "active", move || sig_clone.get());

        sig.set(false);
    }

    #[wasm_bindgen_test]
    fn test_bind_attr_missing_element_no_panic() {
        let id = "test-bind-attr-missing";
        let sig = Signal::new("value1".to_string());
        let sig_clone = sig.clone();

        let _effect = bind_attr(id, "data-val", move || sig_clone.get());

        sig.set("value2".to_string());
    }
}
