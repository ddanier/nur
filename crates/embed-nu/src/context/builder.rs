use crate::{
    error::CrateResult,
    into_value::IntoValue,
    utils::{parse_nu_script, NewEmpty},
};
use std::env;

use nu_protocol::{
    ast::Block,
    engine::{Command, EngineState, Stack, StateWorkingSet},
    PipelineData, Span,
};

use super::{CommandGroupConfig, Context};

/// Builder to create a new nu engine state
pub struct ContextBuilder {
    engine_state: EngineState,
    stack: Stack,
    blocks: Vec<Block>,
}

impl Default for ContextBuilder {
    fn default() -> Self {
        Self {
            engine_state: EngineState::new(),
            stack: Stack::new(),
            blocks: Vec::new(),
        }
    }
}

impl ContextBuilder {
    /// Enables certain command groups specified in the Config on the state
    pub fn with_command_groups(mut self, group_config: CommandGroupConfig) -> CrateResult<Self> {
        macro_rules! toggle_command_groups {
            ($($group:ident),*) => {
                paste::item!(
                $(
                    if group_config.$group {
                        super::bindings::[<bind_ $group _commands>](&mut self.engine_state)?;
                    }
                )*
                )
            }
        }

        toggle_command_groups!(
            core,
            debug,
            filter,
            chart,
            misc,
            path,
            system,
            string,
            bit,
            byte,
            file_system,
            platform,
            date,
            shell,
            format,
            viewer,
            conversion,
            environment,
            math,
            network,
            random,
            generator,
            hash,
            experimental
        );
        Ok(self)
    }

    /// Adds a custom command to the engine that can be used inside nu expressions
    /// This method can also be used to pass nu builtin commands to enable single commands
    /// Instead of whole command groups
    pub fn add_command<C: Command + 'static>(mut self, command: C) -> CrateResult<Self> {
        let mut working_set = StateWorkingSet::new(&self.engine_state);
        working_set.add_decl(Box::new(command));
        let delta = working_set.render();
        self.engine_state.merge_delta(delta)?;

        Ok(self)
    }

    /// Adds a variable to the state
    pub fn add_var<S: ToString, V: IntoValue>(mut self, name: S, value: V) -> CrateResult<Self> {
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

        Ok(self)
    }

    /// Adds an environment variable to the state
    pub fn add_env_var<S: ToString, V: IntoValue>(mut self, name: S, value: V) -> Self {
        self.engine_state
            .add_env_var(name.to_string(), value.into_value());

        self
    }

    /// Adds the environment variables of the parent process to the
    /// states env variables
    pub fn add_parent_env_vars(self) -> Self {
        let mut builder = self;

        for (name, val) in env::vars() {
            builder = builder.add_env_var(name, val);
        }

        builder
    }

    /// Adds a block to the builder
    /// This block is evaluated when building to put
    /// the blocks contents in scope.
    /// Note: Code not contained in declarations is being executed when building
    ///       the context
    pub fn add_block(mut self, block: Block) -> Self {
        self.blocks.push(block);

        self
    }

    /// Adds a script to the context.
    /// This script is being parsed so this operation can fail
    pub fn add_script(mut self, contents: String) -> CrateResult<Self> {
        let block = parse_nu_script(&mut self.engine_state, contents)?;
        self.blocks.push(block);

        Ok(self)
    }

    /// builds the context
    pub fn build(self) -> CrateResult<Context> {
        let mut ctx = Context {
            engine_state: self.engine_state,
            stack: self.stack,
        };
        for block in self.blocks {
            ctx.eval_block(&block, PipelineData::empty())?;
        }

        Ok(ctx)
    }
}
