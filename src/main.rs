use glob::glob;
use rayon::prelude::*;
use std::fs;
use std::fs::File;
use std::io::prelude::*;
use std::path::PathBuf;
use std::time::Instant;
use zip::read::ZipArchive;
use zip::write::FileOptions;
use zip::ZipWriter;

// 定義一個結構來存放所有資料夾的路徑
struct Directories {
    images_directories: Vec<PathBuf>,
    zip_files_directories: Vec<PathBuf>,
}

impl Directories {
    fn new() -> Self {
        Self {
            images_directories: vec![],
            zip_files_directories: vec![],
        }
    }
    fn add(&mut self, path_str: PathBuf) {
        if !contains_zip_file(&path_str) {
            self.images_directories.push(path_str);
        } else {
            self.zip_files_directories.push(path_str);
        }
    }
}

fn main() {
    let start = Instant::now();

    // unzip_all_zipfiles();

    zip_all_directories();

    // 計算執行時間並輸出
    let duration = start.elapsed();
    println!("Total time elapsed: {:?}", duration);
}

fn zip_all_directories() {
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

    // handle images_directories
    println!(
        "\nimages_directories len : {}\n",
        collect.images_directories.len()
    );
    collect.images_directories.into_par_iter().for_each(|path| {
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

    // handle zip_files_directories
    println!(
        "\nzip_files_directories len : {}\n",
        collect.zip_files_directories.len()
    );
    collect.zip_files_directories.into_par_iter().for_each({
        |zip_file_directory|
        // 取得 zip_file_directory 裡面的 PathBuf
        match get_first_file_in_directory(&zip_file_directory) {
            // 檢查是否只有一個 zip file
            Some(zip_file) => {
                // Do something with zip_file
                println!("renaming zip_file: {:?}", zip_file);

                // rename zip file
                fs::rename(&zip_file, zip_file_directory.with_extension("zip"))
                    .expect("rename fail");

                // remove 此 zip_file_directory
                fs::remove_dir_all(&zip_file_directory).expect("remove dir fail");

                println!("rename succeed with {:?}", zip_file);
            }
            None => {
                // Handle case where directory is empty
                println!("Directory is empty");
            }
        }
    })
}

fn unzip_all_zipfiles() {
    let pattern = "D:/temp/*.zip";
    let mut zip_files = vec![];

    // 讀取所有符合 pattern 的 zip 檔案
    for entry in glob::glob(pattern).expect("Failed to read glob pattern") {
        if let Ok(path) = entry {
            zip_files.push(path);
        }
    }

    // 使用 rayon 並行處理每個 zip 檔案
    zip_files.into_par_iter().for_each(|zip_path| {
        println!("Unzipping file {:?}", zip_path);

        // 解壓縮至與 zip 檔案同名的資料夾
        let unzip_dir = zip_path.with_extension("");
        let mut zip_archive = ZipArchive::new(fs::File::open(&zip_path).unwrap()).unwrap();

        fs::create_dir(&unzip_dir).unwrap();

        for i in 0..zip_archive.len() {
            let mut file = zip_archive.by_index(i).unwrap();
            let mut buffer = Vec::new();
            file.read_to_end(&mut buffer).unwrap();

            let file_path = unzip_dir.join(file.name());
            if file.name().ends_with('/') {
                fs::create_dir_all(&file_path).unwrap();
            } else {
                fs::create_dir_all(file_path.parent().unwrap()).unwrap();
                fs::File::create(&file_path)
                    .unwrap()
                    .write_all(&buffer)
                    .unwrap();
            }
        }

        // 解壓縮完成後，移除原始 zip 檔案
        fs::remove_file(&zip_path).unwrap();

        println!("Unzipping file {:?} completed.", zip_path);
    });
}

fn contains_zip_file(path: &PathBuf) -> bool {
    if let Ok(entries) = std::fs::read_dir(path) {
        for entry in entries {
            if let Ok(entry) = entry {
                if let Some(extension) = entry.path().extension() {
                    if extension == "zip" {
                        return true;
                    }
                }
            }
        }
    }
    false
}

fn get_first_file_in_directory(path: &PathBuf) -> Option<PathBuf> {
    if let Ok(entries) = fs::read_dir(path) {
        for entry in entries {
            if let Ok(entry) = entry {
                let file_path = entry.path();
                if file_path.is_file() {
                    return Some(file_path);
                }
            }
        }
    }
    None
}
