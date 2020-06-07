use crate::Exception;
use std::path::Path;
use tokio::fs::File;
use tokio::prelude::*;

#[derive(Debug)]
pub struct Chunk {
    pub size: usize,
    pub index: usize,
    pub count: usize,
    pub data: Vec<u8>,
}

#[derive(Debug)]
pub struct FileReader {
    chunk_count: usize,
    chunk_size: usize,
    remainder: usize,
    file: File,

    // current chunk
    cur_chunk: usize,
}

impl FileReader {
    pub async fn new<P: AsRef<Path>>(path: P, chunk_size: usize) -> Result<Self, Exception> {
        let file = File::open(&path).await?;

        let meta = file.metadata().await?;

        let len = meta.len() as usize;

        let remainder = len % chunk_size;

        let chunk_count = len / chunk_size + if remainder > 0 { 1 } else { 0 };

        Ok(Self {
            chunk_count,
            chunk_size,
            remainder,
            file,
            cur_chunk: 0,
        })
    }

    pub fn file_len(&self) -> usize {
        if self.chunk_count == 0 {
            0
        } else {
            (self.chunk_count - 1) * self.chunk_size + self.remainder
        }
    }
}

impl FileReader {
    fn next_pos(&mut self) -> Option<(usize, usize)> {
        let cur = self.cur_chunk;
        if cur < self.chunk_count {
            let result = if self.remainder > 0 && cur == self.chunk_count - 1 {
                (cur, self.remainder)
            } else {
                (cur, self.chunk_size)
            };

            // increase before return
            self.cur_chunk += 1;

            Some(result)
        } else {
            None
        }
    }

    pub async fn read_chunk(&mut self) -> Option<Result<Chunk, Exception>> {
        let next_pos = self.next_pos();

        match next_pos {
            None => None,
            Some((index, size)) => {
                let mut data = vec![0; size];
                match self.file.read_exact(&mut data).await {
                    Ok(_) => Some(Ok(Chunk {
                        size,
                        index,
                        count: self.chunk_count,
                        data,
                    })),
                    Err(e) => Some(Err(e.into())),
                }
            }
        }
    }
}
