use std::io;
use std::num::Wrapping;

use crate::basehan::BASE_OFFSET;
use crate::basehan::v1::BitCache8Out::{Double, Single};

const DEFAULT_BUFFER_SIZE: usize = 1024 * 1024; // 1 MiB
const ENDING_OFFSET: u32 = 0x6e00;

#[derive(Debug)]
pub enum BaseHanError {
    IoError(io::Error),
    EndOfFile, // Remaining byte in BitCache
}


pub struct BaseHanEncoder {
    buf_out: Vec<char>,
    remainings: BitCache13,
}

impl BaseHanEncoder {
    pub fn new() -> Self {
        BaseHanEncoder {
            buf_out: Vec::with_capacity(DEFAULT_BUFFER_SIZE),
            remainings: BitCache13::default(),
        }
    }

    pub fn with_buffer_size(buffer_size: usize) -> Self {
        BaseHanEncoder {
            buf_out: Vec::with_capacity(buffer_size),
            remainings: BitCache13::default(),
        }
    }

    pub fn update<T>(&mut self, chunk: T) -> Result<Vec<char>, BaseHanError>
    where
        T: AsRef<[u8]>,
    {
        let buf_in = chunk.as_ref();

        for &byte in buf_in {
            if let Some(out) = self.remainings.fill(byte) {
                self.buf_out.push(out);
            }
        }

        let buf_out = std::mem::replace(&mut self.buf_out, Vec::new()); // Replace buffer with new & return the taken value
        return Ok(buf_out);
    }

    /// Dump the remaining bits out.
    pub fn finish(self) -> char {
        self.remainings.dump()
    }
}

#[derive(Default)]
struct BitCache13 {
    inner: u32,
    nbits: usize,
}

impl BitCache13 {
    /// Fill one byte at a time, if full(13 bits), return char and pop it.
    /// Otherwise, return none.

    pub(crate) fn fill(&mut self, byte: u8) -> Option<char> {
        let remain_bits = (self.nbits + 8) % 13;
        let out = match self.nbits {
            0..=4 => { // Not full, return none
                self.inner <<= 8;
                self.inner |= byte as u32;
                None
            }
            5..=12 => {
                self.inner <<= 8;
                self.inner |= byte as u32;
                let output_char_u32 = self.inner >> ((self.nbits + 8) % 13);
                self.inner = self.inner & ((1 << remain_bits) - 1); // head padding nums overflows in u8, and then appended to the buffer
                let output_char = char::from_u32(output_char_u32 + BASE_OFFSET)
                    .expect("Data cannot convert to a valid char, which should never happen.");
                Some(output_char)
            }
            13.. =>
                panic!("Remaining bits overflow! This should never happen!")
        };
        self.nbits = (self.nbits + 8) % 13;
        return out;
    }

    /// Dump remaining bits to a char ranging from 0x6e00 to 0x7e00, indicating the end of stream.
    /// Since the inner is left aligned, it needs to be aligned right in this case.
    /// Otherwise, hint: 1000 0000 0000 can refer to 1 or 10 or 100 or and so on.
    pub(crate) fn dump(self) -> char {
        let out_char_u32 = self.inner + ENDING_OFFSET;
        let out_char = char::from_u32(out_char_u32)
            .expect("Data cannot convert to a valid char, which should never happen.");
        return out_char;
    }
}

pub struct BaseHanDecoder {
    buf_out: Vec<u8>,
    remainings: BitCache8,
    eof: bool,
}

impl BaseHanDecoder {
    pub fn new() -> Self {
        BaseHanDecoder {
            buf_out: Vec::with_capacity(DEFAULT_BUFFER_SIZE),
            remainings: BitCache8::default(),
            eof: false,
        }
    }

    pub fn with_buffer_size(buffer_size: usize) -> Self {
        BaseHanDecoder {
            buf_out: Vec::with_capacity(buffer_size),
            remainings: BitCache8::default(),
            eof: false,
        }
    }

    pub fn update<T>(&mut self, chunk: T) -> Result<Vec<u8>, BaseHanError>
    where
        T: AsRef<[char]>,
    {
        if self.eof {
            return Err(BaseHanError::EndOfFile);
        }
        let buf_in = chunk.as_ref();

        for &c in buf_in {
            let mut c = c as u32;
            if c == 0 {break};
            self.eof = c >= ENDING_OFFSET;
            if self.eof {
                c -= ENDING_OFFSET;
                c <<= self.remainings.nbits + 5;
            } else {
                c -= BASE_OFFSET;
            }
            match self.remainings.fill(c) {
                Single(byte) => {
                    self.buf_out.push(byte);
                }
                Double(bytes) => {
                    self.buf_out.extend_from_slice(&bytes);
                }
            }
            if self.eof {
                break;
            }
        }

        let buf_out = std::mem::replace(&mut self.buf_out, Vec::new()); // Replace buffer with new & return the taken value
        return Ok(buf_out);
    }

    pub fn finish(self) -> Option<u8> {
        self.remainings.dump()
    }
}

#[derive(Debug)]
enum BitCache8Out {
    Single(u8),
    Double([u8; 2]),
}

#[derive(Default)]
struct BitCache8 {
    inner: u32,
    nbits: usize,
}


impl BitCache8 {
    /// Fill 13 bits at a time. The remaining bits are left-aligned (the same as BitCache13)
    /// Return one byte or 2 bytes
    pub(crate) fn fill(&mut self, bits: u32) -> BitCache8Out {
        let remain_bits = (self.nbits + 13) % 8;
        let out = match self.nbits {
            0..=2 => {
                self.inner <<= 13;
                self.inner |= bits;
                let out_byte = (self.inner >> remain_bits) as u8;
                self.inner = self.inner & ((1 << remain_bits) - 1);
                Single(out_byte)
            }
            3.. => {
                self.inner <<= 13;
                self.inner |= bits;
                let out_byte_1 = (self.inner >> (remain_bits + 8)) as u8;
                let out_byte_2 =(self.inner >> remain_bits) as u8;
                self.inner = self.inner & ((1 << remain_bits) - 1);
                Double([out_byte_1, out_byte_2])
            }
        };
        self.nbits = (self.nbits + 13) % 8;
        return out;
    }

    /// Dump the remaining byte out.
    /// Typically, this is expected to return none when reaching the last character.
    pub(crate) fn dump(self) -> Option<u8> {
        if self.inner != 0 {
            return Some(self.inner as u8);
        }
        None
    }
}




