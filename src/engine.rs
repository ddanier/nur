use std::path::PathBuf;
use nu_cli::gather_parent_env_vars;
use nu_protocol::{engine::EngineState, report_error_new, Span, Value};
use nu_std::load_standard_library;
use crate::errors::{NurError, NurResult};
use crate::nu_version::NU_VERSION;

pub fn init_engine_state(project_path: &PathBuf) -> NurResult<EngineState> {
    let engine_state = nu_cmd_lang::create_default_context();
    let engine_state = nu_command::add_shell_command_context(engine_state);
    #[cfg(feature = "extra")]
        let engine_state = nu_cmd_extra::add_extra_command_context(engine_state);
    #[cfg(feature = "dataframe")]
        let engine_state = nu_cmd_dataframe::add_dataframe_context(engine_state);
    let engine_state = nu_cli::add_cli_context(engine_state);
    let engine_state = nu_explore::add_explore_context(engine_state);

    // Prepare engine state to be changed
    let mut engine_state = engine_state;

    // Custom additions only used in cli
    let delta = {
        let mut working_set = nu_protocol::engine::StateWorkingSet::new(&engine_state);
        working_set.add_decl(Box::new(nu_cli::NuHighlight));
        working_set.add_decl(Box::new(nu_cli::Print));
        working_set.render()
    };

    if let Err(err) = engine_state.merge_delta(delta) {
        report_error_new(&engine_state, &err);
        return Err(NurError::NurInitError(String::from("Could not load CLI functions")))
    }

    // First, set up env vars as strings only
    gather_parent_env_vars(&mut engine_state, project_path);
    engine_state.add_env_var(
        "NU_VERSION".to_string(),
        Value::string(NU_VERSION, Span::unknown()),
    );

    // Load std library
    if let Err(_) = load_standard_library(&mut engine_state) {
        return Err(NurError::NurInitError(String::from("Could not load std library")))
    }

    // Set some engine flags
    engine_state.is_interactive = false;
    engine_state.is_login = false;
    engine_state.history_enabled = false;

    Ok(engine_state)
}
