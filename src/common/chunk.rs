use std::ops::Range;

#[derive(Debug)]
pub(crate) struct Chunk {
    pub index: usize,
    pub data: Vec<u8>,
}

#[derive(Debug)]
pub(crate) struct ChunkIterator {
    chunk_count: usize,
    chunk_size: u64,
    remainder: u64,

    // current chunk index
    index: usize,
}

impl ChunkIterator {
    pub fn new(size: u64, chunk_size: u64) -> Self {
        let remainder = size % chunk_size;

        let chunk_count = (size / chunk_size) as usize + if remainder > 0 { 1 } else { 0 };

        Self {
            chunk_count,
            chunk_size,
            remainder,
            index: 0,
        }
    }
}

impl Iterator for ChunkIterator {
    type Item = (usize, Range<u64>);

    fn next(&mut self) -> Option<Self::Item> {
        let cur = self.index;

        if cur < self.chunk_count {
            let start = self.index as u64 * self.chunk_size;

            let result = if self.remainder > 0 && cur == self.chunk_count - 1 {
                (cur, start..start + self.remainder)
            } else {
                (cur, start..start + self.chunk_size)
            };

            // increase before return
            self.index += 1;

            Some(result)
        } else {
            None
        }
    }
}
