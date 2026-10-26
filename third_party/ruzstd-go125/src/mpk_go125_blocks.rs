//! Private block-only access. MPK owns Go1.25's frame, prefix, checksum and
//! eight-MiB history policy; upstream's eager streaming interface is not used.
use crate::blocks::block::BlockType;
use crate::decoding::{block_decoder, decode_buffer::DecodeBuffer, scratch::DecoderScratch};
use alloc::{
    string::{String, ToString},
    vec::Vec,
};

pub struct Go125Blocks {
    scratch: DecoderScratch,
    window: usize,
}
impl Go125Blocks {
    pub fn new(window: usize) -> Self {
        Self {
            scratch: DecoderScratch::new(window),
            window,
        }
    }
    pub fn next(&mut self, input: &mut &[u8]) -> Result<(Vec<u8>, bool), String> {
        let mut decoder = block_decoder::new();
        let (header, _) = decoder
            .read_block_header(&mut *input)
            .map_err(|e| e.to_string())?;
        let size = if header.block_type == BlockType::RLE {
            header.decompressed_size
        } else {
            header.content_size
        } as usize;
        if size > 128 << 10 || self.window != 0 && size > self.window {
            return Err("Go1.25 block size".into());
        }
        let before = self.scratch.buffer.len();
        decoder
            .decode_block_content(&header, &mut self.scratch, &mut *input)
            .map_err(|e| e.to_string())?;
        let all = self.scratch.buffer.drain();
        if all.len() - before > 128 << 10 {
            return Err("Go1.25 expanded block size".into());
        }
        let result = all[before..].to_vec();
        let history = &all[all.len().saturating_sub(self.window)..];
        self.scratch.buffer = DecodeBuffer::new(self.window);
        self.scratch.buffer.push(history);
        Ok((result, header.last_block))
    }
}
