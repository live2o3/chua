mod chunk;
pub(crate) mod json;
mod upload;

pub(crate) use chunk::{Chunk, ChunkIterator};
use std::error::Error;
pub(crate) use upload::Uploader;

pub const FILE_ROUTE: &'static str = "file";
pub const PART_NAME: &'static str = "chunk";

pub type Exception = Box<dyn Error + Sync + Send + 'static>;
