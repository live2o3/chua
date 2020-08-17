mod file;
mod upload;
mod util;

use file::FileReader;
use gloo_file::Blob;
use reqwest::Url;
use uuid::Uuid;

pub async fn upload(_url: Url, file: web_sys::File, chunk_size: u64, _parallel: usize) {
    let _client = reqwest::Client::new();

    let _file_id = Uuid::new_v4();

    let _reader = FileReader::new(Blob::from(file), chunk_size);

    // TODO: WIP
}
