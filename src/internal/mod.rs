use crate::common::{ChunkUploader, Exception};
use crate::internal::file::FileReader;
use futures_channel::mpsc;
use reqwest::IntoUrl;
use std::path::Path;
use std::time::Duration;
use uuid::Uuid;

mod file;

pub async fn upload<P: AsRef<Path>, U: IntoUrl>(
    base_url: U,
    path: P,
    chunk_size: u64,
    parallel: usize,
) -> Result<(), Exception> {
    let path = path.as_ref();
    if !path.is_file() {
        return Err("The path is not pointing a regular file".into());
    }

    let base_url = base_url.into_url()?;

    let (reader, _size) = FileReader::new(path, chunk_size).await?;

    let client = reqwest::ClientBuilder::new()
        .timeout(Duration::from_secs(20))
        .build()?;

    // TODO: 这里的文件ID应该是从服务器端请求的
    let file_id = Uuid::new_v4();

    let (sender, receiver) = mpsc::unbounded();

    tokio::spawn(reader.run(receiver));

    let mut vec = Vec::with_capacity(parallel);

    for _ in 0..parallel {
        let uploader = ChunkUploader::new(client.clone(), base_url.clone(), file_id.clone());
        vec.push(tokio::spawn(uploader.run(sender.clone())));
    }

    let _ = futures::future::join_all(vec).await;

    Ok(())
}
