use std::fs::File;
use std::io::Write;
use std::path::PathBuf;
use std::{fs, io};

#[derive(Debug)]
pub struct Sinker {
    target_dir: PathBuf,
}

impl Sinker {
    pub fn new<P: Into<PathBuf>>(target: P) -> Self {
        Self {
            target_dir: target.into(),
        }
    }

    pub fn write_file(&self, filename: &str, data: &[u8]) -> io::Result<()> {
        let path = self
            .target_dir
            .join(filename.strip_prefix('/').unwrap_or(filename));

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
