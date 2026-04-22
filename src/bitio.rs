use std::{
    fs::File,
    io::{self, BufWriter, Write},
};

pub struct BitWriter {
    pub buffer: u8,
    pub count: u8,
    pub writer: BufWriter<File>,
}

impl BitWriter {
    pub fn new(file: File) -> BitWriter {
        BitWriter {
            buffer: 0,
            count: 0,
            writer: BufWriter::new(file),
        }
    }

    pub fn write_bit(&mut self, bit: bool) -> io::Result<()> {
        self.buffer <<= 1;

        if bit {
            self.buffer |= 1;
        }
        self.count += 1;

        if self.count == 8 {
            self.writer.write_all(&[self.buffer])?;
            self.buffer = 0;
            self.count = 0;
        }

        Ok(())
    }

    pub fn write_byte(&mut self, byte: u8) -> io::Result<()> {
        if self.count == 0 {
            self.writer.write_all(&[byte])?;
        } else {
            self.buffer |= byte >> (8 - self.count);

            self.writer.write_all(&[self.buffer])?;

            self.buffer = byte << self.count;
        }

        Ok(())
    }

    pub fn flush(&mut self) -> io::Result<()> {
        if self.count > 0 {
            self.buffer <<= 8 - self.count;
            self.writer.write_all(&[self.buffer])?;

            self.buffer = 0;
            self.count = 0;
        }

        self.writer.flush()
    }
}
