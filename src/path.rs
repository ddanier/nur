use std::path::PathBuf;
use crate::errors::{NurError, NurResult};

pub fn find_project_path(
    cwd: &PathBuf,
) -> NurResult<PathBuf> {
    let mut path = cwd.clone();

    loop {
        let taskfile_path = path.join("nurfile");
        if taskfile_path.exists() {
            if let Some(project_path) = path.to_str() {
                return Ok(PathBuf::from(project_path));
            }
        }

        if let Some(parent) = path.parent() {
            path = parent.to_path_buf();
        } else {
            return Err(NurError::NurTaskfileNotFound());
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::env;
    use std::fs::{create_dir, File};

    #[test]
    fn test_find_project_path() {
        // Create a temporary directory and a "nurfile" inside it
        let temp_dir = env::temp_dir();
        let nurfile_path = temp_dir.join("nurfile");
        File::create(&nurfile_path).unwrap();

        // Test the function with the temporary directory as the current working directory
        let expected_path = temp_dir.to_path_buf();
        let actual_path = find_project_path(&temp_dir).unwrap();
        assert_eq!(expected_path, actual_path);

        // Clean up
        std::fs::remove_file(nurfile_path).unwrap();
    }

    #[test]
    fn test_find_project_path_subdirectory() {
        // Create a temporary directory and a subdirectory inside it
        let temp_dir = env::temp_dir();
        let sub_dir = temp_dir.join("sub");
        create_dir(&sub_dir).unwrap();

        // Create a "nurfile" inside the temporary directory
        let nurfile_path = temp_dir.join("nurfile");
        File::create(&nurfile_path).unwrap();

        // Test the function with the subdirectory as the current working directory
        let expected_path = temp_dir.to_path_buf();
        let actual_path = find_project_path(&sub_dir).unwrap();
        assert_eq!(expected_path, actual_path);

        // Clean up
        std::fs::remove_file(nurfile_path).unwrap();
        std::fs::remove_dir(sub_dir).unwrap();
    }

    #[test]
    fn test_find_project_path_error() {
        // Create a temporary directory without a "nurfile"
        let temp_dir = env::temp_dir();

        // Test the function with the temporary directory as the current working directory
        match find_project_path(&temp_dir) {
            Ok(_) => panic!("Expected an error, but got Ok"),
            Err(e) => match e {
                NurError::NurTaskfileNotFound() => (), // Test passes
                _ => panic!("Expected NurTaskfileNotFound, but got a different error"),
            },
        }
    }
}