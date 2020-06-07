use crate::file::FileReader;
use crate::upload::{ranger_loop, send_loop};
use std::error::Error;
use std::path::Path;
use std::time::Duration;
use tokio::sync::mpsc;
use tokio::time::Instant;

mod file;
mod upload;

pub type Exception = Box<dyn Error + Sync + Send + 'static>;

pub async fn upload<P: AsRef<Path>>(
    url: &str,
    path: P,
    chunk_size: usize,
    parallel: usize,
) -> Result<(), Exception> {
    let path = path.as_ref();
    if !path.is_file() {
        return Err("The path is not pointing a regular file".into());
    }

    let file_name = match path.file_name() {
        Some(name) => match name.to_str() {
            Some(s) => s.to_string(),
            None => return Err("The filename is not a valid Unicode string".into()),
        },
        None => return Err("The path termiates in \"..\"".into()),
    };

    let ranger = FileReader::new(path, chunk_size).await?;

    let file_len = ranger.file_len();

    let (sender, receiver) = mpsc::unbounded_channel();

    tokio::spawn(ranger_loop(receiver, ranger));

    let client = reqwest::ClientBuilder::new()
        .timeout(Duration::from_secs(20))
        .build()?;

    let start = Instant::now();

    let mut vec = Vec::with_capacity(parallel);

    for _ in 0..parallel {
        vec.push(tokio::spawn(send_loop(
            client.clone(),
            url.to_owned(),
            file_name.clone(),
            sender.clone(),
        )));
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

pub fn upload_blocking<P: AsRef<Path>>(
    url: &str,
    path: P,
    chunk_size: usize,
    parallel: usize,
) -> Result<(), Exception> {
    lazy_static::lazy_static! {
        static ref RUNTIME: tokio::runtime::Runtime = tokio::runtime::Builder::new()
            .threaded_scheduler()
            .enable_all()
            .build()
            .unwrap();
    }

    RUNTIME
        .handle()
        .clone()
        .block_on(upload(url, path, chunk_size, parallel))
}
