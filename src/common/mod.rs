mod chunk;
pub(crate) mod json;
mod upload;

pub(crate) use chunk::{Chunk, ChunkIterator};
use std::error::Error;
pub(crate) use upload::ChunkUploader;

pub const FILE_ROUTE: &'static str = "file";

pub type Exception = Box<dyn Error + Sync + Send + 'static>;
