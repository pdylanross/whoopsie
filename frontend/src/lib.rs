#[allow(clippy::single_component_path_imports)]
#[allow(unused_imports)]
use app;

#[wasm_bindgen::prelude::wasm_bindgen]
pub fn hydrate() {
    use app::*;
    tracing::init_tracing();

    leptos::mount::hydrate_body(App);
}
