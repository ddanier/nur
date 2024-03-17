use crate::errors::{NurError, NurResult};
#[cfg(feature = "plugin")]
use nu_cli::read_plugin_file;
use nu_engine::get_full_help;
use nu_protocol::engine::Command;
use nu_protocol::report_error;
use nu_protocol::{
    ast::Block,
    engine::{EngineState, Stack, StateWorkingSet},
    PipelineData, Value,
};
#[cfg(feature = "plugin")]
use nu_protocol::{Span, Spanned};
use nu_utils::stdout_write_all_and_flush;
use std::fs;
use std::path::Path;

#[derive(Clone)]
pub struct Context {
    engine_state: EngineState,
    stack: Stack,
}

impl Context {
    fn _parse_nu_script(&mut self, file_path: Option<&str>, contents: String) -> NurResult<Block> {
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
        nu_engine::eval_block(
            &self.engine_state,
            &mut self.stack,
            block,
            input,
            false,
            false,
        )
        .map_err(NurError::from)
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

    pub fn eval<S: ToString>(&mut self, contents: S, input: PipelineData) -> NurResult<i64> {
        self._eval(None, contents, input, false, false)
    }

    pub fn eval_and_print<S: ToString>(
        &mut self,
        contents: S,
        input: PipelineData,
    ) -> NurResult<i64> {
        self._eval(None, contents, input, true, false)
    }

    pub fn eval_and_merge_env<S: ToString>(
        &mut self,
        contents: S,
        input: PipelineData,
    ) -> NurResult<i64> {
        self._eval(None, contents, input, false, true)
    }

    pub fn source<P: AsRef<Path>>(&mut self, file_path: P, input: PipelineData) -> NurResult<i64> {
        let contents = fs::read_to_string(&file_path)?;

        self._eval(file_path.as_ref().to_str(), contents, input, false, false)
    }

    pub fn source_and_merge_env<P: AsRef<Path>>(
        &mut self,
        file_path: P,
        input: PipelineData,
    ) -> NurResult<i64> {
        let contents = fs::read_to_string(&file_path)?;

        self._eval(file_path.as_ref().to_str(), contents, input, false, true)
    }

    #[cfg(feature = "plugin")]
    pub fn read_plugin_file<P: AsRef<Path>>(&mut self, nur_plugin_path: P) {
        if nur_plugin_path.as_ref().exists() {
            read_plugin_file(
                &mut self.engine_state,
                &mut self.stack,
                Some(Spanned {
                    item: String::from(nur_plugin_path.as_ref().to_str().unwrap()),
                    span: Span::unknown(),
                }),
                "nushell", // will never be used, as we have set nur_plugin_path
            );
        }
    }

    pub fn has_def<S: AsRef<str>>(&self, name: S) -> bool {
        self.engine_state
            .find_decl(name.as_ref().as_bytes(), &[])
            .is_some()
    }

    pub fn get_def<S: AsRef<str>>(&self, name: S) -> Option<&dyn Command> {
        if let Some(decl_id) = self.engine_state.find_decl(name.as_ref().as_bytes(), &[]) {
            Some(self.engine_state.get_decl(decl_id).as_ref())
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

impl From<EngineState> for Context {
    fn from(engine_state: EngineState) -> Context {
        Context {
            engine_state,
            stack: Stack::new(),
        }
    }
}
