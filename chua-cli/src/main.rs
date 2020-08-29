use chua::{upload, ChuaResult};
use std::path::PathBuf;
use structopt::StructOpt;
use url::Url;

/// 欻(chua), 文件分片上传工具
#[derive(Debug, Clone, StructOpt)]
#[structopt(name = "chua-cli")]
struct Opts {
    /// url to post
    #[structopt(short, long)]
    base_url: Url,

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
async fn main() -> ChuaResult<()> {
    let Opts {
        base_url,
        file,
        chunk_size,
        parallel,
    } = Opts::from_args();

    let file_id = upload(base_url, &file, chunk_size, parallel).await?;

    println!("File {} uploaded.(id: {})", file.display(), file_id);

    Ok(())
}
