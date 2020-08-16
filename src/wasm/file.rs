use futures::future::join;
use futures::StreamExt;
use futures_channel::{mpsc, oneshot};
use gloo_file::futures::read_as_array_buffer;
use gloo_file::Blob;
use wasm_bindgen::UnwrapThrowExt;

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
    file: Blob,

    // current chunk
    cur_chunk: usize,
}

impl FileReader {
    pub fn new(file: Blob, chunk_size: u64) -> Self {
        let len = file.size();

        let remainder = len % chunk_size;
        let chunk_count = (len / chunk_size) as usize + if remainder > 0 { 1 } else { 0 };

        Self {
            chunk_count,
            chunk_size,
            remainder,
            file,
            cur_chunk: 0,
        }
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

    async fn read_chunk(&mut self) -> Option<Chunk> {
        let next_pos = self.next_pos();

        match next_pos {
            None => None,
            Some((index, size)) => {
                let start = index as u64 * self.chunk_size;
                let end = start + size;
                let slice = self.file.slice(start, end);
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
