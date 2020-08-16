use crate::internal::Exception;
use futures::future::join;
use futures::StreamExt;
use futures_channel::{mpsc, oneshot};
use std::path::Path;
use tokio::fs::File;
use tokio::prelude::*;

#[derive(Debug)]
pub struct Chunk {
    pub index: usize,
    pub data: Vec<u8>,
}

#[derive(Debug)]
pub(super) struct FileReader {
    chunk_count: usize,
    chunk_size: u64,
    remainder: u64,
    file: File,

    // current chunk
    cur_chunk: usize,
}

impl FileReader {
    pub async fn new<P: AsRef<Path>>(path: P, chunk_size: u64) -> Result<Self, Exception> {
        let file = File::open(&path).await?;

        let meta = file.metadata().await?;

        let len = meta.len();

        let remainder = len % chunk_size;

        let chunk_count = (len / chunk_size) as usize + if remainder > 0 { 1 } else { 0 };

        Ok(Self {
            chunk_count,
            chunk_size,
            remainder,
            file,
            cur_chunk: 0,
        })
    }

    pub fn file_len(&self) -> u64 {
        if self.chunk_count == 0 {
            0
        } else {
            (self.chunk_count - 1) as u64 * self.chunk_size + self.remainder
        }
    }

    fn next_pos(&mut self) -> Option<(usize, u64)> {
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

    async fn read_chunk(&mut self) -> Option<Result<Chunk, Exception>> {
        let next_pos = self.next_pos();

        match next_pos {
            None => None,
            Some((index, size)) => {
                let mut data = vec![0; size as usize];
                match self.file.read_exact(&mut data).await {
                    Ok(_) => Some(Ok(Chunk { index, data })),
                    Err(e) => Some(Err(e.into())),
                }
            }
        }
    }

    pub(crate) async fn run(
        mut self,
        mut receiver: mpsc::UnboundedReceiver<oneshot::Sender<Option<Chunk>>>,
    ) -> Result<(), Exception> {
        while let (Some(sender), read_chunk) = join(receiver.next(), self.read_chunk()).await {
            match read_chunk {
                Some(result) => match result {
                    Ok(chunk) => sender
                        .send(Some(chunk))
                        .map_err(|_| format!("cannot send data to send_loop"))?,
                    Err(e) => return Err(e.into()),
                },
                None => {
                    sender
                        .send(None)
                        .map_err(|_| format!("cannot send EOF to send_loop"))?;
                    break;
                }
            }
        }

        Ok(())
    }
}
