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
