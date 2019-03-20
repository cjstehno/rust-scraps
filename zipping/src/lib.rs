#[cfg(test)]
#[macro_use]
extern crate hamcrest2;

use std::path::Path;

mod zipping;
mod taring;

/// Shared helper function to recursively list files in a directory
#[allow(dead_code)]
fn list_files_recursive(path: &Path) -> Vec<String> {
    let mut file_paths = vec![];
    let mut directories = vec![path.to_path_buf()];

    while !directories.is_empty() {
        for dir_entry in directories.pop().unwrap().read_dir().unwrap() {
            let entry = dir_entry.unwrap();
            let file_type = entry.file_type().unwrap();
            let entry_path = entry.path();

            if file_type.is_dir() {
                directories.push(entry_path);
            } else if file_type.is_file() {
                file_paths.push(entry_path.to_str().unwrap().to_string().replace("\\", "/"));
            }
        }
    }

    file_paths
}