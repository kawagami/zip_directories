use std::fs;
use std::fs::File;
use std::io::prelude::*;
use std::path::PathBuf;
use zip::write::FileOptions;
use zip::ZipWriter;

use glob::glob;
use rayon::prelude::*;

struct Directories {
    pathes: Vec<PathBuf>,
}

impl Directories {
    fn new() -> Self {
        Self { pathes: vec![] }
    }
    fn add(&mut self, path_str: PathBuf) {
        self.pathes.push(path_str);
    }
}

fn main() {
    let pattern = "D:/temp/*/"; // 加上 / 限制只匹配子資料夾
    let mut collect = Directories::new();
    for entry in glob(pattern).expect("Failed to read glob pattern") {
        match entry {
            Ok(path) => {
                // 在这里把 path 加入 collect
                collect.add(path);
            }
            Err(e) => println!("{:?}", e),
        }
    }

    collect.pathes.into_par_iter().for_each(|path| {
        let zip_file_path = path.with_extension("zip");
        let file = File::create(&zip_file_path).unwrap();
        let mut zip = ZipWriter::new(file);
        let options = FileOptions::default().compression_method(zip::CompressionMethod::Stored);

        walkdir::WalkDir::new(&path)
            .into_iter()
            .filter_map(|e| e.ok())
            .filter(|e| !e.path().is_dir())
            .for_each(|e| {
                let file_path = e.path().strip_prefix(&path).unwrap();
                zip.start_file_from_path(file_path, options).unwrap();

                let mut file = File::open(e.path()).unwrap();
                let mut buffer = Vec::new();
                file.read_to_end(&mut buffer).unwrap();
                zip.write_all(&*buffer).unwrap();
            });

        // 壓縮成功後，移除原始資料夾
        if zip.finish().is_ok() {
            if let Err(e) = fs::remove_dir_all(path) {
                println!("Failed to remove directory: {:?}", e);
            }
        }
    });
}
