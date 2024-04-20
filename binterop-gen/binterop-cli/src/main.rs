use backend::helpers::process_text;
use std::path::PathBuf;
use std::{env, fs};

fn main() {
    let args = env::args()
        .skip(1)
        .filter(|arg| !arg.starts_with("--"))
        .collect::<Vec<_>>();

    if args.is_empty() {
        eprintln!("No arguments were provided!");
        return;
    }

    for path in args.iter().map(PathBuf::from) {
        println!("{path:?}");
        let path = match fs::canonicalize(path) {
            Ok(path) => path,
            Err(err) => {
                eprintln!("\tFailed to canonicalize path! Error: {err:?}");
                continue;
            }
        };
        println!("{path:?}");

        match fs::read_to_string(&path) {
            Ok(file_text) => {
                if process_text(&path, &file_text).is_ok() {
                } else if let Err(err) = process_text(&path, &file_text) {
                    eprintln!("\t{err}")
                }
            }
            Err(err) => eprintln!("{err:?}"),
        }

        println!()
    }
}
