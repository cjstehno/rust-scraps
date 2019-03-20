use std::fs::File;
use std::path::Path;

use tar::{Archive, Builder};

// FIXME: this example needs better error handling
// TODO: note that this does not compress the tar (just tar not gz)

// FIXME: document
#[allow(dead_code)]
pub fn create_tar_file(path: &Path, tar_file: &File) -> Result<(), ()> {
    let mut tar_builder = Builder::new(tar_file);

    if path.is_file() {
        tar_builder.append_path(path).expect("Unable to append file.");
    } else if path.is_dir() {
        unimplemented!("not ready yet");
//        tar_multiple(&mut tar_builder, path)?;
    } else {
        // path is not a file or directory
        return Err(());
    }

    match tar_builder.finish() {
        Ok(_) => Ok(()),
        Err(_) => Err(())
    }
}

/*fn tar_multiple(tar_builder: &mut Builder<&File>, path: &Path) -> Result<(), Error> {
    let mut directories = vec![path.to_path_buf()];

    while !directories.is_empty() {
        let directory = directories.pop().unwrap();

        for dir_entry in directory.read_dir().unwrap() {
            let entry = dir_entry.unwrap();
            let file_type = entry.file_type().unwrap();

            if file_type.is_dir() {
                directories.push(entry.path());
            } else if file_type.is_file() {
                tar_builder.append_path(entry.path());
            }
        }
    }

    Ok(())
}*/

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

#[cfg(test)]
mod tests {
    use std::fs::File;
    use std::path::Path;
    use std::borrow::Borrow;

    use hamcrest2::equal_to;
    use hamcrest2::matchers::compared_to::greater_than;
    use hamcrest2::prelude::*;
    use tempfile::TempDir;

    use crate::list_files_recursive;
    use crate::taring::{create_tar_file, list_tar_contents};

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

    /*


        // make sure the file has the items we expect
        let listing_file = File::open(&zip_path).unwrap();
        let entries = list_zip_contents(&listing_file).expect("Unable to list zip contents!");

        assert_that!(&entries, len(1));
        assert_that!(&entries.contains("file-a.txt (12 bytes, 27 compressed)".to_string().borrow()), equal_to(true));
    */
}