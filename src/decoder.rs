use std::io::{self, Cursor, Read};
use std::str::from_utf8_unchecked;

#[derive(Debug)]
pub struct Decoder<'a> {
    reader: Cursor<&'a Vec<u8>>,
    sender: flume::Sender<(&'a str, &'a [u8])>,
}

impl<'a> Decoder<'a> {
    pub fn new(buf: &'a Vec<u8>, sender: flume::Sender<(&'a str, &'a [u8])>) -> Self {
        Self {
            reader: Cursor::new(buf),
            sender,
        }
    }

    pub fn start(mut self) -> io::Result<()> {
        let first_mark = self.read_u8()?;
        if first_mark != 0xBE {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                "Invalid first mark",
            ));
        }

        let _file_info_offset = self.read_u32()?;
        let _index_info_length = self.read_u32()?;
        let _body_info_length = self.read_u32()?;

        let last_mark = self.read_u8()?;
        if last_mark != 0xED {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                "Invalid last mark",
            ));
        }

        let file_count = self.read_u32()?;

        for _ in 0..file_count {
            let name_len = self.read_u32()? as usize;
            let name = self.read_string(name_len)?;
            let offset = self.read_u32()? as usize;
            let size = self.read_u32()? as usize;

            let data = self.get_data(offset, size)?;

            self.sender
                .send((name, data))
                .expect("Failed to send data to worker thread");
        }

        Ok(())
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
