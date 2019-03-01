extern crate clap;
extern crate dirs;
extern crate ini;

use clap::{App, Arg, SubCommand};
use ini::Ini;
use std::env;
use std::io;
use std::path::{Path, PathBuf};

fn find_application_dirs() -> io::Result<Vec<PathBuf>> {
    let data_home = match env::var_os("XDG_DATA_HOME") {
        Some(val) => {
            PathBuf::from(val)
        },
        None => {
            let home = dirs::home_dir().ok_or(io::Error::new(io::ErrorKind::Other, "Couldn't get home dir"))?;
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

fn get_dir_desktop_files(path: &Path) -> io::Result<Vec<std::fs::DirEntry>> {
    return Ok(path.read_dir()?
             .filter_map(|v| v.ok())
             .filter(|e| match e.file_type() {
                 Ok(ft) => (ft.is_file() | ft.is_symlink()),
                 _ => false
              })
             .filter(|e| e.file_name().to_string_lossy().ends_with(".desktop"))
             .collect::<Vec<_>>());
}

fn ls_one_dir(path : &Path) -> io::Result<()> {
    println!("{}", path.to_string_lossy());
    if !path.is_dir() {
        println!("  (Not a directory)");
        return Ok(())
    }
    let mut filenames = get_dir_desktop_files(&path)?.iter()
                        .map(|e| e.file_name().to_string_lossy().into_owned())
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

fn find(needle: &str) -> io::Result<()> {
    fn err_other(s: &str) -> io::Error {
        io::Error::new(io::ErrorKind::Other, s)
    }

    for appsdir in find_application_dirs()? {
        for dtfile in get_dir_desktop_files(&appsdir)? {
            let info = Ini::load_from_file(dtfile.path())
                       .map_err(|e| err_other(&e.to_string()))?;
            let sec = match info.section(Some("Desktop Entry")) {
                Some(s) => s,
                None => {return Err(err_other("No [Desktop Entry] section"));}
            };
            let name = match sec.get("Name") {
                Some(p) => p,
                None => {return Err(err_other("No Name key"));}
            };
            if name.to_lowercase().contains(needle) {
                println!("{}", dtfile.path().to_string_lossy());
                println!("  Name = {}", name);
            }
        }
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
                    .subcommand(SubCommand::with_name("find")
                                .about("Find a desktop file by application name")
                                .arg(Arg::with_name("pattern")
                                     .help("The name to search for")
                                     .required(true)
                                )
                               )
                    .get_matches();
    if let Some(matches) = matches.subcommand_matches("find") {
        find(matches.value_of("pattern").unwrap()).unwrap();
    } else if matches.is_present("ls") {
        ls().unwrap();
    }
}
