use super::runtime::get_slice;
use crate::common::{Chunk, ChunkIterator, Exception};
use futures::future::join;
use futures::StreamExt;
use futures_channel::{mpsc, oneshot};

#[derive(Debug)]
pub(super) struct FileReader {
    size_iter: ChunkIterator,
    file: web_sys::Blob,
}

impl FileReader {
    pub fn new(file: web_sys::Blob, chunk_size: u64) -> (Self, u64) {
        let size = file.size() as u64;

        let size_iter = ChunkIterator::new(size, chunk_size);

        (Self { size_iter, file }, size)
    }

    async fn read_chunk(&mut self) -> Option<Result<Chunk<web_sys::Blob>, Exception>> {
        let next_pos = self.size_iter.next();

        match next_pos {
            None => None,
            Some((index, range)) => {
                let data = get_slice(&self.file, range.start, range.end);
                Some(Ok(Chunk { index, data }))
            }
        }
    }

    pub(crate) async fn run(
        mut self,
        mut receiver: mpsc::UnboundedReceiver<oneshot::Sender<Option<Chunk<web_sys::Blob>>>>,
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
