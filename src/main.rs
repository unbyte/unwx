use crate::decoder::Decoder;
use crate::sinker::Sinker;
use std::time::Instant;
use std::{fs, io, thread};

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
        fs::remove_dir_all(&output).expect("error when clean output dir");
    }

    let timer = Instant::now();
    let data = fs::read(&input).expect("error when read input package");
    thread::scope(|scope| {
        let (sender, recv) = flume::unbounded();
        scope.spawn(move || Sinker::new(output, recv).start());
        Decoder::new(&data, sender)
            .start()
            .expect("error when decode input package");
    });
    eprintln!("done in {:?}", timer.elapsed());
    Ok(())
}
