use crate::args::gather_commandline_args;
use crate::names::{
    NUR_CONFIG_CONFIG_FILENAME, NUR_CONFIG_ENV_FILENAME, NUR_CONFIG_LIB_PATH, NUR_CONFIG_PATH,
    NUR_FILE, NUR_LOCAL_FILE,
};
use crate::path::find_project_path;
use std::env;
use std::path::PathBuf;

#[derive(Clone)]
pub(crate) struct NurState {
    pub(crate) run_path: PathBuf,
    pub(crate) has_project_path: bool,
    pub(crate) project_path: PathBuf,

    pub(crate) config_dir: PathBuf,
    pub(crate) lib_dir_path: PathBuf,
    pub(crate) env_path: PathBuf,
    pub(crate) config_path: PathBuf,

    pub(crate) nurfile_path: PathBuf,
    pub(crate) local_nurfile_path: PathBuf,

    pub(crate) args_to_nur: Vec<String>,
    pub(crate) task_name: String,
    pub(crate) args_to_task: Vec<String>,
}

impl NurState {
    pub(crate) fn new(run_path: PathBuf) -> Self {
        // Get initial directory details
        let found_project_path = find_project_path(&run_path);
        let has_project_path = found_project_path.is_some();
        let project_path = found_project_path.unwrap_or(run_path.clone());

        // Set all paths
        let config_dir = project_path.join(NUR_CONFIG_PATH);
        let lib_dir_path = config_dir.join(NUR_CONFIG_LIB_PATH);
        let env_path = config_dir.join(NUR_CONFIG_ENV_FILENAME);
        let config_path = config_dir.join(NUR_CONFIG_CONFIG_FILENAME);
        let nurfile_path = project_path.join(NUR_FILE);
        let local_nurfile_path = project_path.join(NUR_LOCAL_FILE);

        let (args_to_nur, task_name, args_to_task) = gather_commandline_args(&mut env::args());

        NurState {
            run_path,
            has_project_path,
            project_path,

            config_dir,
            lib_dir_path,
            env_path,
            config_path,

            nurfile_path,
            local_nurfile_path,

            args_to_nur,
            task_name,
            args_to_task,
        }
    }
}
