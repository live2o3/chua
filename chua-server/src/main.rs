mod reply;

use crate::reply::{CompleteReply, InitializeReply, UploadChunkReply};
use bytes::Buf;
use chua::{
    CompleteError, CompleteResult, InitializeError, InitializeParam, InitializeResult,
    UploadChunkError, UploadChunkResult, PART_NAME,
};
use std::convert::Infallible;
use std::error::Error;
use std::path::{Path, PathBuf};
use structopt::StructOpt;
use tokio::fs::{create_dir_all, File, OpenOptions};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::stream::StreamExt;
use uuid::Uuid;
use warp::multipart::FormData;
use warp::Filter;

#[macro_use]
extern crate log;

type Exception = Box<dyn Error + Sync + Send + 'static>;
const META_FILE_NAME: &'static str = ".meta";

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
    #[structopt(short = "s", long, parse(from_os_str))]
    static_dir: PathBuf,

    /// Path to chunk storage directory
    #[structopt(short = "t", long, parse(from_os_str))]
    temp_dir: PathBuf,
}

#[tokio::main]
async fn main() -> Result<(), Exception> {
    std::env::set_var("RUST_LOG", log::Level::Info.to_string());
    env_logger::init();

    let opts = Opts::from_args();

    let with_opts = {
        let opts = opts.clone();
        warp::any().map(move || opts.clone())
    };

    // 上传分片
    // PUT /file/{fileId}/{index}
    let upload_chunk = {
        warp::put()
            .and(with_opts.clone())
            .and(warp::path("file"))
            .and(warp::path::param())
            .and(warp::path::param())
            .and(warp::multipart::form().max_length(opts.max_chunk_size + 1024)) // 留1K给除分片之外的数据
            .and_then(
                |opts: Opts, file_id: Uuid, index: usize, mut form: FormData| async move {
                    debug!("upload_chunk: {}.{}", file_id, index);

                    while let Some(result) = form.next().await {
                        match result {
                            Ok(mut part) if part.name() == PART_NAME => {
                                return if let Some(result) = part.data().await {
                                    match result {
                                        Ok(data) => {
                                            let chunk_path = opts
                                                .temp_dir
                                                .join(file_id.to_string())
                                                .join(index.to_string());

                                            match save_chunk(&chunk_path, data).await {
                                                Ok(_size) => {
                                                    // TODO: check size

                                                    Ok::<UploadChunkReply, Infallible>(
                                                        UploadChunkResult::Ok.into(),
                                                    )
                                                }
                                                Err(e) => Ok(UploadChunkResult::Err {
                                                    error: UploadChunkError::Other(e.to_string()),
                                                }
                                                .into()),
                                            }
                                        }
                                        Err(e) => Ok(UploadChunkResult::Err {
                                            error: UploadChunkError::Other(e.to_string()),
                                        }
                                        .into()),
                                    }
                                } else {
                                    Ok(UploadChunkResult::Err {
                                        error: UploadChunkError::Size,
                                    }
                                    .into())
                                };
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

                    Ok::<UploadChunkReply, Infallible>(
                        UploadChunkResult::Err {
                            error: UploadChunkError::Size,
                        }
                        .into(),
                    )
                },
            )
    };

    // 初始化上传
    // POST /file
    let initialize = {
        warp::post()
            .and(with_opts.clone())
            .and(warp::path("file"))
            .and(warp::body::json())
            .and_then(move |opts: Opts, param: InitializeParam| {
                async move {
                    // TODO: 根据 MD5 和 size 检查文件是否已上传

                    if param.size == 0 || param.size > opts.max_file_size {
                        return Ok(InitializeResult::Err {
                            error: InitializeError::Size(opts.max_file_size),
                        }
                        .into());
                    }

                    if param.chunk_size == 0 || param.chunk_size > opts.max_chunk_size {
                        return Ok(InitializeResult::Err {
                            error: InitializeError::ChunkSize(opts.max_chunk_size),
                        }
                        .into());
                    }

                    let id = Uuid::new_v4();
                    let chunk_dir = opts.temp_dir.join(id.to_string());

                    if let Err(error) = initialize(param, &chunk_dir).await {
                        return Ok(InitializeResult::Err {
                            error: InitializeError::Other(error.to_string()),
                        }
                        .into());
                    }

                    Ok::<InitializeReply, Infallible>(
                        InitializeResult::Ok {
                            id,
                            duplicated: false,
                        }
                        .into(),
                    )
                }
            })
    };

    // 完成上传
    // POST /file/{fileId}
    let complete = warp::post()
        .and(with_opts.clone())
        .and(warp::path("file"))
        .and(warp::path::param())
        .and_then(|opts: Opts, file_id: Uuid| async move {
            debug!("upload_complete: {}", file_id);
            // 检查所有的分片是否都在
            let chunk_dir = opts.temp_dir.join(file_id.to_string());

            match build_file(file_id, opts.static_dir, &chunk_dir).await {
                Ok(meta) => {
                    info!("File {}.{} completed.", file_id, meta.extension);
                }
                Err(e) => {
                    return Ok(CompleteResult::Err {
                        error: CompleteError::Other(e.to_string()),
                    }
                    .into())
                }
            }

            Ok::<CompleteReply, Infallible>(CompleteResult::Ok.into())
        });

    let file = warp::get().and(warp::fs::dir(opts.static_dir));

    let routes = initialize.or(upload_chunk).or(complete).or(file);

    warp::serve(routes).run(([0, 0, 0, 0], opts.port)).await;

    Ok(())
}

async fn initialize(param: InitializeParam, chunk_dir: impl AsRef<Path>) -> Result<(), Exception> {
    let chunk_dir = chunk_dir.as_ref();
    create_dir_all(&chunk_dir).await?;

    let meta_file_path = chunk_dir.join(META_FILE_NAME);

    let mut meta_file = OpenOptions::new()
        .create_new(true)
        .write(true)
        .open(meta_file_path)
        .await?;

    let meta = serde_json::to_string(&param)?;

    meta_file.write_all(meta.as_bytes()).await?;

    Ok(())
}

async fn read_meta(chunk_dir: impl AsRef<Path>) -> Result<InitializeParam, Exception> {
    let meta_file_path = chunk_dir.as_ref().join(META_FILE_NAME);

    let mut meta_file = File::open(meta_file_path).await?;

    let mut meta = String::new();
    meta_file.read_to_string(&mut meta).await?;

    Ok(serde_json::from_str(&meta)?)
}

async fn build_file(
    file_id: Uuid,
    target_dir: impl AsRef<Path>,
    chunk_dir: impl AsRef<Path>,
) -> Result<InitializeParam, Exception> {
    let meta = read_meta(chunk_dir.as_ref()).await?;

    let quotient = meta.size / meta.chunk_size;
    let remainder = meta.size % meta.chunk_size;

    let chunk_count = if remainder == 0 {
        quotient
    } else {
        quotient + 1
    };

    for i in 0..chunk_count {
        let chunk_path = chunk_dir.as_ref().join(i.to_string());
        let len = chunk_path.metadata()?.len();
        let chunk_size = if i == chunk_count - 1 && remainder != 0 {
            remainder
        } else {
            meta.chunk_size
        };
        if len != chunk_size {
            return Err(format!(
                "The size of chunk {} is invalid.({}, expected: {})",
                i, len, chunk_size
            )
            .into());
        }
    }

    let target_path = {
        let mut p = target_dir.as_ref().join(file_id.to_string());
        p.set_extension(&meta.extension);
        p
    };
    let mut target = OpenOptions::new()
        .create(true)
        .write(true)
        .open(target_path)
        .await?;

    for i in 0..chunk_count {
        let mut file = File::open(chunk_dir.as_ref().join(i.to_string())).await?;

        tokio::io::copy(&mut file, &mut target).await?;
    }

    target.flush().await?;
    drop(target);

    Ok(meta)
}

async fn save_chunk(chunk_path: impl AsRef<Path>, mut data: impl Buf) -> Result<u64, Exception> {
    let mut chunk_file = OpenOptions::new()
        .create(true)
        .write(true)
        .open(&chunk_path)
        .await?;

    let mut size = 0;
    while data.has_remaining() {
        size += chunk_file.write_buf(&mut data).await? as u64;
    }

    chunk_file.flush().await?;

    Ok(size)
}
