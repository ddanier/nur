use nu_protocol::{
    engine::EngineState, report_error_new,
};

pub fn get_engine_state() -> EngineState {
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
    }

    engine_state
}
