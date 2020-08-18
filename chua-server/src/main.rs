mod reply;

use crate::reply::{CompleteReply, InitializeReply, UploadChunkReply};
use chua::{
    CompleteResult, InitializeParam, InitializeResult, UploadChunkError, UploadChunkResult,
    PART_NAME,
};
use std::convert::Infallible;
use std::error::Error;
use tokio::stream::StreamExt;
use uuid::Uuid;
use warp::multipart::FormData;
use warp::Filter;

type Exception = Box<dyn Error + Sync + Send + 'static>;

const MAX_CHUNK_SIZE: u64 = 1024 * 1024 * 10;

#[tokio::main]
async fn main() -> Result<(), Exception> {
    // 上传分片
    // PUT /file/{fileId}/{index}
    let upload_chunk = warp::put()
        .and(warp::path("file"))
        .and(warp::path::param())
        .and(warp::path::param())
        .and(warp::multipart::form().max_length(MAX_CHUNK_SIZE)) // 最大20M
        .and_then(upload_chunk);

    // 初始化上传
    // POST /file
    let initialize = warp::post()
        .and(warp::path("file"))
        .and(warp::body::json())
        .and_then(upload_initialize);

    // 完成上传
    // POST /file/{fileId}
    let complete = warp::post()
        .and(warp::path("file"))
        .and(warp::path::param())
        .and_then(upload_complete);

    let file = warp::get().and(warp::fs::dir("./public/"));

    let routes = initialize.or(upload_chunk).or(complete).or(file);

    warp::serve(routes).run(([0, 0, 0, 0], 8080)).await;

    Ok(())
}

async fn upload_initialize(param: InitializeParam) -> Result<InitializeReply, Infallible> {
    println!("upload_initialize: {:?}", param);
    Ok(InitializeResult::Ok {
        id: Uuid::new_v4(),
        duplicated: false,
    }
    .into())
}

async fn upload_complete(file_id: Uuid) -> Result<CompleteReply, Infallible> {
    println!("upload_complete: {}", file_id);
    Ok(CompleteResult::Ok.into())
}

async fn upload_chunk(
    file_id: Uuid,
    index: usize,
    mut form: FormData,
) -> Result<UploadChunkReply, Infallible> {
    println!("upload_chunk: {}.{}", file_id, index);

    while let Some(result) = form.next().await {
        match result {
            Ok(part) if part.name() == PART_NAME => return Ok(UploadChunkResult::Ok.into()),
            Err(e) => {
                return Ok(UploadChunkResult::Err {
                    error: UploadChunkError::Other(e.to_string()),
                }
                .into())
            }
            _ => continue,
        }
    }

    Ok(UploadChunkResult::Ok.into())
}
