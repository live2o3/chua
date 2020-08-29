mod file;

use crate::common::{ChuaError, Uploader};
use crate::{ChuaResult, CompleteResult, InitializeParam, InitializeResult};
use file::FileReader;
use futures_channel::mpsc;
use reqwest::IntoUrl;
use std::path::Path;
use std::time::Duration;
use uuid::Uuid;

pub async fn upload(
    base_url: impl IntoUrl,
    path: impl AsRef<Path>,
    chunk_size: u64,
    parallel: usize,
) -> ChuaResult<Uuid> {
    let path = path.as_ref();
    if !path.is_file() {
        return Err(ChuaError::Other(
            "The path is not pointing a regular file".into(),
        ));
    }

    let extension = match path.extension() {
        None => String::new(),
        Some(ext) => ext.to_str().unwrap_or("").to_string(),
    };

    let (reader, size) = FileReader::new(path, chunk_size).await?;

    let uploader = Uploader::new(base_url, Duration::from_secs(20)).await?;

    let init_param = InitializeParam {
        size,
        chunk_size,
        extension,
        md5: "".to_string(),
    };

    let file_id = match uploader.initialize(init_param).await {
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

    let parallel = if parallel == 0 {
        num_cpus::get()
    } else {
        parallel
    };

    let mut vec = Vec::with_capacity(parallel);

    for _ in 0..parallel {
        let uploader = uploader.clone();
        vec.push(tokio::spawn(uploader.upload_chunk(file_id, sender.clone())));
    }

    let _ = futures::future::join_all(vec).await;

    if let CompleteResult::Err { error } = uploader.complete(&file_id).await? {
        return Err(format!("{:?}", error).into());
    }

    Ok(file_id)
}
