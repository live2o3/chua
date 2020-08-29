use crate::common::{ChuaError, Chunk, ChunkIterator};
use crate::ChuaResult;
use futures::future::join;
use futures::StreamExt;
use futures_channel::{mpsc, oneshot};
use std::path::Path;
use tokio::fs::File;
use tokio::prelude::*;

#[derive(Debug)]
pub(super) struct FileReader {
    size_iter: ChunkIterator,
    file: File,
}

impl FileReader {
    pub async fn new<P: AsRef<Path>>(path: P, chunk_size: u64) -> ChuaResult<(Self, u64)> {
        let file = File::open(&path).await?;

        let meta = file.metadata().await?;

        let size = meta.len();

        let size_iter = ChunkIterator::new(size, chunk_size);

        Ok((Self { size_iter, file }, size))
    }

    async fn read_chunk(&mut self) -> Option<ChuaResult<Chunk<Vec<u8>>>> {
        let next_pos = self.size_iter.next();

        match next_pos {
            None => None,
            Some((index, range)) => {
                let size = range.end - range.start;
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
        mut receiver: mpsc::UnboundedReceiver<oneshot::Sender<Option<Chunk<Vec<u8>>>>>,
    ) -> Result<(), ChuaError> {
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
