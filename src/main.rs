use std::process;
use std::time::Instant;

mod decoder;
mod decryptor;
mod sinker;

fn main() {
    let timer = Instant::now();

    let unwx = cli::Unwx::from_env_or_exit();
    if let Err(e) = unwx.run() {
        eprintln!("{:#}", e);
        eprintln!("Failed in {:?}", timer.elapsed());
        process::exit(1);
    }
    eprintln!("Done in {:?}", timer.elapsed());
}

mod cli {
    use std::{
        fs,
        path::{Path, PathBuf},
    };

    use anyhow::Context;

    use crate::{
        decoder::Decoder,
        decryptor::{self, DecryptorBuilder},
        sinker::Sinker,
    };

    xflags::xflags! {
        /// unpack wx pkg
        cmd unwx {
            /// output directory for unpacked files
            optional -o,--output output: String
            /// clean output directory before write
            optional -c,--clean
            /// wxid of the package
            optional -w, --wxid wxid: String
            /// path to the package
            required input: String
        }
    }

    impl Unwx {
        fn input(&self) -> anyhow::Result<PathBuf> {
            PathBuf::from(&self.input)
                .canonicalize()
                .context("Failed to canonicalize input path")
        }

        fn output(&self) -> PathBuf {
            self.output
                .as_ref()
                .map(PathBuf::from)
                .unwrap_or_else(|| Path::new(&self.input).with_extension("unpacked"))
        }

        pub fn run(&self) -> anyhow::Result<()> {
            let sinker = self.get_sinker()?;

            let data = self.get_input_data()?;
            let decoder = Decoder::new(&data).context("Failed to initialize decoder")?;

            rayon::scope(|scope| {
                let sinker = &sinker;

                for file in decoder {
                    let file = file.context("Failed to decode file")?;

                    scope.spawn(move |_| {
                        if let Err(e) = sinker.write_file(file.name, file.data) {
                            eprintln!("Failed to write file {}\n{:#}", file.name, e);
                        }
                    });
                }

                anyhow::Ok(())
            })?;

            Ok(())
        }

        fn get_sinker(&self) -> anyhow::Result<Sinker> {
            let output = self.output();
            let sinker = Sinker::new(output);
            if self.clean {
                sinker.clean().context("Failed to clean output directory")?;
            }
            Ok(sinker)
        }

        fn get_input_data(&self) -> anyhow::Result<Vec<u8>> {
            let input = self.input()?;
            let data = fs::read(&input).context("Failed to read input package")?;

            if decryptor::should_decrypt(&data) {
                DecryptorBuilder::new()
                    .guess_wxid_from_path(&input)
                    .set_wxid(self.wxid.clone())
                    .build()
                    .ok_or_else(|| {
                        anyhow::anyhow!(
                            "can not infer wxid from path, should set wxid manually by -w, --wxid"
                        )
                    })?
                    .decrypt(&data)
                    .context("Failed to decrypt data")
            } else {
                Ok(data)
            }
        }
    }
}
