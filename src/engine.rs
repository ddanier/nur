use crate::args::{parse_commandline_args, NurArgs};
use crate::errors::{NurError, NurResult};
use crate::names::{
    NUR_ENV_NU_LIB_DIRS, NUR_NAME, NUR_VAR_CONFIG_DIR, NUR_VAR_DEFAULT_LIB_DIR,
    NUR_VAR_PROJECT_PATH, NUR_VAR_RUN_PATH, NUR_VAR_TASK_NAME,
};
use crate::nu_version::NU_VERSION;
use crate::state::NurState;
use nu_cli::gather_parent_env_vars;
use nu_engine::get_full_help;
use nu_protocol::ast::Block;
use nu_protocol::engine::{Command, Stack, StateWorkingSet};
use nu_protocol::eval_const::create_nu_constant;
use nu_protocol::{
    engine::EngineState, report_error, report_error_new, PipelineData, Record, Span, Type, Value,
    NU_VARIABLE_ID,
};
use nu_std::load_standard_library;
use nu_utils::stdout_write_all_and_flush;
use std::fs;
use std::path::Path;
use std::sync::Arc;

pub(crate) fn init_engine_state<P: AsRef<Path>>(project_path: P) -> NurResult<EngineState> {
    let engine_state = nu_cmd_lang::create_default_context();
    let engine_state = nu_command::add_shell_command_context(engine_state);
    let engine_state = nu_cmd_extra::add_extra_command_context(engine_state);
    #[cfg(feature = "dataframe")]
    let engine_state = nu_cmd_dataframe::add_dataframe_context(engine_state);
    let engine_state = nu_cli::add_cli_context(engine_state);
    let engine_state = nu_explore::add_explore_context(engine_state);
    let engine_state = crate::commands::create_nu_context(engine_state);
    let engine_state = crate::commands::create_nur_context(engine_state);

    // Prepare engine state to be changed
    let mut engine_state = engine_state;

    // First, set up env vars as strings only
    gather_parent_env_vars(&mut engine_state, project_path.as_ref());
    engine_state.add_env_var(
        "NU_VERSION".to_string(),
        Value::string(NU_VERSION, Span::unknown()),
    );

    // Load std library
    if load_standard_library(&mut engine_state).is_err() {
        return Err(NurError::InitError(String::from(
            "Could not load std library",
        )));
    }

    // Set some engine flags
    engine_state.is_interactive = false;
    engine_state.is_login = false;
    engine_state.history_enabled = false;

    Ok(engine_state)
}

#[derive(Clone)]
pub(crate) struct NurEngine {
    pub(crate) engine_state: EngineState,
    pub(crate) stack: Stack,

    pub(crate) state: NurState,
}

impl NurEngine {
    pub(crate) fn new(engine_state: EngineState, nur_state: NurState) -> NurResult<NurEngine> {
        let mut nur_engine = NurEngine {
            engine_state,
            stack: Stack::new(),

            state: nur_state,
        };

        nur_engine._apply_nur_state()?;

        Ok(nur_engine)
    }

    fn _apply_nur_state(&mut self) -> NurResult<()> {
        // Set default scripts path
        self.engine_state.add_env_var(
            NUR_ENV_NU_LIB_DIRS.to_string(),
            Value::test_string(self.state.lib_dir_path.to_string_lossy()),
        );

        // Set config and env paths to .nur versions
        self.engine_state
            .set_config_path("env-path", self.state.env_path.clone());
        self.engine_state
            .set_config_path("config-path", self.state.config_path.clone());

        // Set up the $nu constant before evaluating any files (need to have $nu available in them)
        let nu_const = create_nu_constant(
            &self.engine_state,
            PipelineData::empty().span().unwrap_or_else(Span::unknown),
        )?;
        self.engine_state
            .set_variable_const_val(NU_VARIABLE_ID, nu_const);

        // Set up the $nur constant record (like $nu)
        let mut nur_record = Record::new();
        nur_record.push(
            NUR_VAR_RUN_PATH,
            Value::string(
                String::from(self.state.run_path.to_str().unwrap()),
                Span::unknown(),
            ),
        );
        nur_record.push(
            NUR_VAR_PROJECT_PATH,
            Value::string(
                String::from(self.state.project_path.to_str().unwrap()),
                Span::unknown(),
            ),
        );
        nur_record.push(
            NUR_VAR_TASK_NAME,
            Value::string(&self.state.task_name, Span::unknown()),
        );
        nur_record.push(
            NUR_VAR_CONFIG_DIR,
            Value::string(
                String::from(self.state.config_dir.to_str().unwrap()),
                Span::unknown(),
            ),
        );
        nur_record.push(
            NUR_VAR_DEFAULT_LIB_DIR,
            Value::string(
                String::from(self.state.lib_dir_path.to_str().unwrap()),
                Span::unknown(),
            ),
        );
        let mut working_set = StateWorkingSet::new(&self.engine_state);
        let nur_var_id = working_set.add_variable(
            NUR_NAME.as_bytes().into(),
            Span::unknown(),
            Type::Any,
            false,
        );
        self.stack
            .add_var(nur_var_id, Value::record(nur_record, Span::unknown()));
        self.engine_state.merge_delta(working_set.render())?;

        Ok(())
    }

    pub(crate) fn parse_args(&mut self) -> NurArgs {
        parse_commandline_args(&self.state.args_to_nur.join(" "), &mut self.engine_state)
            .unwrap_or_else(|_| std::process::exit(1))
    }

    fn _parse_nu_script(
        &mut self,
        file_path: Option<&str>,
        contents: String,
    ) -> NurResult<Arc<Block>> {
        if file_path.is_some() {
            self.engine_state.start_in_file(file_path);
        }

        let mut working_set = StateWorkingSet::new(&self.engine_state);
        let block = nu_parser::parse(&mut working_set, file_path, &contents.into_bytes(), false);

        if working_set.parse_errors.is_empty() {
            let delta = working_set.render();
            self.engine_state.merge_delta(delta)?;

            Ok(block)
        } else {
            if let Some(err) = working_set.parse_errors.first() {
                report_error(&working_set, err);
                std::process::exit(1);
            }

            Err(NurError::ParseErrors(working_set.parse_errors))
        }
    }

    fn _execute_block(&mut self, block: &Block, input: PipelineData) -> NurResult<PipelineData> {
        nu_engine::get_eval_block(&self.engine_state)(
            &self.engine_state,
            &mut self.stack,
            block,
            input,
        )
        .map_err(|err| {
            report_error_new(&self.engine_state, &err);
            std::process::exit(1);
        })
    }

    fn _eval<S: ToString>(
        &mut self,
        file_path: Option<&str>,
        contents: S,
        input: PipelineData,
        print: bool,
        merge_env: bool,
    ) -> NurResult<i64> {
        let str_contents = contents.to_string();

        if str_contents.is_empty() {
            return Ok(0);
        }

        let block = self._parse_nu_script(file_path, str_contents)?;

        let result = self._execute_block(&block, input)?;

        // Merge env is requested
        if merge_env {
            match nu_engine::env::current_dir(&self.engine_state, &self.stack) {
                Ok(cwd) => {
                    if let Err(e) = self.engine_state.merge_env(&mut self.stack, cwd) {
                        let working_set = StateWorkingSet::new(&self.engine_state);
                        report_error(&working_set, &e);
                    }
                }
                Err(e) => {
                    let working_set = StateWorkingSet::new(&self.engine_state);
                    report_error(&working_set, &e);
                }
            }
        }

        // Print result is requested
        if print {
            let exit_code = result.print(&self.engine_state, &mut self.stack, false, false)?;
            Ok(exit_code)
        } else {
            if let PipelineData::ExternalStream {
                exit_code: Some(exit_code),
                ..
            } = result
            {
                let mut exit_codes: Vec<_> = exit_code.into_iter().collect();
                return match exit_codes.pop() {
                    #[cfg(unix)]
                    Some(Value::Error { error, .. }) => Err(NurError::from(*error)),
                    Some(Value::Int { val, .. }) => Ok(val),
                    _ => Ok(0),
                };
            }
            Ok(0)
        }
    }

    // pub fn eval<S: ToString>(&mut self, contents: S, input: PipelineData) -> NurResult<i64> {
    //     self._eval(None, contents, input, false, false)
    // }

    pub(crate) fn eval_and_print<S: ToString>(
        &mut self,
        contents: S,
        input: PipelineData,
    ) -> NurResult<i64> {
        self._eval(None, contents, input, true, false)
    }

    pub(crate) fn eval_and_merge_env<S: ToString>(
        &mut self,
        contents: S,
        input: PipelineData,
    ) -> NurResult<i64> {
        self._eval(None, contents, input, false, true)
    }

    pub(crate) fn source<P: AsRef<Path>>(
        &mut self,
        file_path: P,
        input: PipelineData,
    ) -> NurResult<i64> {
        let contents = fs::read_to_string(&file_path)?;

        self._eval(file_path.as_ref().to_str(), contents, input, false, false)
    }

    pub(crate) fn source_and_merge_env<P: AsRef<Path>>(
        &mut self,
        file_path: P,
        input: PipelineData,
    ) -> NurResult<i64> {
        let contents = fs::read_to_string(&file_path)?;

        self._eval(file_path.as_ref().to_str(), contents, input, false, true)
    }

    pub(crate) fn has_def<S: AsRef<str>>(&self, name: S) -> bool {
        self.engine_state
            .find_decl(name.as_ref().as_bytes(), &[])
            .is_some()
    }

    pub(crate) fn get_def<S: AsRef<str>>(&self, name: S) -> Option<&dyn Command> {
        if let Some(decl_id) = self.engine_state.find_decl(name.as_ref().as_bytes(), &[]) {
            Some(self.engine_state.get_decl(decl_id))
        } else {
            None
        }
    }

    pub(crate) fn print_help(&mut self, command: &dyn Command) {
        let full_help = get_full_help(
            &command.signature(),
            &command.examples(),
            &self.engine_state,
            &mut self.stack,
            true,
        );

        let _ = std::panic::catch_unwind(move || stdout_write_all_and_flush(full_help));
    }
}
