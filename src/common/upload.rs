use crate::common::{Chunk, Exception, FILE_ROUTE, PART_NAME};
use crate::{CompleteResult, InitializeParam, InitializeResult};
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
    pub(crate) async fn new(base_url: impl IntoUrl) -> Result<Self, Exception> {
        Ok(Self {
            client: reqwest::Client::new(),
            base_url: base_url.into_url()?,
        })
    }
    #[cfg(not(target_arch = "wasm32"))]
    pub(crate) async fn new(base_url: impl IntoUrl, timeout: Duration) -> Result<Self, Exception> {
        Ok(Self {
            client: reqwest::ClientBuilder::new().timeout(timeout).build()?,
            base_url: base_url.into_url()?,
        })
    }

    pub(crate) async fn initialize(
        &self,
        param: InitializeParam,
    ) -> Result<InitializeResult, Exception> {
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

    pub(crate) async fn complete(self, file_id: &Uuid) -> Result<CompleteResult, Exception> {
        let url = self.base_url.join(&format!("{}/{}", FILE_ROUTE, file_id))?;

        let result: CompleteResult = self.client.post(url).send().await?.json().await?;

        Ok(result)
    }

    pub(crate) async fn upload_chunk(
        self,
        file_id: Uuid,
        mut sender: mpsc::UnboundedSender<oneshot::Sender<Option<Chunk>>>,
    ) -> Result<(), Exception> {
        loop {
            let (os, or) = oneshot::channel();

            sender.send(os).await?;

            match or.await? {
                None => break,
                Some(chunk) => {
                    let len = chunk.data.len();
                    let index = chunk.index;

                    let resp = self.send_part(file_id, chunk).await?;

                    println!(
                        "{}.part{:?} ({} bytes) uploaded, response: {}.",
                        file_id, index, len, resp
                    );
                }
            }
        }

        Ok(())
    }

    async fn send_part(&self, file_id: Uuid, chunk: Chunk) -> Result<String, Exception> {
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
}
