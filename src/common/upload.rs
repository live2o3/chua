use crate::common::{Chunk, Exception, FILE_ROUTE, PART_NAME};
use crate::{CompleteResult, InitializeParam, InitializeResult};
use futures::SinkExt;
use futures_channel::{mpsc, oneshot};
use reqwest::{IntoUrl, Url};
use uuid::Uuid;

#[derive(Debug)]
pub(crate) struct ChunkUploader {
    client: reqwest::Client,
    base_url: Url,
    file_id: Uuid,
}

impl ChunkUploader {
    pub fn new(client: reqwest::Client, base_url: Url, file_id: Uuid) -> Self {
        Self {
            client,
            base_url,
            file_id,
        }
    }

    async fn send_part(&self, chunk: Chunk) -> Result<String, Exception> {
        use reqwest::multipart::*;

        let Chunk { index, data } = chunk;

        let file_id = self.file_id.to_string();
        let file = Part::bytes(data).file_name(file_id.clone());
        let form = Form::new().part(PART_NAME, file);

        let url = self
            .base_url
            .clone()
            .join(&format!("{}/{}/{}", FILE_ROUTE, file_id, index))?;

        let req = self.client.put(url).multipart(form).send().await?;

        Ok(req.text().await?)
    }

    pub async fn run(
        self,
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

                    let resp = self.send_part(chunk).await?;

                    println!(
                        "{}.part{:?} ({} bytes) uploaded, response: {}.",
                        self.file_id, index, len, resp
                    );
                }
            }
        }

        Ok(())
    }
}

pub(crate) async fn initialize(
    base_url: impl IntoUrl,
    client: &reqwest::Client,
    param: InitializeParam,
) -> Result<InitializeResult, Exception> {
    let url = base_url.into_url()?.join(&format!("{}", FILE_ROUTE))?;

    let result: InitializeResult = client
        .post(url)
        .body(serde_json::to_string(&param)?)
        .send()
        .await?
        .json()
        .await?;

    Ok(result)
}

pub(crate) async fn complete(
    base_url: impl IntoUrl,
    client: &reqwest::Client,
    file_id: &Uuid,
) -> Result<CompleteResult, Exception> {
    let url = base_url
        .into_url()?
        .join(&format!("{}/{}", FILE_ROUTE, file_id))?;

    let result: CompleteResult = client.post(url).send().await?.json().await?;

    Ok(result)
}
