use wasm_bindgen::prelude::*;

#[wasm_bindgen]
pub async fn upload(
    base_url: String,
    file: web_sys::File,
    chunk_size: f64,
    parallel: usize,
) -> Result<JsValue, JsValue> {
    match chua::upload(&base_url, file, chunk_size as u64, parallel).await {
        Ok(uuid) => Ok(JsValue::from_str(&uuid.to_string())),
        Err(e) => Err(JsValue::from_str(&e.to_string())),
    }
}
