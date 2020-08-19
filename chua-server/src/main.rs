mod reply;

use crate::reply::{CompleteReply, InitializeReply, UploadChunkReply};
use chua::{
    CompleteResult, InitializeError, InitializeParam, InitializeResult, UploadChunkError,
    UploadChunkResult, PART_NAME,
};
use std::convert::Infallible;
use std::error::Error;
use std::path::PathBuf;
use structopt::StructOpt;
use tokio::stream::StreamExt;
use uuid::Uuid;
use warp::multipart::FormData;
use warp::Filter;

type Exception = Box<dyn Error + Sync + Send + 'static>;

#[derive(Debug, Clone, StructOpt)]
#[structopt(name = "chua-server")]
struct Opts {
    /// Http port to listen on
    #[structopt(short = "p", long)]
    port: u16,

    /// Max chunk size
    #[structopt(short = "c", long)]
    max_chunk_size: u64,

    /// Max file size
    #[structopt(short = "f", long)]
    max_file_size: u64,

    /// Path to static directory
    #[structopt(short = "p", long, parse(from_os_str))]
    static_path: PathBuf,
}

#[tokio::main]
async fn main() -> Result<(), Exception> {
    let Opts {
        port,
        max_chunk_size,
        max_file_size,
        static_path,
    } = Opts::from_args();

    // 上传分片
    // PUT /file/{fileId}/{index}
    let upload_chunk = warp::put()
        .and(warp::path("file"))
        .and(warp::path::param())
        .and(warp::path::param())
        .and(warp::multipart::form().max_length(max_chunk_size + 1024)) // 留1K给除分片之外的数据
        .and_then(
            |file_id: Uuid, index: usize, mut form: FormData| async move {
                println!("upload_chunk: {}.{}", file_id, index);

                while let Some(result) = form.next().await {
                    match result {
                        Ok(part) if part.name() == PART_NAME => {
                            return Ok(UploadChunkResult::Ok.into())
                        }
                        Err(e) => {
                            return Ok(UploadChunkResult::Err {
                                error: UploadChunkError::Other(e.to_string()),
                            }
                            .into())
                        }
                        _ => continue,
                    }
                }

                Ok::<UploadChunkReply, Infallible>(UploadChunkResult::Ok.into())
            },
        );

    // 初始化上传
    // POST /file
    let initialize = warp::post()
        .and(warp::path("file"))
        .and(warp::body::json())
        .and_then(move |param: InitializeParam| async move {
            let InitializeParam {
                size, chunk_size, ..
            } = param;

            if size == 0 || size > max_file_size {
                return Ok(InitializeResult::Err {
                    error: InitializeError::Size(max_file_size),
                }
                .into());
            }

            if chunk_size == 0 || chunk_size > max_chunk_size {
                return Ok(InitializeResult::Err {
                    error: InitializeError::ChunkSize(max_chunk_size),
                }
                .into());
            }

            Ok::<InitializeReply, Infallible>(
                InitializeResult::Ok {
                    id: Uuid::new_v4(),
                    duplicated: false,
                }
                .into(),
            )
        });

    // 完成上传
    // POST /file/{fileId}
    let complete = warp::post()
        .and(warp::path("file"))
        .and(warp::path::param())
        .and_then(|file_id: Uuid| async move {
            println!("upload_complete: {}", file_id);
            Ok::<CompleteReply, Infallible>(CompleteResult::Ok.into())
        });

    let file = warp::get().and(warp::fs::dir(static_path));

    let routes = initialize.or(upload_chunk).or(complete).or(file);

    warp::serve(routes).run(([0, 0, 0, 0], port)).await;

    Ok(())
}
