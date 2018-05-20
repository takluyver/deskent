extern crate clap;

use clap::{App, SubCommand};
use std::env;
use std::io;
use std::path::{Path, PathBuf};

fn find_application_dirs() -> io::Result<Vec<PathBuf>> {
    let data_home = match env::var_os("XDG_DATA_HOME") {
        Some(val) => {
            PathBuf::from(val)
        },
        None => {
            let home = env::home_dir().ok_or(io::Error::new(io::ErrorKind::Other, "Couldn't get home dir"))?;
            home.join(".local/share")
        }
    };
    let extra_data_dirs = match env::var_os("XDG_DATA_DIRS") {
        Some(val) => {
            env::split_paths(&val).map(PathBuf::from).collect()
        },
        None => {
            vec![PathBuf::from("/usr/local/share"),
                 PathBuf::from("/usr/share")]
        }
    };

    let mut res = Vec::new();
    res.push(data_home.join("applications"));
    for dir in extra_data_dirs {
        res.push(dir.join("applications"));
    }
    Ok(res)
}

fn ls_one_dir(path : &Path) -> io::Result<()> {
    println!("{}", path.to_string_lossy());
    if !path.is_dir() {
        println!("  (Not a directory)");
        return Ok(())
    }
    let mut filenames = path.read_dir()?
                       .filter_map(|v| v.ok())
                       .filter(|e| match e.file_type() {
                           Ok(ft) => ft.is_file(),
                           _ => false
                       })
                       .map(|e| e.file_name().to_string_lossy().into_owned())
                       .filter(|f| f.ends_with(".desktop"))
                       .collect::<Vec<_>>();
    if filenames.is_empty() {
        println!("  (No .desktop files found)");
        return Ok(())
    }
    filenames.sort_by_key(|f| f.to_lowercase());
    for filename in filenames {
        println!("  {}", filename);
    }
    Ok(())
}

fn ls() -> io::Result<()> {
    for appsdir in find_application_dirs()? {
        ls_one_dir(&appsdir)?;
    }
    Ok(())
}

fn main() {
    let version = env!("CARGO_PKG_VERSION");
    let matches = App::new("Deskent")
                    .version(version)
                    .author("Thomas Kluyver")
                    .about("Inspect desktop entry (.desktop) files.")
                    .subcommand(SubCommand::with_name("ls")
                                .about("List installed .desktop files.")
                               )
                    .get_matches();
    
    if matches.is_present("ls") {
        ls().unwrap();
    }
}
