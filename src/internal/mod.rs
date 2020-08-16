use crate::internal::file::FileReader;
use crate::internal::upload::ChunkUploader;
use futures_channel::mpsc;
use reqwest::IntoUrl;
use std::error::Error;
use std::path::Path;
use std::time::{Duration, Instant};
use uuid::Uuid;

mod file;
mod upload;

pub type Exception = Box<dyn Error + Sync + Send + 'static>;

pub async fn upload<P: AsRef<Path>, U: IntoUrl>(
    url: U,
    path: P,
    chunk_size: u64,
    parallel: usize,
) -> Result<(), Exception> {
    let path = path.as_ref();
    if !path.is_file() {
        return Err("The path is not pointing a regular file".into());
    }

    let url = url.into_url()?;

    let reader = FileReader::new(path, chunk_size).await?;

    let file_len = reader.file_len();

    let (sender, receiver) = mpsc::unbounded();

    tokio::spawn(reader.run(receiver));

    let client = reqwest::ClientBuilder::new()
        .timeout(Duration::from_secs(20))
        .build()?;

    let file_id = Uuid::new_v4();

    let start = Instant::now();

    let mut vec = Vec::with_capacity(parallel);

    for _ in 0..parallel {
        let uploader = ChunkUploader::new(client.clone(), url.clone(), file_id.clone());
        vec.push(tokio::spawn(uploader.run(sender.clone())));
    }

    let _ = futures::future::join_all(vec).await;

    println!(
        "file: {}, size: {} bytes, time: {}ms",
        path.display(),
        file_len,
        start.elapsed().as_millis()
    );

    Ok(())
}
