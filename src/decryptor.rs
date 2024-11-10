use std::{io, path::Path};

pub fn should_decrypt(data: &[u8]) -> bool {
    data[..6].eq(b"V1MMWX")
}

#[derive(Debug)]
pub struct DecryptorBuilder {
    wxid: Option<String>,
}

impl DecryptorBuilder {
    pub fn new() -> Self {
        Self { wxid: None }
    }

    pub fn set_wxid(mut self, wxid: Option<String>) -> Self {
        if let Some(wxid) = wxid {
            self.wxid = Some(wxid);
        }
        self
    }

    pub fn guess_wxid_from_path<P: AsRef<Path>>(mut self, path: P) -> Self {
        if let Some(wxid) = implement::wxid_from_path(path.as_ref()) {
            self.wxid = Some(wxid);
        }
        self
    }

    pub fn build(self) -> Option<Decryptor> {
        self.wxid.map(Decryptor::new)
    }
}

#[derive(Debug)]
pub struct Decryptor {
    wxid: String,
}

impl Decryptor {
    fn new(wxid: String) -> Self {
        Self { wxid }
    }

    pub fn decrypt(&self, data: &[u8]) -> io::Result<Vec<u8>> {
        implement::decrypt(&self.wxid, data)
    }
}

#[cfg(target_os = "windows")]
mod implement {
    use aes::{
        cipher::{BlockDecryptMut, KeyIvInit},
        Aes256, Block,
    };
    use pbkdf2::pbkdf2_hmac;
    use sha1::Sha1;
    use std::{io, path::Path};

    const SALT: &[u8] = b"saltiest";
    const IV: &[u8] = b"the iv: 16 bytes";

    fn get_xor_key(wxid: &str) -> u8 {
        if wxid.len() >= 2 {
            wxid.as_bytes()[wxid.len() - 2]
        } else {
            0x66
        }
    }

    fn decrypt_header(wxid: &str, data: &[u8]) -> Vec<u8> {
        let mut key = [0u8; 32];
        pbkdf2_hmac::<Sha1>(wxid.as_bytes(), SALT, 1000, &mut key);

        let mut cipher = cbc::Decryptor::<Aes256>::new(&key.into(), IV.into());
        let mut buf = data[6..1030].to_vec();
        buf.chunks_mut(16).for_each(|chunk| {
            let block = Block::from_mut_slice(chunk);
            cipher.decrypt_block_mut(block);
        });
        // drop the last(1024th) byte
        buf.pop();
        buf
    }

    // The structure of the encrypted package is as follows:
    // Bytes 0..6: V1MMWX
    // Bytes 6..1030: Correspond to the first 1023 bytes of the source file, which are AES encrypted
    // Remaining bytes: The rest of the content corresponds to the bytes of the source file XORed with the second-to-last character of wxid
    pub fn decrypt(wxid: &str, data: &[u8]) -> io::Result<Vec<u8>> {
        let mut result = decrypt_header(wxid, data);
        let xor_key = get_xor_key(wxid);
        result.extend(data[1030..].iter().map(|byte| *byte ^ xor_key));
        Ok(result)
    }

    pub fn wxid_from_path(path: &Path) -> Option<String> {
        let mut segments = path.iter();
        segments.find(|seg| matches!(seg.to_str(), Some("Applet")))?;
        segments
            .next()
            .map(|wxid_segment| wxid_segment.to_string_lossy().to_string())
    }
}

#[cfg(not(target_os = "windows"))]
mod implement {
    use std::{io, path::Path};

    pub fn decrypt(_wxid: &str, _data: &[u8]) -> io::Result<Vec<u8>> {
        unimplemented!("decrypt for unix platforms")
    }

    pub fn wxid_from_path(_path: &Path) -> Option<String> {
        unimplemented!("wxid_from_path for unix platforms")
    }
}
