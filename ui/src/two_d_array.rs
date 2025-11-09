use nwn_lib::files::two_da::DataTable;
use std::{
    fs::File,
    io::BufReader,
    path::{Path, PathBuf},
};

use crate::error::Error;

type ZipArchive = zip::ZipArchive<BufReader<File>>;

#[derive(Debug)]
struct Zip {
    archive: ZipArchive,
    name: String,
}

#[derive(Debug)]
pub struct FileReader2DA {
    file: Zip,
}
impl FileReader2DA {
    pub fn new(game_dir: &Path) -> Result<Self, Error> {
        let data_path = game_dir.join("data");

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

        Ok(Self { file })
    }

    pub fn read(&mut self, file_name: &str) -> Result<DataTable, Error> {
        let path = format!("{}/{}", self.file.name, file_name);
        let entry = self.file.archive.by_path(&path).unwrap();

        nwn_lib::files::two_da::parse(entry).map_err(Error::LibError)
    }
}
