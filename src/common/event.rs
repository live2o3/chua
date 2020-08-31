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

pub fn event_channel() -> (EventSender, EventReceiver) {
    let (sender, receiver) = channel(0);
    (EventSender(sender), EventReceiver(receiver))
}

pub struct EventSender(Sender<Event>);

impl EventSender {
    pub async fn send(&mut self, event: Event) -> ChuaResult<()> {
        Ok(self.0.send(event).await?)
    }
}

#[derive(Clone)]
pub struct ChuaClient {
    client: reqwest::Client,
    base_url: Url,
}

pub enum Resume {
    Uploading(Chua),
    Completed(Uuid),
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
    ) -> ChuaResult<Resume> {
        let url = self.base_url.join(&format!("{}/{}", FILE_ROUTE, file_id))?;
        let result: CompleteResult = self.client.post(url).send().await?.json().await?;

        match result {
            CompleteResult::Ok => Ok(Resume::Completed(file_id)),
            CompleteResult::Err { error } => match error {
                CompleteError::Incomplete {
                    param:
                        UploadParam {
                            size, chunk_size, ..
                        },
                    ranges,
                } => {
                    let file_size = path.as_ref().metadata()?.len();
                    if size != file_size {
                        return Err(format!(
                            "file size error: (expected: {}, actual: {})",
                            size, file_size
                        )
                        .into());
                    }

                    Ok(Resume::Uploading(Chua {
                        client: self.clone(),
                        path: path.as_ref().to_path_buf(),
                        chunk_size,
                        parallel,
                        ranges: Some(ranges),
                    }))
                }
                CompleteError::MD5 { expected, actual } => {
                    Err(format!("md5 error(expected: {}, actual: {})", expected, actual).into())
                }
                CompleteError::Other { detail } => Err(detail.into()),
            },
        }
    }

    pub fn new_upload(
        &self,
        path: impl AsRef<PathBuf>,
        chunk_size: u64,
        parallel: usize,
    ) -> ChuaResult<Chua> {
        Ok(Chua {
            client: self.clone(),
            path: path.as_ref().to_path_buf(),
            chunk_size,
            parallel,
            ranges: None,
        })
    }

    pub fn get_base_url(&self) -> &Url {
        &self.base_url
    }
}

pub struct Chua {
    client: ChuaClient,
    path: PathBuf,
    chunk_size: u64,
    parallel: usize,
    ranges: Option<Vec<Range<usize>>>,
}

impl Chua {
    pub async fn run(self) -> ChuaResult<()> {
        Ok(())
    }
}

pub struct EventReceiver(Receiver<Event>);

impl Stream for EventReceiver {
    type Item = Event;

    fn poll_next(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        Pin::new(&mut self.get_mut().0).poll_next(cx)
    }
}
