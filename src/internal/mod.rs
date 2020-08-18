use crate::common::{complete, initialize, ChunkUploader, Exception};
use crate::internal::file::FileReader;
use crate::{CompleteResult, InitializeParam, InitializeResult};
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
) -> Result<Uuid, Exception> {
    let path = path.as_ref();
    if !path.is_file() {
        return Err("The path is not pointing a regular file".into());
    }

    let base_url = base_url.into_url()?;

    let (reader, size) = FileReader::new(path, chunk_size).await?;

    let client = reqwest::ClientBuilder::new()
        .timeout(Duration::from_secs(20))
        .build()?;

    let extension = match path.extension() {
        None => String::new(),
        Some(ext) => ext.to_str().unwrap_or("").to_string(),
    };

    let init_param = InitializeParam {
        size,
        chunk_size,
        extension,
        md5: "".to_string(),
    };

    let file_id = match initialize(base_url.clone(), &client, init_param).await {
        Ok(result) => match result {
            InitializeResult::Ok { id, duplicated } => {
                if duplicated {
                    return Ok(id);
                }

                id
            }
            InitializeResult::Err { error } => return Err(format!("{:?}", error).into()),
        },
        Err(e) => return Err(e),
    };

    let (sender, receiver) = mpsc::unbounded();

    tokio::spawn(reader.run(receiver));

    let mut vec = Vec::with_capacity(parallel);

    for _ in 0..parallel {
        let uploader = ChunkUploader::new(client.clone(), base_url.clone(), file_id.clone());
        vec.push(tokio::spawn(uploader.run(sender.clone())));
    }

    let _ = futures::future::join_all(vec).await;

    if let CompleteResult::Err { error } = complete(base_url.clone(), &client, &file_id).await? {
        return Err(format!("{:?}", error).into());
    }

    Ok(file_id)
}
