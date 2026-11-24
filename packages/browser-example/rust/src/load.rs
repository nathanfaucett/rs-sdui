use a2ui::BoxError;
#[cfg(not(target_arch = "wasm32"))]
use futures::future::ready;
use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct LoadedTodo {
    pub title: String,
    pub completed: bool,
}

#[cfg(not(target_arch = "wasm32"))]
pub fn load_examples() -> impl Future<Output = Result<Vec<LoadedTodo>, BoxError>> + Send {
    let result = serde_json::from_str(include_str!("../../public/todos.json")).map_err(Into::into);
    ready(result)
}

#[cfg(target_arch = "wasm32")]
pub async fn load_examples() -> Result<Vec<LoadedTodo>, BoxError> {
    use wasm_bindgen::JsCast;
    use wasm_bindgen_futures::JsFuture;

    let request = if let Some(window) = web_sys::window() {
        window.fetch_with_str("/todos.json")
    } else {
        let worker = js_sys::global()
            .dyn_into::<web_sys::WorkerGlobalScope>()
            .map_err(|_| "neither a browser window nor worker scope is available")?;
        worker.fetch_with_str("/todos.json")
    };
    let response = JsFuture::from(request)
        .await
        .map_err(|error| format!("failed to fetch example tasks: {error:?}"))?;
    let response: web_sys::Response = response
        .dyn_into()
        .map_err(|error| format!("fetch did not return a Response: {error:?}"))?;
    if !response.ok() {
        return Err(format!("failed to fetch example tasks: HTTP {}", response.status()).into());
    }
    let text = JsFuture::from(
        response
            .text()
            .map_err(|error| format!("failed to read example tasks: {error:?}"))?,
    )
    .await
    .map_err(|error| format!("failed to read example tasks: {error:?}"))?;
    let text = text.as_string().ok_or("the response body was not text")?;
    Ok(serde_json::from_str(&text)?)
}
