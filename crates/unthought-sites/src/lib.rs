//! Unthought Sites - Site builder using Crepuscularity templates
//!
//! This crate transforms Unthought's `WebsiteTemplatePayload` (JSON from the editor/AI)
//! into static HTML sites using Crepuscularity's `.crepus` template engine.
//!
//! ## Usage
//!
//! ```rust,ignore
//! use unthought_sites::{build_site, SitePayload};
//!
//! let payload: SitePayload = serde_json::from_str(json)?;
//! let html = build_site(&payload)?;
//! ```
//!
//! ## WASM Usage
//!
//! Build with the `wasm` feature enabled:
//! ```bash
//! wasm-pack build --target web --features wasm
//! ```
//!
//! Then call from JavaScript:
//! ```javascript
//! import init, { build_site_wasm } from './unthought_sites.js';
//! await init();
//! const html = build_site_wasm(JSON.stringify(payload));
//! ```

pub mod starter;
pub mod transform;
pub mod types;

pub use starter::starter_site_payload;
pub use transform::{build_site, build_site_from_json};
pub use types::*;

// WASM bindings
#[cfg(feature = "wasm")]
mod wasm {
    use wasm_bindgen::prelude::*;

    /// Initialize panic hook for better error messages in WASM
    #[wasm_bindgen(start)]
    pub fn init() {
        console_error_panic_hook::set_once();
    }

    /// Build a site from a JSON payload string.
    ///
    /// # Arguments
    /// * `payload_json` - JSON string containing the SitePayload
    ///
    /// # Returns
    /// HTML string on success, or throws an error
    #[wasm_bindgen]
    pub fn build_site_wasm(payload_json: &str) -> Result<String, JsError> {
        super::build_site_from_json(payload_json)
            .map(|result| result.html)
            .map_err(|e| JsError::new(&e.to_string()))
    }

    /// Get the version of the unthought-sites crate
    #[wasm_bindgen]
    pub fn version() -> String {
        env!("CARGO_PKG_VERSION").to_string()
    }
}
