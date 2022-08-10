use std::{
    io::{Read, Seek},
    iter::Enumerate,
};

pub struct Seeker {
    buffer: Vec<u8>,
}

impl Seeker {
    pub fn new() -> Self {
        Seeker { buffer: Vec::new() }
    }

    pub fn write(&mut self, data: u8) {
        self.buffer.push(data);
    }
}

impl Default for Seeker {
    fn default() -> Self {
        Self::new()
    }
}

impl Read for Seeker {
    fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
        for (i, byte) in (&self.buffer).iter().enumerate() {
            if *byte == 0 {
                return Ok(i);
            }
            buf[i] = *byte;
        }
        Ok(0)
    }
}
