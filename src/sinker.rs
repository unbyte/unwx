use std::fs::File;
use std::io::Write;
use std::path::PathBuf;
use std::{fs, io};

#[derive(Debug)]
pub struct Sinker<'a> {
    target_dir: PathBuf,
    recv: flume::Receiver<(&'a str, &'a [u8])>,
}

impl<'a> Sinker<'a> {
    pub fn new<P: Into<PathBuf>>(target: P, recv: flume::Receiver<(&'a str, &'a [u8])>) -> Self {
        Self {
            target_dir: target.into(),
            recv,
        }
    }

    pub fn start(self) {
        while let Ok((filename, data)) = self.recv.recv() {
            if let Err(e) = self.write_file(filename, data) {
                eprintln!("warning: write failed {}: {:?}", filename, e);
            }
        }
    }

    fn write_file(&self, filename: &str, data: &[u8]) -> io::Result<()> {
        let mut path = self.target_dir.clone();
        path.push(filename.strip_prefix('/').unwrap_or(filename));

        let mut file = match File::create(&path) {
            Ok(file) => file,
            Err(e) if e.kind() == io::ErrorKind::NotFound => {
                if let Some(parent) = path.parent() {
                    fs::create_dir_all(parent)?;
                    File::create(&path)?
                } else {
                    return Err(e);
                }
            }
            Err(e) => return Err(e),
        };

        file.write_all(data)
    }
}
