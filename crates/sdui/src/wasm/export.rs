/// Exports the worker API for one application constructor and its state type.
///
/// The invoking crate must depend directly on `wasm-bindgen`.
#[macro_export]
macro_rules! export_application {
    ($application:path, $state:ty $(,)?) => {
        ::std::thread_local! {
            static APPLICATION: $crate::WasmApplication<$state> =
                $crate::WasmApplication::new($application());
        }

        #[::wasm_bindgen::prelude::wasm_bindgen]
        pub fn initial_messages() -> Result<String, ::wasm_bindgen::JsValue> {
            APPLICATION.with($crate::WasmApplication::initial_messages)
        }

        #[::wasm_bindgen::prelude::wasm_bindgen]
        pub fn snapshot_messages() -> Result<String, ::wasm_bindgen::JsValue> {
            APPLICATION.with($crate::WasmApplication::snapshot_messages)
        }

        #[::wasm_bindgen::prelude::wasm_bindgen]
        pub async fn handle_action(input: &str) -> Result<String, ::wasm_bindgen::JsValue> {
            APPLICATION
                .with(|application| application.clone())
                .handle_action(input)
                .await
        }
    };
}
