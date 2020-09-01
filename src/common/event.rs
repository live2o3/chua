use crate::{
    ChuaResult, CompleteError, CompleteResult, InitializeResult, UploadChunkResult, UploadParam,
    FILE_ROUTE,
};
use futures::channel::mpsc::{channel, Receiver, Sender};
use futures::{SinkExt, Stream};
use reqwest::IntoUrl;
use std::ops::Range;
use std::path::PathBuf;
use std::pin::Pin;
use std::task::{Context, Poll};
use std::time::Duration;
use tokio::stream::StreamExt;
use url::Url;
use uuid::Uuid;

#[derive(Debug)]
pub enum Event {
    Initialized {
        param: UploadParam,
        result: ChuaResult<InitializeResult>,
    },
    ChunkUploaded {
        file_id: Uuid,
        index: usize,
        result: ChuaResult<UploadChunkResult>,
    },
    Completed {
        file_id: Uuid,
        result: ChuaResult<CompleteResult>,
    },
}

pub struct Emitter(Sender<Event>);

impl Emitter {
    pub async fn emit(&mut self, event: Event) -> ChuaResult<()> {
        Ok(self.0.send(event).await?)
    }
}

pub struct Progress(Receiver<Event>);

impl Stream for Progress {
    type Item = Event;

    fn poll_next(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        Pin::new(&mut self.get_mut().0).poll_next(cx)
    }
}

fn progress_channel() -> (Emitter, Progress) {
    let (sender, receiver) = channel(0);
    (Emitter(sender), Progress(receiver))
}

#[derive(Clone)]
pub struct ChuaClient {
    client: reqwest::Client,
    base_url: Url,
}

impl ChuaClient {
    pub fn new(
        base_url: impl IntoUrl,
        #[cfg(not(target_arch = "wasm32"))] timeout: Duration,
    ) -> ChuaResult<Self> {
        let base_url = base_url.into_url()?;

        #[cfg(target_arch = "wasm32")]
        let client = reqwest::Client::new();

        #[cfg(not(target_arch = "wasm32"))]
        let client = reqwest::ClientBuilder::new().timeout(timeout).build()?;

        Ok(Self { client, base_url })
    }

    pub async fn resume_upload(
        &self,
        file_id: Uuid,
        path: impl AsRef<PathBuf>,
        parallel: usize,
    ) -> ChuaResult<Progress> {
        // complete url
        let url = self.base_url.join(&format!("{}/{}", FILE_ROUTE, file_id))?;
        let file_size = path.as_ref().metadata()?.len();
        let client = self.client.clone();

        let (mut emitter, progress) = progress_channel();

        tokio::spawn(async move {
            let result: CompleteResult = client.post(url).send().await?.json().await?;

            match result {
                CompleteResult::Ok => {}
                CompleteResult::Err { error } => match error {
                    CompleteError::Incomplete { param, ranges } => {
                        if param.size != file_size {
                            return Err(format!(
                                "file size error: (expected: {}, actual: {})",
                                param.size, file_size
                            )
                            .into());
                        }

                        // TODO: do upload
                    }
                    CompleteError::MD5 { expected, actual } => {
                        //Err(format!("md5 error(expected: {}, actual: {})", expected, actual).into())
                    }
                    CompleteError::Other { detail } => {} //Err(detail.into()),
                },
            }

            let _ = emitter;

            ChuaResult::Ok(())
        });

        Ok(progress)
    }

    pub fn new_upload(
        &self,
        path: impl AsRef<PathBuf>,
        chunk_size: u64,
        parallel: usize,
    ) -> ChuaResult<Progress> {
        let (emitter, progress) = progress_channel();

        tokio::spawn(async move {
            // TODO: do upload
            let _ = emitter;
        });

        Ok(progress)
    }

    pub fn get_base_url(&self) -> &Url {
        &self.base_url
    }
}
