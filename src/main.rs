use crate::decoder::Decoder;
use crate::sinker::Sinker;
use std::time::Instant;
use std::{fs, io};

mod decoder;
mod sinker;

mod flags {
    xflags::xflags! {
        /// unpack wx pkg
        cmd unwx {
            /// output directory for unpacked files
            optional -o,--output output: String
            /// clean output directory before write
            optional -c,--clean
            /// path to the package
            required input: String
        }
    }
}

fn main() -> io::Result<()> {
    let cmd = flags::Unwx::from_env_or_exit();

    let input = cmd.input;
    let output = cmd.output.unwrap_or(format!("{input}.unpacked"));

    if cmd.clean {
        match fs::remove_dir_all(&output) {
            Ok(_) => (),
            Err(e) if e.kind() != io::ErrorKind::NotFound => {
                panic!("failed to clean output directory: {:?}", e);
            }
            _ => (),
        }
    }

    let timer = Instant::now();

    let data = fs::read(&input).expect("error when read input package");

    let decoder = Decoder::new(&data).expect("error when initializing decoder");
    let sinker = Sinker::new(output);

    rayon::scope(|scope| {
        let sinker = &sinker;
        for file in decoder {
            let file = file.expect("error when decoding file");

            scope.spawn(move |_| {
                if let Err(e) = sinker.write_file(file.name, file.data) {
                    eprintln!("warning: write failed {}: {:?}", file.name, e);
                }
            });
        }
    });

    eprintln!("done in {:?}", timer.elapsed());
    Ok(())
}
