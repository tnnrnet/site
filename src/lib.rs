#![recursion_limit = "256"]

pub mod app;
pub mod content;

#[cfg(feature = "hydrate")]
#[wasm_bindgen::prelude::wasm_bindgen]
pub fn hydrate() {
    use crate::app::*;
    console_error_panic_hook::set_once();
    leptos::mount::hydrate_body(App);
    // Event handlers are attached now. End-to-end tests wait for this
    // attribute before clicking anything that needs the WASM side.
    if let Some(html) = web_sys::window()
        .and_then(|w| w.document())
        .and_then(|d| d.document_element())
    {
        let _ = html.set_attribute("data-hydrated", "");
    }
}
