mod nur;

use nu_protocol::engine::{EngineState, StateWorkingSet};
pub(crate) use nur::Nur;

pub(crate) fn create_nu_context(mut engine_state: EngineState) -> EngineState {
    // Custom additions only used in cli, normally registered in nu main() as "custom additions"
    let delta = {
        let mut working_set = StateWorkingSet::new(&engine_state);
        working_set.add_decl(Box::new(nu_cli::NuHighlight));
        working_set.add_decl(Box::new(nu_cli::Print));
        working_set.render()
    };

    if let Err(err) = engine_state.merge_delta(delta) {
        eprintln!("Error creating nu command context: {err:?}");
    }

    engine_state
}

pub(crate) fn create_nur_context(mut engine_state: EngineState) -> EngineState {
    // Add nur own commands
    let delta = {
        let mut working_set = StateWorkingSet::new(&engine_state);
        working_set.add_decl(Box::new(nur::Nur));
        working_set.render()
    };

    if let Err(err) = engine_state.merge_delta(delta) {
        eprintln!("Error creating nur command context: {err:?}");
    }

    engine_state
}
