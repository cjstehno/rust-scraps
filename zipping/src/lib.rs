#[cfg(test)]
#[macro_use]
extern crate hamcrest2;

use core::borrow::Borrow;
use std::fs::File;
use std::io::{Read, Write};
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
        writer.add_directory(directory.to_str().unwrap(), options)?;

        for dir_entry in directory.read_dir().unwrap() {
            let entry = dir_entry.unwrap();
            let file_type = entry.file_type().unwrap();

            if file_type.is_dir() {
                directories.push(entry.path());
            } else if file_type.is_file() {
                writer.start_file(directory.join(entry.file_name().to_str().unwrap()).to_str().unwrap(), options)?;

                let mut file = File::open(entry.path()).unwrap();
                file.read_to_end(&mut buffer)?;
                writer.write_all(&*buffer)?;
                buffer.clear();
            }
        }
    }

    Ok(())
}

/// FIXME: document
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

#[cfg(test)]
mod tests {
    use core::borrow::Borrow;
    use std::fs::File;
    use std::path::Path;

    use hamcrest2::equal_to;
    use hamcrest2::matchers::compared_to::greater_than;
    use hamcrest2::prelude::*;
    use tempfile::TempDir;

    use crate::{create_zip_file, list_zip_contents};

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
        println!("entries: {:?}", &entries);

        assert_that!(&entries, len(9));
        assert_that!(&entries.contains("file-a.txt (12 bytes, 27 compressed)".to_string().borrow()), equal_to(true));
    }
}
