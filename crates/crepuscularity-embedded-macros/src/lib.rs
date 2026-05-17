//! Proc-macros for [`crepuscularity-embedded`](https://docs.rs/crepuscularity-embedded).

use crepuscularity_core::parser::parse_template;
use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, LitStr};

/// Parse-check a `.crepus` template at compile time and expand to its source `&str`.
///
/// ```ignore
/// use crepuscularity_embedded::Ui;
/// use crepuscularity_embedded_macros::embedded_template;
///
/// let mut ui = Ui::new(240, 320, embedded_template!("ui/dashboard.crepus"));
/// ```
///
/// Equivalent to `include_str!(…)` plus a build-time parse error if the template is invalid.
#[proc_macro]
pub fn embedded_template(input: TokenStream) -> TokenStream {
    let path = parse_macro_input!(input as LitStr);
    let path_value = path.value();
    let manifest_dir = std::env::var("CARGO_MANIFEST_DIR").unwrap_or_else(|_| ".".into());
    let full = std::path::Path::new(&manifest_dir).join(&path_value);
    let source = std::fs::read_to_string(&full).unwrap_or_else(|e| {
        panic!("embedded_template!: could not read {}: {e}", full.display());
    });
    if let Err(err) = parse_template(&source) {
        panic!(
            "embedded_template!: invalid .crepus at {}: {err}",
            full.display()
        );
    }
    quote! {{
        include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/", #path_value))
    }}
    .into()
}
