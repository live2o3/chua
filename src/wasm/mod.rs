mod file;
pub(crate) mod runtime;

use crate::common::Uploader;
use crate::{ChuaResult, CompleteResult, InitializeParam, InitializeResult};
use file::FileReader;
use futures_channel::mpsc;
use reqwest::IntoUrl;
use uuid::Uuid;

pub async fn upload(
    base_url: impl IntoUrl,
    file: web_sys::File,
    chunk_size: u64,
    parallel: usize,
) -> ChuaResult<Uuid> {
    let name: String = file.name();

    let extension = match name.rfind('.') {
        None => "",
        Some(index) => &name[index + 1..],
    }
    .into();

    let (reader, size) = FileReader::new(file.into(), chunk_size);

    let uploader = Uploader::new(base_url).await?;

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

    runtime::spawn(async move { reader.run(receiver).await });

    // Chrome 和 Firefox 的默认并行连接数都是 6
    let parallel = if parallel == 0 { 6 } else { parallel };

    let mut vec = Vec::with_capacity(parallel);

    for _ in 0..parallel {
        let uploader = uploader.clone();
        vec.push(runtime::spawn(
            uploader.upload_chunk(file_id, sender.clone()),
        ));
    }

    let _ = futures::future::join_all(vec).await;

    if let CompleteResult::Err { error } = uploader.complete(&file_id).await? {
        return Err(format!("{:?}", error).into());
    }

    Ok(file_id)
}
