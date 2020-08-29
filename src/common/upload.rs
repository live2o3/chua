use crate::common::{ChuaError, Chunk, FILE_ROUTE, PART_NAME};
use crate::{ChuaResult, CompleteResult, InitializeParam, InitializeResult};
use futures::SinkExt;
use futures_channel::{mpsc, oneshot};
use reqwest::{IntoUrl, Url};
use uuid::Uuid;

#[cfg(not(target_arch = "wasm32"))]
use std::time::Duration;

#[derive(Debug, Clone)]
pub(crate) struct Uploader {
    client: reqwest::Client,
    base_url: Url,
}

impl Uploader {
    #[cfg(target_arch = "wasm32")]
    pub(crate) async fn new(base_url: impl IntoUrl) -> Result<Self, ChuaError> {
        Ok(Self {
            client: reqwest::Client::new(),
            base_url: base_url.into_url()?,
        })
    }
    #[cfg(not(target_arch = "wasm32"))]
    pub(crate) async fn new(base_url: impl IntoUrl, timeout: Duration) -> ChuaResult<Self> {
        Ok(Self {
            client: reqwest::ClientBuilder::new().timeout(timeout).build()?,
            base_url: base_url.into_url()?,
        })
    }

    pub(crate) async fn initialize(&self, param: InitializeParam) -> ChuaResult<InitializeResult> {
        let url = self.base_url.join(&format!("{}", FILE_ROUTE))?;

        let result: InitializeResult = self
            .client
            .post(url)
            .body(serde_json::to_string(&param)?)
            .send()
            .await?
            .json()
            .await?;

        Ok(result)
    }

    pub(crate) async fn complete(self, file_id: &Uuid) -> ChuaResult<CompleteResult> {
        let url = self.base_url.join(&format!("{}/{}", FILE_ROUTE, file_id))?;

        let result: CompleteResult = self.client.post(url).send().await?.json().await?;

        Ok(result)
    }

    #[cfg(not(target_arch = "wasm32"))]
    pub(crate) async fn upload_chunk(
        self,
        file_id: Uuid,
        mut sender: mpsc::UnboundedSender<oneshot::Sender<Option<Chunk<Vec<u8>>>>>,
    ) -> Result<(), ChuaError> {
        loop {
            let (os, or) = oneshot::channel();

            sender.send(os).await?;

            match or.await? {
                None => break,
                Some(chunk) => {
                    let len = chunk.data.len();
                    let index = chunk.index;

                    let resp = self.send_chunk(file_id, chunk).await?;

                    println!(
                        "{}.part{:?} ({} bytes) uploaded, response: {}.",
                        file_id, index, len, resp
                    );
                }
            }
        }

        Ok(())
    }

    // 用这个针对 wasm 单独实现的 multipart上传，就可以工作了；
    // 使用reqwest实现的send_chunk在native和wasm下都能编译，但是在 wasm下有bug，有明显卡顿且上传的分片不正确）
    #[cfg(target_arch = "wasm32")]
    pub(crate) async fn upload_chunk(
        self,
        file_id: Uuid,
        mut sender: mpsc::UnboundedSender<oneshot::Sender<Option<Chunk<web_sys::Blob>>>>,
    ) -> ChuaResult<()> {
        loop {
            let (os, or) = oneshot::channel();

            sender.send(os).await?;

            match or.await? {
                None => break,
                Some(chunk) => {
                    let index = chunk.index;

                    let resp = self.send_chunk(file_id, chunk).await?;

                    println!("{}.part{:?} uploaded, response: {}.", file_id, index, resp);
                }
            }
        }

        Ok(())
    }

    // TODO: 这段代码在 wasm32 下不能工作，考虑为 wasm32 单独实现
    #[cfg(not(target_arch = "wasm32"))]
    async fn send_chunk(&self, file_id: Uuid, chunk: Chunk<Vec<u8>>) -> ChuaResult<String> {
        use reqwest::multipart::*;

        let Chunk { index, data } = chunk;

        let file_id = file_id.to_string();
        let file = Part::bytes(data).file_name(file_id.clone());
        let form = Form::new().part(PART_NAME, file);

        let url = self
            .base_url
            .clone()
            .join(&format!("{}/{}/{}", FILE_ROUTE, file_id, index))?;

        let req = self.client.put(url).multipart(form).send().await?;

        Ok(req.text().await?)
    }

    #[cfg(target_arch = "wasm32")]
    async fn send_chunk(&self, file_id: Uuid, chunk: Chunk<web_sys::Blob>) -> ChuaResult<String> {
        use crate::wasm::runtime::promise;
        use js_sys::Uint8Array;
        use wasm_bindgen::JsValue;
        use wasm_bindgen::UnwrapThrowExt;
        use web_sys::{window, FormData, Request, RequestInit, Response};

        let Chunk { index, data } = chunk;

        let form = FormData::new().unwrap_throw();

        let js_value: &JsValue = form.as_ref();

        form.append_with_blob(PART_NAME, &data).unwrap_throw();

        let mut init = RequestInit::new();

        init.method("PUT");

        init.body(Some(js_value));

        let upload_url = format!("/{}/{}/{}", FILE_ROUTE, file_id, index);

        let js_req = match Request::new_with_str_and_init(&upload_url, &init) {
            Ok(js_req) => js_req,
            Err(e) => return Err(format!("{:?}", e).into()),
        };

        // Await the fetch() promise
        let p = window()
            .expect("window should exist")
            .fetch_with_request(&js_req);

        let js_resp = promise::<Response>(p).await.unwrap_throw();

        let status = js_resp.status();

        if status != 200 {
            return Err("status code is not 200".into());
        }

        let buf_js = promise::<JsValue>(js_resp.array_buffer().unwrap_throw())
            .await
            .unwrap_throw();

        let buffer = Uint8Array::new(&buf_js);
        let mut bytes = vec![0u8; buffer.length() as usize];
        buffer.copy_to(&mut bytes);

        Ok(String::from_utf8(bytes)?)
    }
}
