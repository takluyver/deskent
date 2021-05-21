extern crate clap;
extern crate dirs;
extern crate ini;

use clap::{App, AppSettings, Arg, SubCommand};
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
    match path.read_dir() {
        Ok(readdir) => {
            Ok(
                readdir
                .filter_map(|v| v.ok())
                .filter(|e| match e.file_type() {
                  Ok(ft) => (ft.is_file() | ft.is_symlink()),
                  _ => false
                })
                .filter(|e| e.file_name().to_string_lossy().ends_with(".desktop"))
                .collect::<Vec<_>>()
            )
        }
        Err(e) => {
            if e.kind() == io::ErrorKind::NotFound {
                Ok(Vec::new())
            } else {
                Err(e)
            }
        }
    }
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
        let files = match get_dir_desktop_files(&appsdir) {
            Ok(v) => v,
            Err(e) => {
                println!("Could not list {}: {}", &appsdir.to_string_lossy(), e);
                continue;
            }
        };
        for dtfile in files {
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

fn main() -> io::Result<()> {
    let version = env!("CARGO_PKG_VERSION");
    let matches = App::new("Deskent")
        .version(version)
        .author("Thomas Kluyver")
        .about("Inspect desktop entry (.desktop) files.")
        .setting(AppSettings::SubcommandRequiredElseHelp)
        .subcommand(
            SubCommand::with_name("ls")
            .about("List installed .desktop files.")
        )
        .subcommand(
            SubCommand::with_name("find")
            .about("Find a desktop file by application name")
            .arg(Arg::with_name("pattern")
                .help("The name to search for")
                .required(true)
            )
        )
        .get_matches();
    match matches.subcommand() {
        ("find", Some(matches)) => find(matches.value_of("pattern").unwrap()),
        ("ls", _) => ls(),
        _ => {
            unreachable!();
        }
    }
}
