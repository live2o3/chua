use chua::{upload, Exception};
use std::path::PathBuf;
use structopt::StructOpt;
use url::Url;

/// 欻(chua), 文件分片上传工具
#[derive(Debug, Clone, StructOpt)]
#[structopt(name = "chua")]
struct Opts {
    /// url to post
    #[structopt(short, long)]
    url: Url,

    /// parallelism
    #[structopt(short, long)]
    parallel: usize,

    /// chunk Size
    #[structopt(short, long)]
    chunk_size: u64,

    /// file to upload
    #[structopt(short, long, parse(from_os_str))]
    file: PathBuf,
}

#[tokio::main]
async fn main() -> Result<(), Exception> {
    let Opts {
        url,
        file,
        chunk_size,
        parallel,
    } = Opts::from_args();

    upload(url, file, chunk_size, parallel).await
}
