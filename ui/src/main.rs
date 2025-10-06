mod error;
mod ids;

use crate::error::Error;
use nwn_lib::files::gff::Gff;
use std::{fs::File, io::Read, path::Path};

fn open_file() -> Result<Gff, Error> {
    let mut args = std::env::args().skip(1);
    let file_path = args.next().expect("missing file arg");
    let ext = Path::new(&file_path).extension().and_then(|x| x.to_str());

    match ext {
        Some("zip") => {
            let file = File::open(file_path).unwrap();
            let mut reader = zip::read::ZipArchive::new(file).unwrap();
            let save = {
                let mut save = reader
                    .by_name("playerlist.ifo")
                    .expect("missing playerlist.ifo");
                let mut buf = Vec::with_capacity(save.size() as usize);
                save.read_to_end(&mut buf).unwrap();
                std::io::Cursor::new(buf)
            };

            Gff::read_without_tlk(save).map_err(|e| e.into())
        }
        Some("ifo") => {
            let file = File::open(&file_path).unwrap();
            Gff::read_without_tlk(file).map_err(|e| e.into())
        }

        Some(e) => panic!("unexpected file ext: {e}"),
        None => panic!("unknown file type"),
    }
}

fn main() {
    let save = open_file().unwrap();

    let x = save.root.bfs_iter()
        .find(|x| x.label == "KnownList1");
    println!("{x:#?}");
}
