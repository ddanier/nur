mod bindings;
mod builder;
mod command_group_config;
use std::collections::HashMap;

pub use builder::*;
pub use command_group_config::CommandGroupConfig;
use nu_protocol::{
    ast::{Block, Call},
    engine::{EngineState, Stack, StateWorkingSet},
    PipelineData, Span,
};

use crate::{
    argument::IntoArgument,
    error::{CrateError, CrateResult},
    utils::parse_nu_script,
    IntoValue, NewEmpty,
};

/// Represents the evaluation context of nu scripts and commands
/// This context is the state of the engine itself plus the stack
/// It stores variables on
#[derive(Clone)]
pub struct Context {
    engine_state: EngineState,
    stack: Stack,
}

impl Context {
    pub fn builder() -> ContextBuilder {
        ContextBuilder::default()
    }

    /// Evaluates the given block with the current engine context (stack plus engine state)
    pub fn eval_block(&mut self, block: &Block, input: PipelineData) -> CrateResult<PipelineData> {
        nu_engine::eval_block(
            &self.engine_state,
            &mut self.stack,
            block,
            input,
            false,
            false,
        )
        .map_err(CrateError::from)
    }

    /// Evals nu script as string with the current engine context
    pub fn eval_raw<S: ToString>(
        &mut self,
        contents: S,
        input: PipelineData,
    ) -> CrateResult<PipelineData> {
        let block = parse_nu_script(&mut self.engine_state, contents.to_string())?;

        self.eval_block(&block, input)
    }

    /// Returns a variable defined in the stack
    pub fn get_var<S: AsRef<str>>(&self, name: S) -> Option<nu_protocol::Value> {
        let name = name.as_ref();
        let dollar_name = format!("${name}");
        let var_id = self.engine_state.active_overlays(&vec![]).find_map(|o| {
            o.vars
                .get(dollar_name.as_bytes())
                .or(o.vars.get(name.as_bytes()))
        })?;
        self.stack.get_var(*var_id, Span::new(0, 0)).ok()
    }

    /// Returns if the given function exists in the context
    pub fn has_fn<S: AsRef<str>>(&mut self, name: S) -> bool {
        self.engine_state
            .find_decl(name.as_ref().as_bytes(), &vec![])
            .is_some()
    }

    /// Calls a function by the given name
    /// Errs if the function doesn't exist
    pub fn call_fn<S: AsRef<str>, I: IntoIterator<Item = A>, A: IntoArgument>(
        &mut self,
        name: S,
        args: I,
    ) -> CrateResult<PipelineData> {
        let args = args
            .into_iter()
            .map(|a| a.into_argument().into_nu_argument())
            .collect::<Vec<_>>();

        let decl_id = self
            .engine_state
            .find_decl(name.as_ref().as_bytes(), &vec![])
            .ok_or_else(|| CrateError::FunctionNotFound(name.as_ref().to_string()))?;
        let call = Call {
            decl_id,
            head: Span::empty(),
            arguments: args,
            redirect_stdout: true,
            redirect_stderr: true,
            parser_info: HashMap::new(),
        };

        let data = nu_engine::eval_call(
            &self.engine_state,
            &mut self.stack,
            &call,
            PipelineData::empty(),
        )?;

        Ok(data)
    }

    /// Prints the data of the given pipeline to stdout
    pub fn print_pipeline(&mut self, pipeline: PipelineData) -> CrateResult<()> {
        pipeline.print(&self.engine_state, &mut self.stack, false, false)?;

        Ok(())
    }

    /// Prints the data of the given pipeline to stderr
    pub fn print_pipeline_stderr(&mut self, pipeline: PipelineData) -> CrateResult<()> {
        pipeline.print(&self.engine_state, &mut self.stack, false, true)?;

        Ok(())
    }

    /// Adds a variable to the context
    pub fn add_var<S: ToString, V: IntoValue>(&mut self, name: S, value: V) -> CrateResult<()> {
        let mut working_set = StateWorkingSet::new(&self.engine_state);

        let var_id = working_set.add_variable(
            name.to_string().into_bytes(),
            Span::empty(),
            nu_protocol::Type::Any,
            false,
        );
        self.stack.add_var(var_id, value.into_value());
        let delta = working_set.render();
        self.engine_state.merge_delta(delta)?;

        Ok(())
    }
}
