#[cfg(all(target_arch = "wasm32", feature = "dom"))]
mod tests {
    use crepuscularity_web::dom::{ElementRef, StaticElementRef};
    use wasm_bindgen_test::*;

    #[wasm_bindgen_test]
    fn test_element_ref_attr_error() {
        let el = ElementRef::new("missing-attr");
        let err = el.attr("class", "foo").unwrap_err();
        let err_msg = err.as_string().unwrap();
        assert!(
            err_msg == "window.document is unavailable"
                || err_msg == "missing DOM id `#missing-attr`",
            "unexpected error message: {}",
            err_msg
        );
    }

    #[wasm_bindgen_test]
    fn test_static_element_ref_attr_error() {
        let el = StaticElementRef::new("missing-attr-static");
        let err = el.attr("class", "foo").unwrap_err();
        let err_msg = err.as_string().unwrap();
        assert!(
            err_msg == "window.document is unavailable"
                || err_msg == "missing DOM id `#missing-attr-static`",
            "unexpected error message: {}",
            err_msg
        );
    }
}
