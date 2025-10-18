use cfg_if::cfg_if;
use nwn_lib::files::two_da::DataTable;
use std::{collections::HashMap, fs::File, io::BufReader, path::PathBuf};

use crate::error::Error;

fn find_steamapps_path() -> PathBuf {
    fn replace_home(home: &str, s: &str) -> String {
        s.replace("~", home)
    }

    let possible_directories = {
        cfg_if! {
            if #[cfg(target_os = "windows")] {
                [
                    r"C:\Program Files (x86)\Steam\steamapps",
                    r"C:\Program Files\Steam\steamapps",
                    r"D:\Program Files (x86)\Steam\steamapps",
                    r"D:\Program Files\Steam\steamapps",
                ]
                    .map(PathBuf::from)
            } else if #[cfg(target_os = "macos")] {
                let home = std::env::var("HOME")
                    .expect("Missing HOME env var");

                [
                    r"~/Library/Application Support/Steam/steamapps",
                    r"~/Library/Application Support/Steam/SteamApps",
                ]
                .map(|s| replace_home(&home, s))
                .map(PathBuf::from)
            } else if #[cfg(target_os = "linux")] {
                let home = std::env::var("HOME")
                    .expect("Missing HOME env var");

                [
                     "~/.steam/steam/steamapps",
                     "~/.local/share/Steam/steamapps",
                     "~/.var/app/com.valvesoftware.Steam/.local/share/Steam/steamapps",
                     "~/snap/steam/common/.local/share/Steam/steamapps",
                ]
                .map(|s| replace_home(&home, s))
                .map(PathBuf::from)
            } else {
                compile_error!("target os not supported")
            }
        }
    };

    possible_directories
        .into_iter()
        .find(|path| path.exists())
        .expect("Could not find steam path")
}

type ZipArchive = zip::ZipArchive<BufReader<File>>;

#[derive(Debug)]
struct Zip {
    archive: ZipArchive,
    name: String,
}

#[derive(Debug)]
pub struct FileReader {
    file: Zip,
    cache: HashMap<String, DataTable>,
}
impl FileReader {
    pub fn new() -> Result<Self, Error> {
        let steamapps = find_steamapps_path();
        let data_path = steamapps
            .join("common")
            .join("NWN2 Enhanced Edition")
            .join("data");

        if !data_path.exists() {
            return Err(Error::MissingGamePath(data_path));
        }

        fn open_zip(path: PathBuf) -> Result<Zip, Error> {
            let f = File::open(&path)?;
            let reader = BufReader::new(f);

            let zip = zip::ZipArchive::new(reader)
                .unwrap_or_else(|_| panic!("Failed to read zip file: {}", path.display()));

            let name = path
                .file_stem()
                .expect("Failed to get file name")
                .to_string_lossy()
                .to_ascii_uppercase();

            Ok(Zip { name, archive: zip })
        }

        let file = open_zip(data_path.join("2da.zip"))?;

        Ok(Self {
            file,
            cache: HashMap::new(),
        })
    }

    pub fn read(&mut self, file_name: &str) -> Result<&DataTable, Error> {
        if self.cache.contains_key(file_name) {
            Ok(self.cache.get(file_name).unwrap())
        } else {
            let path = format!("{}/{}", self.file.name, file_name);
            let entry = self.file.archive.by_path(&path).unwrap();

            let table = nwn_lib::files::two_da::parse(entry)?;

            self.cache.insert(file_name.to_string(), table);
            Ok(self.cache.get(file_name).unwrap())
        }
    }
}
