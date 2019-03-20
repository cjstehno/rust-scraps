use std::fs::{copy, create_dir_all, File};
use std::path::Path;

use tar::{Archive, Builder};

// FIXME: this example needs better error handling
// TODO: note that this does not compress the tar (just tar not gz)

////
/// Creates a tar file from the path, which may be a single file or a directory.
///
/// # Arguments
///
/// * `path` - the path to the file or directory to be archived
/// * `tar_file` - the file handle to the tar file being created
///
#[allow(dead_code)]
pub fn create_tar_file(path: &Path, tar_file: &File) -> Result<(), ()> {
    let mut tar_builder = Builder::new(tar_file);

    if path.is_file() {
        tar_builder.append_path(path).expect("Unable to append file.");
    } else if path.is_dir() {
        let mut directories = vec![path.to_path_buf()];

        while !directories.is_empty() {
            let directory = directories.pop().unwrap();

            for dir_entry in directory.read_dir().unwrap() {
                let entry = dir_entry.unwrap();
                let file_type = entry.file_type().unwrap();

                if file_type.is_dir() {
                    directories.push(entry.path());
                } else if file_type.is_file() {
                    tar_builder.append_path(entry.path()).unwrap();
                }
            }
        }
    } else {
        // path is not a file or directory
        return Err(());
    }

    match tar_builder.finish() {
        Ok(_) => Ok(()),
        Err(_) => Err(())
    }
}

////
/// Lists the entries in a given tar file (files and directories).
///
/// # Arguments
///
/// * `tar_file` - the file handle to the zip file being listed
///
#[allow(dead_code)]
pub fn list_tar_contents(tar_file: &File) -> Result<Vec<String>, ()> {
    let mut archive = Archive::new(tar_file);

    let mut contents = vec![];

    for file in archive.entries().unwrap() {
        let file = file.unwrap();
        contents.push(format!("{} ({:?} bytes)", file.header().path().unwrap().to_str().unwrap(), file.header().size().unwrap()));
    }

    Ok(contents)
}

///
/// Un-tars the specified tar file into the given directory path.
///
/// # Arguments
///
/// * `tar_file` - the tar file to be unzipped
/// * `out_path` - the path to the directory where the tar is to be un-archived.
///
#[allow(dead_code)]
pub fn untar_file(tar_file: &File, out_path: &Path) -> Result<(), ()> {
    let mut archive = Archive::new(tar_file);

    for file in archive.entries().unwrap() {
        let file = file.unwrap();
        let entry_path = file.path().unwrap();
        let mut file_path = out_path.join(&entry_path);

        if let Some(p) = file_path.parent() {
            if !p.exists() {
                create_dir_all(&p).unwrap();
            }
        }

//        let mut out_file = File::create(&file_path).unwrap();
        copy(&entry_path, &mut file_path).unwrap();
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use std::borrow::Borrow;
    use std::fs::File;
    use std::path::Path;

    use hamcrest2::equal_to;
    use hamcrest2::matchers::compared_to::greater_than;
    use hamcrest2::prelude::*;
    use tempfile::TempDir;

    use crate::list_files_recursive;
    use crate::taring::{create_tar_file, list_tar_contents, untar_file};

    #[test]
    fn create_single_file_tar() {
        let temp_dir = TempDir::new().unwrap();
        let tar_path = temp_dir.path().join("single-file.tar");
        let tar_file = File::create(&tar_path).unwrap();

        let path = Path::new("rc/file-a.txt");

        create_tar_file(path, &tar_file).expect("Problem creating tar file.");

        assert_that!(tar_path.is_file(), equal_to(true));
        assert_that!(tar_path.exists(), equal_to(true));
        assert_that!(tar_path.metadata().unwrap().len(), greater_than(100));

        // make sure the file has the items we expect
        let listing_file = File::open(&tar_path).unwrap();
        let entries = list_tar_contents(&listing_file).expect("Unable to list tar contents");

        println!("entry: {:?}", entries[0]);

        assert_that!(&entries, len(1));
        assert_that!(&entries.contains("rc/file-a.txt (12 bytes)".to_string().borrow()), equal_to(true));
    }

    #[test]
    fn tar_directory_of_files() {
        let temp_dir = TempDir::new().unwrap();
        let tar_path = temp_dir.path().join("multi-file.tar");
        let tar_file = File::create(&tar_path).unwrap();

        let path = Path::new("rc/");

        create_tar_file(path, &tar_file).expect("Failed to create tar file!");

        assert_that!(tar_path.is_file(), equal_to(true));
        assert_that!(tar_path.exists(), equal_to(true));
        assert_that!(tar_path.metadata().unwrap().len(), greater_than(1000));

        // make sure the file has the items we expect
        let listing_file = File::open(&tar_path).unwrap();
        let entries = list_tar_contents(&listing_file).expect("Unable to list tar contents!");

        assert_that!(&entries, len(5));
        assert_that!(&entries.contains("rc/file-a.txt (12 bytes)".to_string().borrow()), equal_to(true));
        assert_that!(&entries.contains("rc/alpha/file-b.txt (15 bytes)".to_string().borrow()), equal_to(true));
        assert_that!(&entries.contains("rc/alpha/charlie/file-d.txt (24 bytes)".to_string().borrow()), equal_to(true));
        assert_that!(&entries.contains("rc/alpha/charlie/file-e.txt (35 bytes)".to_string().borrow()), equal_to(true));
        assert_that!(&entries.contains("rc/alpha/bravo/file-c.txt (22 bytes)".to_string().borrow()), equal_to(true));
    }

    #[test]
    fn untar_a_file() {
        // create a zip file to unzip
        let temp_dir = TempDir::new().unwrap();
        let tar_path = temp_dir.path().join("multi-file.tar");
        let tar_file = File::create(&tar_path).unwrap();
        let content_path = Path::new("rc/");

        create_tar_file(content_path, &tar_file).expect("Failed to create tar file!");
        assert_that!(tar_path.exists(), equal_to(true));

        // unzip the file
        let tared_file = File::open(&tar_path).unwrap();
        let unarch_path = temp_dir.path().join("exploded/");

        untar_file(&tared_file, &unarch_path).expect("Problem un-taring file!");

        // make sure the unzipped directory has the expected contents
        let untared_file_paths = list_files_recursive(&unarch_path);
        assert_that!(untared_file_paths.len(), equal_to(5));

        assert_that!(untared_file_paths[0].ends_with("/rc/file-a.txt"), equal_to(true));
        assert_that!(untared_file_paths[1].ends_with("/rc/alpha/file-b.txt"), equal_to(true));
        assert_that!(untared_file_paths[2].ends_with("/rc/alpha/charlie/file-d.txt"), equal_to(true));
        assert_that!(untared_file_paths[3].ends_with("/rc/alpha/charlie/file-e.txt"), equal_to(true));
        assert_that!(untared_file_paths[4].ends_with("/rc/alpha/bravo/file-c.txt"), equal_to(true));
    }
}