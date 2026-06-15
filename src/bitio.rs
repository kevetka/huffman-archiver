// SPDX-License-Identifier: MIT

use std::{
    fs::File,
    io::{self, BufReader, BufWriter, Read, Write},
};

/// Потоковый побитовый писатель.
///
/// Буферизует биты и записывает полные байты во внутренний `BufWriter`.
/// Неполный байт сбрасывается методом `flush`.
pub struct BitWriter {
    pub buffer: u8,
    pub count: u8,
    pub writer: BufWriter<File>,
}

impl BitWriter {
    /// Создаёт нового `BitWriter`, обёрнутого вокруг переданного файла.
    pub fn new(file: File) -> BitWriter {
        BitWriter {
            buffer: 0,
            count: 0,
            writer: BufWriter::new(file),
        }
    }

    /// Записывает один бит (true = 1, false = 0).
    ///
    /// При накоплении 8 бит сбрасывает полный байт в выходной поток.
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

    /// Записывает целый байт.
    ///
    /// Если в буфере есть неполный байт, дописывает старшие биты `byte`
    /// к текущему буферу, затем выравнивает оставшиеся младшие биты.
    pub fn write_byte(&mut self, byte: u8) -> io::Result<()> {
        if self.count == 0 {
            self.writer.write_all(&[byte])?;
        } else {
            self.buffer <<= 8 - self.count;
            self.buffer |= byte >> self.count;

            self.writer.write_all(&[self.buffer])?;

            self.buffer = byte & ((1 << self.count) - 1);
        }

        Ok(())
    }

    /// Сбрасывает неполный байт (дополняя нулями) и вызывает `flush` у
    /// внутреннего `BufWriter`.
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

/// Потоковый побитовый читатель.
///
/// Читает байты из внутреннего `BufReader` и выдаёт биты по одному,
/// начиная со старшего (MSB).
pub struct BitReader {
    pub buffer: u8,
    pub count: u8,
    pub reader: BufReader<File>,
}

impl BitReader {
    /// Создаёт нового `BitReader`, обёрнутого вокруг переданного файла.
    pub fn new(file: File) -> BitReader {
        BitReader {
            buffer: 0,
            count: 0,
            reader: BufReader::new(file),
        }
    }

    /// Читает один бит.
    ///
    /// Возвращает `Some(true)` или `Some(false)` при успехе,
    /// `None` при достижении конца файла.
    pub fn read_bit(&mut self) -> io::Result<Option<bool>> {
        if self.count == 0 {
            let mut byte = [0u8; 1];
            match self.reader.read_exact(&mut byte) {
                Ok(_) => {
                    self.buffer = byte[0];
                    self.count = 8;
                }
                Err(e) if e.kind() == io::ErrorKind::UnexpectedEof => {
                    return Ok(None);
                }
                Err(e) => return Err(e),
            }
        }

        let bit = self.buffer & 128 != 0;
        self.buffer <<= 1;
        self.count -= 1;
        Ok(Some(bit))
    }

    /// Читает 8 бит и собирает их в байт (MSB first).
    ///
    /// Возвращает `None`, если во время чтения встретился конец файла.
    pub fn read_byte(&mut self) -> io::Result<Option<u8>> {
        let mut byte = 0u8;
        for _ in 0..8 {
            match self.read_bit()? {
                Some(bit) => {
                    byte = (byte << 1) | (bit as u8);
                }
                None => return Ok(None),
            }
        }
        Ok(Some(byte))
    }
}
