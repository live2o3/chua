use crate::common::{Chunk, ChunkIterator};
use futures::future::join;
use futures::StreamExt;
use futures_channel::{mpsc, oneshot};
use gloo_file::futures::read_as_array_buffer;
use gloo_file::Blob;
use wasm_bindgen::UnwrapThrowExt;

#[derive(Debug)]
pub(super) struct FileReader {
    size_iter: ChunkIterator,
    file: Blob,
}

impl FileReader {
    pub fn new(file: Blob, chunk_size: u64) -> Self {
        let size = file.size();

        let size_iter = ChunkIterator::new(size, chunk_size);

        Self { size_iter, file }
    }

    async fn read_chunk(&mut self) -> Option<Chunk> {
        let next_pos = self.size_iter.next();

        match next_pos {
            None => None,
            Some((index, range)) => {
                let slice = self.file.slice(range.start, range.end);
                match read_as_array_buffer(&slice).await {
                    Ok(buffer) => {
                        let data: Vec<u8> = js_sys::Uint8Array::new_with_byte_offset_and_length(
                            &buffer,
                            0,
                            buffer.byte_length(),
                        )
                        .to_vec();
                        Some(Chunk { index, data })
                    }
                    Err(e) => None,
                }
            }
        }
    }

    pub(crate) async fn run(
        mut self,
        mut receiver: mpsc::UnboundedReceiver<oneshot::Sender<Option<Chunk>>>,
    ) {
        while let (Some(sender), read_chunk) = join(receiver.next(), self.read_chunk()).await {
            match read_chunk {
                Some(chunk) => sender
                    .send(Some(chunk))
                    .map_err(|_| format!("cannot send data to send_loop"))
                    .unwrap_throw(),
                None => {
                    sender
                        .send(None)
                        .map_err(|_| format!("cannot send EOF to send_loop"))
                        .unwrap_throw();
                    break;
                }
            }
        }
    }
}
