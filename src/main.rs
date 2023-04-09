use glob::glob;
use rayon::prelude::*;
use std::fs;
use std::fs::File;
use std::io::prelude::*;
use std::path::PathBuf;
use std::time::Instant;
use zip::write::FileOptions;
use zip::ZipWriter;

// 定義一個結構來存放所有資料夾的路徑
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
    let start = Instant::now();

    let pattern = "D:/temp/*/"; // 加上 / 限制只匹配子資料夾
    let mut collect = Directories::new();
    for entry in glob(pattern).expect("Failed to read glob pattern") {
        match entry {
            Ok(path) => {
                collect.add(path); // 把找到的資料夾路徑存入 collect 結構
            }
            Err(e) => println!("{:?}", e),
        }
    }

    collect.pathes.into_par_iter().for_each(|path| {
        // 設定要產生的 zip 檔案路徑和檔名
        let zip_file_path = path.with_extension("zip");
        // 建立要壓縮的檔案的輸出串流
        let file = File::create(&zip_file_path).unwrap();
        let mut zip = ZipWriter::new(file); // 建立壓縮物件
        let options = FileOptions::default().compression_method(zip::CompressionMethod::Stored);

        // 使用 walkdir 來遍歷資料夾下的所有檔案，並進行壓縮
        walkdir::WalkDir::new(&path)
            .into_iter()
            .filter_map(|e| e.ok())
            .filter(|e| !e.path().is_dir()) // 只處理檔案
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
            if let Err(e) = fs::remove_dir_all(&path) {
                println!("Failed to remove directory: {:?}", e);
            } else {
                println!("Successfully compressed and removed directory: {:?}", path);
            }
        } else {
            println!("Failed to compress directory: {:?}", path);
        }
    });

    // 計算執行時間並輸出
    let duration = start.elapsed();
    println!("Total time elapsed: {:?}", duration);
}
