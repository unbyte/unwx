use std::io::{self, Cursor, Read};
use std::str::from_utf8_unchecked;

#[derive(Debug)]
pub struct DecodedFile<'a> {
    pub name: &'a str,
    pub data: &'a [u8],
}

#[derive(Debug)]
pub struct Decoder<'a> {
    reader: Cursor<&'a Vec<u8>>,
    remaining_files: u32,
}

impl<'a> Decoder<'a> {
    pub fn new(buf: &'a Vec<u8>) -> io::Result<Self> {
        let mut decoder = Self {
            reader: Cursor::new(buf),
            remaining_files: 0,
        };

        if decoder.read_u8()? != 0xBE {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                "Invalid first mark",
            ));
        }

        decoder.read_u32()?; // file_info_offset
        decoder.read_u32()?; // index_info_length
        decoder.read_u32()?; // body_info_length

        if decoder.read_u8()? != 0xED {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                "Invalid last mark",
            ));
        }

        decoder.remaining_files = decoder.read_u32()?;

        Ok(decoder)
    }

    fn read_u8(&mut self) -> io::Result<u8> {
        let mut buf = [0; 1];
        self.reader.read_exact(&mut buf)?;
        Ok(u8::from_be_bytes(buf))
    }

    fn read_u32(&mut self) -> io::Result<u32> {
        let mut buf = [0; 4];
        self.reader.read_exact(&mut buf)?;
        Ok(u32::from_be_bytes(buf))
    }

    fn read_bytes(&mut self, len: usize) -> io::Result<&'a [u8]> {
        let start = self.reader.position();
        let bytes = self.get_data(start as usize, len)?;
        self.reader.set_position(start + len as u64);
        Ok(bytes)
    }

    fn read_string(&mut self, len: usize) -> io::Result<&'a str> {
        Ok(unsafe { from_utf8_unchecked(self.read_bytes(len)?) })
    }

    fn get_data(&mut self, offset: usize, len: usize) -> io::Result<&'a [u8]> {
        let buf = *self.reader.get_ref();
        let end = offset + len;
        if end > buf.len() {
            return Err(io::Error::new(
                io::ErrorKind::UnexpectedEof,
                "failed to fill whole buffer",
            ));
        }
        Ok(&buf[offset..end])
    }
}

impl<'a> Iterator for Decoder<'a> {
    type Item = io::Result<DecodedFile<'a>>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.remaining_files == 0 {
            return None;
        }

        self.remaining_files -= 1;

        Some((|| {
            let name_len = self.read_u32()? as usize;
            let name = self.read_string(name_len)?;
            let offset = self.read_u32()? as usize;
            let size = self.read_u32()? as usize;
            let data = self.get_data(offset, size)?;

            Ok(DecodedFile { name, data })
        })())
    }
}
