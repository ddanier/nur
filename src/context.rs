use nu_protocol::{ast::Block, engine::{EngineState, Stack, StateWorkingSet}, PipelineData};
use crate::errors::{NurResult, NurError};
use std::fs;
use std::path::Path;
use nu_engine::get_full_help;
use nu_protocol::engine::Command;
use nu_utils::stdout_write_all_and_flush;
use nu_protocol::report_error;

#[derive(Clone)]
pub struct Context {
    engine_state: EngineState,
    stack: Stack,
}

impl Context {
    fn parse_nu_script(
        &mut self,
        file_path: Option<&str>,
        contents: String,
    ) -> NurResult<Block> {
        let mut working_set = StateWorkingSet::new(&self.engine_state);
        let block = nu_parser::parse(
            &mut working_set,
            file_path,
            &contents.into_bytes(),
            false,
        );

        if working_set.parse_errors.is_empty() {
            let delta = working_set.render();
            self.engine_state.merge_delta(delta)?;

            Ok(block)
        } else {
            if let Some(_) = file_path {
                if let Some(err) = working_set.parse_errors.first() {
                    report_error(&working_set, err);
                    std::process::exit(1);
                }
            }

            Err(NurError::NurParseErrors(working_set.parse_errors))
        }
    }

    fn _execute_block(
        &mut self,
        block: &Block,
        input: PipelineData,
    ) -> NurResult<PipelineData> {
        nu_engine::eval_block(
            &self.engine_state,
            &mut self.stack,
            block,
            input,
            false,
            false,
        ).map_err(NurError::from)
    }

    fn _eval<S: ToString>(
        &mut self,
        file_path: Option<&str>,
        contents: S,
        input: PipelineData,
        print: bool,
    ) -> NurResult<()> {
        let str_contents = contents.to_string();

        if str_contents.len() == 0 {
            return Ok(());
        }

        let block = self.parse_nu_script(
            file_path,
            str_contents,
        )?;

        let result = self._execute_block(&block, input)?;

        if print {
            result.print(
                &self.engine_state,
                &mut self.stack,
                false,
                false,
            )?;
        }

        Ok(())
    }

    pub fn eval<S: ToString>(
        &mut self,
        contents: S,
        input: PipelineData,
    ) -> NurResult<()> {
        self._eval(None, contents, input, false)
    }

    pub fn eval_and_print<S: ToString>(
        &mut self,
        contents: S,
        input: PipelineData,
    ) -> NurResult<()> {
        self._eval(None, contents, input, true)
    }

    pub fn source<P: AsRef<Path>>(
        &mut self,
        file_path: P,
        input: PipelineData,
    ) -> NurResult<()> {
        let contents = fs::read_to_string(&file_path)?;

        self._eval(file_path.as_ref().to_str(), contents, input, false)
    }

    // pub fn get_var<S: AsRef<str>>(
    //     &mut self,
    //     name: S,
    // ) -> Option<nu_protocol::Value> {
    //     let name = name.as_ref();
    //     let dollar_name = format!("${name}");
    //     let var_id = self.engine_state.active_overlays(&vec![]).find_map(|o| {
    //         o.vars
    //             .get(dollar_name.as_bytes())
    //             .or(o.vars.get(name.as_bytes()))
    //     })?;
    //     self.stack.get_var(*var_id, Span::new(0, 0)).ok()
    // }

    pub fn has_def<S: AsRef<str>>(
        &self,
        name: S,
    ) -> bool {
        self.engine_state
            .find_decl(name.as_ref().as_bytes(), &vec![])
            .is_some()
    }

    pub fn get_def<S: AsRef<str>>(
        &self,
        name: S,
    ) -> Option<&Box<dyn Command>> {
        if let Some(decl_id) = self.engine_state.find_decl(name.as_ref().as_bytes(), &vec![]) {
            Some(self.engine_state.get_decl(decl_id))
        } else {
            None
        }
    }

    // pub fn call_def<S: AsRef<str>, I: IntoIterator<Item = A>, A: IntoArgument>(
    //     &mut self,
    //     name: S,
    //     args: I,
    // ) -> NurResult<PipelineData> {
    //     let args = args
    //         .into_iter()
    //         .map(|a| a.into_argument().into_nu_argument())
    //         .collect::<Vec<_>>();
    //
    //     let decl_id = self.engine_state
    //         .find_decl(name.as_ref().as_bytes(), &vec![])
    //         .ok_or_else(|| Err(()))?;  // function not found
    //     let call = Call {
    //         decl_id,
    //         head: Span::empty(),
    //         arguments: args,
    //         redirect_stdout: true,
    //         redirect_stderr: true,
    //         parser_info: HashMap::new(),
    //     };
    //
    //     let data = nu_engine::eval_call(
    //         &self.engine_state,
    //         &mut self.stack,
    //         &call,
    //         PipelineData::empty(),
    //     )?;
    //
    //     Ok(data)
    // }

    pub(crate) fn print_help(
        &mut self,
        command: Box<dyn Command>,
    ) {
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
            engine_state: engine_state,
            stack: Stack::new(),
        }
    }
}