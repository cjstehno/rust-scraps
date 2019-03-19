use std::fs::File;
use std::io::{Read, Write};
use std::path::Path;

use zip::result::{ZipError, ZipResult};
use zip::write::FileOptions;
use zip::ZipWriter;

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
        .unix_permissions(0o755);

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
pub fn list_files() -> Option<Vec<Path>> {

}

#[cfg(test)]
mod tests {
    use std::fs::File;
    use std::path::Path;

    use tempfile::TempDir;

    use crate::create_zip_file;

    #[test]
    fn zip_single_file() {
        let temp_dir = TempDir::new().unwrap();
        let zip_path = temp_dir.path().join("single-file.zip");
        let zip_file = File::create(&zip_path).unwrap();

        let path = Path::new("rc/file-a.txt");

        create_zip_file(path, &zip_file).expect("Failed to create zip file!");

        assert_eq!(zip_path.is_file(), true);
        assert_eq!(zip_path.exists(), true);
        assert_eq!(zip_file.metadata().unwrap().len() > 0, true);

        // TODO: assert that it has the specified contents
    }

    #[test]
    fn zip_directory_of_files() {
        let temp_dir = TempDir::new().unwrap();
        let zip_path = temp_dir.path().join("multi-file.zip");
        let zip_file = File::create(&zip_path).unwrap();

        let path = Path::new("rc/");

        create_zip_file(path, &zip_file).expect("Failed to create zip file!");

        assert_eq!(zip_path.is_file(), true);
        assert_eq!(zip_path.exists(), true);
        assert_eq!(zip_file.metadata().unwrap().len() > 0, true);

        // TODO: assert that it has the specified contents
    }
}
