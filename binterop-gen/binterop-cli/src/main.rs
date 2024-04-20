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
        if path
            .as_os_str()
            .to_str()
            .map(|path_str| path_str.starts_with("--"))
            .unwrap_or_default()
        {
            continue;
        }

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
                if let Err(err) = process_text(&path, &file_text, &args) {
                    eprintln!("\t{err}")
                }
            }
            Err(err) => eprintln!("{err:?}"),
        }

        println!()
    }
}
