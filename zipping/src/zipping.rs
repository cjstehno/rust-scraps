use std::fs::{create_dir_all, File};
use std::io::{copy, Read, Write};
use std::path::Path;

use zip::{ZipArchive, ZipWriter};
use zip::result::{ZipError, ZipResult};
use zip::write::FileOptions;

////
/// Creates a compressed zip file from the path, which may be a single file or a directory. When
/// zipping a directory, the specified directory will be the top-level entry of the zip file.
///
/// # Arguments
///
/// * `path` - the path to the file or directory to be zipped
/// * `zip_file` - the file handle to the zip file being created
///
#[allow(dead_code)]
pub fn create_zip_file(path: &Path, zip_file: &File) -> ZipResult<()> {
    let mut writer = ZipWriter::new(zip_file);

    let options = FileOptions::default()
        .compression_method(zip::CompressionMethod::Deflated)
        .unix_permissions(0o777);

    if path.is_file() {
        zip_single(&mut writer, path, options)?;
    } else if path.is_dir() {
        zip_multiple(&mut writer, path, options)?;
    } else {
        // path is not a file or directory
        return Err(ZipError::FileNotFound);
    }

    writer.finish()?;

    Ok(())
}

fn zip_single(writer: &mut ZipWriter<&File>, path: &Path, options: FileOptions) -> ZipResult<()> {
    writer.start_file(path.file_name().unwrap().to_str().unwrap(), options)?;
    let mut file = File::open(path).unwrap();

    let mut buffer = vec![];
    file.read_to_end(&mut buffer)?;
    writer.write_all(&*buffer)?;
    buffer.clear();

    Ok(())
}

fn zip_multiple(writer: &mut ZipWriter<&File>, path: &Path, options: FileOptions) -> ZipResult<()> {
    let mut buffer = vec![];
    let mut directories = vec![path.to_path_buf()];

    while !directories.is_empty() {
        let directory = directories.pop().unwrap();
        writer.add_directory(directory.to_str().unwrap().replace("\\", "/"), options)?;

        for dir_entry in directory.read_dir().unwrap() {
            let entry = dir_entry.unwrap();
            let file_type = entry.file_type().unwrap();

            if file_type.is_dir() {
                directories.push(entry.path());
            } else if file_type.is_file() {
                writer.start_file(directory.join(entry.path().components().last().unwrap()).to_str().unwrap().replace("\\", "/"), options)?;

                let mut file = File::open(entry.path()).unwrap();
                file.read_to_end(&mut buffer)?;
                writer.write_all(&*buffer)?;
                buffer.clear();
            }
        }
    }

    Ok(())
}

////
/// Lists the entries in a given zip file (files and directories). Files will have information
/// about their actual and compressed size.
///
/// # Arguments
///
/// * `zip_file` - the file handle to the zip file being listed
///
#[allow(dead_code)]
pub fn list_zip_contents(zip_file: &File) -> ZipResult<Vec<String>> {
    let mut archive = ZipArchive::new(zip_file)?;

    (0..archive.len())
        .map(|idx| archive.by_index(idx).and_then(|zf| Ok(if zf.size() > 0 {
            format!("{} ({} bytes, {} compressed)", zf.name(), zf.size(), zf.compressed_size())
        } else {
            format!("{}", zf.name())
        })))
        .collect()
}

///
/// Unzips the specified zip file into the given directory path.
///
/// # Arguments
///
/// * `zip_file` - the zip file to be unzipped
/// * `out_path` - the path to the directory where the zip is to be unzipped.
///
#[allow(dead_code)]
pub fn unzip_file(zip_file: &File, out_path: &Path) -> ZipResult<()> {
    let mut archive = ZipArchive::new(zip_file)?;

    for i in 0..archive.len() {
        let mut file = archive.by_index(i).unwrap();
        let file_path = out_path.join(file.sanitized_name());

        if (&*file.name()).ends_with('/') {
            // create the directory entry
            create_dir_all(&file_path).unwrap();
        } else {
            // create any missing parent directories
            if let Some(p) = file_path.parent() {
                if !p.exists() {
                    create_dir_all(&p).unwrap();
                }
            }

            // create the file
            let mut out_file = File::create(&file_path).unwrap();
            copy(&mut file, &mut out_file).unwrap();
        }
    }

    Ok(())
}


#[cfg(test)]
mod tests {
    use core::borrow::Borrow;
    use std::fs::File;
    use std::path::Path;

    use hamcrest2::equal_to;
    use hamcrest2::matchers::compared_to::greater_than;
    use hamcrest2::prelude::*;
    use tempfile::TempDir;

    use crate::zipping::{create_zip_file, list_zip_contents, unzip_file};

    #[test]
    fn unzip_a_file() {
        // create a zip file to unzip
        let temp_dir = TempDir::new().unwrap();
        let zip_path = temp_dir.path().join("multi-file.zip");
        let zip_file = File::create(&zip_path).unwrap();
        let content_path = Path::new("rc/");

        create_zip_file(content_path, &zip_file).expect("Failed to create zip file!");
        assert_that!(zip_path.exists(), equal_to(true));

        // unzip the file
        let zipped_file = File::open(&zip_path).unwrap();
        let unzipped_path = temp_dir.path().join("unzipped/");

        unzip_file(&zipped_file, &unzipped_path).expect("Problem unzipping file!");

        // make sure the unzipped directory has the expected contents
        let unzipped_file_paths = list_files_recursive(&unzipped_path);
        assert_that!(unzipped_file_paths.len(), equal_to(5));

        assert_that!(unzipped_file_paths[0].ends_with("/rc/file-a.txt"), equal_to(true));
        assert_that!(unzipped_file_paths[1].ends_with("/rc/alpha/file-b.txt"), equal_to(true));
        assert_that!(unzipped_file_paths[2].ends_with("/rc/alpha/charlie/file-d.txt"), equal_to(true));
        assert_that!(unzipped_file_paths[3].ends_with("/rc/alpha/charlie/file-e.txt"), equal_to(true));
        assert_that!(unzipped_file_paths[4].ends_with("/rc/alpha/bravo/file-c.txt"), equal_to(true));
    }

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

    #[test]
    fn zip_single_file() {
        let temp_dir = TempDir::new().unwrap();
        let zip_path = temp_dir.path().join("single-file.zip");
        let zip_file = File::create(&zip_path).unwrap();

        let path = Path::new("rc/file-a.txt");

        create_zip_file(path, &zip_file).expect("Failed to create zip file!");

        assert_that!(zip_path.is_file(), equal_to(true));
        assert_that!(zip_path.exists(), equal_to(true));
        assert_that!(zip_path.metadata().unwrap().len(), greater_than(100));

        // make sure the file has the items we expect
        let listing_file = File::open(&zip_path).unwrap();
        let entries = list_zip_contents(&listing_file).expect("Unable to list zip contents!");

        assert_that!(&entries, len(1));
        assert_that!(&entries.contains("file-a.txt (12 bytes, 27 compressed)".to_string().borrow()), equal_to(true));
    }

    #[test]
    fn zip_directory_of_files() {
        let temp_dir = TempDir::new().unwrap();
        let zip_path = temp_dir.path().join("multi-file.zip");
        let zip_file = File::create(&zip_path).unwrap();

        let path = Path::new("rc/");

        create_zip_file(path, &zip_file).expect("Failed to create zip file!");

        assert_that!(zip_path.is_file(), equal_to(true));
        assert_that!(zip_path.exists(), equal_to(true));
        assert_that!(zip_path.metadata().unwrap().len(), greater_than(1000));

        // make sure the file has the items we expect
        let listing_file = File::open(&zip_path).unwrap();
        let entries = list_zip_contents(&listing_file).expect("Unable to list zip contents!");

        assert_that!(&entries, len(9));
        assert_that!(&entries.contains("rc/".to_string().borrow()), equal_to(true));
        assert_that!(&entries.contains("rc/file-a.txt (12 bytes, 27 compressed)".to_string().borrow()), equal_to(true));
        assert_that!(&entries.contains("rc/alpha/".to_string().borrow()), equal_to(true));
        assert_that!(&entries.contains("rc/alpha/file-b.txt (15 bytes, 28 compressed)".to_string().borrow()), equal_to(true));
        assert_that!(&entries.contains("rc/alpha/charlie/".to_string().borrow()), equal_to(true));
        assert_that!(&entries.contains("rc/alpha/charlie/file-d.txt (24 bytes, 34 compressed)".to_string().borrow()), equal_to(true));
        assert_that!(&entries.contains("rc/alpha/charlie/file-e.txt (35 bytes, 43 compressed)".to_string().borrow()), equal_to(true));
        assert_that!(&entries.contains("rc/alpha/bravo/".to_string().borrow()), equal_to(true));
        assert_that!(&entries.contains("rc/alpha/bravo/file-c.txt (22 bytes, 34 compressed)".to_string().borrow()), equal_to(true));
    }
}
