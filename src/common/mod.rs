mod chunk;
mod error;
pub(crate) mod json;
mod upload;

pub(crate) use chunk::{Chunk, ChunkIterator};
pub(crate) use upload::Uploader;

pub const FILE_ROUTE: &'static str = "file";
pub const PART_NAME: &'static str = "chunk";

pub use error::*;
