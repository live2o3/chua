mod file;
mod runtime;

use crate::common::{ChunkUploader, Exception};
use file::FileReader;
use futures_channel::mpsc;
use gloo_file::Blob;
use reqwest::Url;
use uuid::Uuid;

pub async fn upload(
    base_url: Url,
    file: web_sys::File,
    chunk_size: u64,
    parallel: usize,
) -> Result<(), Exception> {
    let reader = FileReader::new(Blob::from(file), chunk_size);

    let (sender, receiver) = mpsc::unbounded();

    runtime::spawn(async move { reader.run(receiver).await });

    let client = reqwest::Client::new();

    let file_id = Uuid::new_v4();

    let mut vec = Vec::with_capacity(parallel);

    for _ in 0..parallel {
        let uploader = ChunkUploader::new(client.clone(), base_url.clone(), file_id.clone());
        vec.push(runtime::spawn(uploader.run(sender.clone())));
    }

    let _ = futures::future::join_all(vec).await;

    Ok(())
}
