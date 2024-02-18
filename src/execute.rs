use nu_protocol::{
    ast::{Block, Call},
    engine::{EngineState, Stack, StateWorkingSet},
    PipelineData, Span,
};
use std::collections::HashMap;
use crate::errors::{NurResult, NurError};
use std::fs;
use std::path::Path;

pub fn parse_nu_script(
    engine_state: &mut EngineState,
    contents: String,
) -> NurResult<Block> {
    let mut working_set = StateWorkingSet::new(&engine_state);
    let block = nu_parser::parse(&mut working_set, None, &contents.into_bytes(), false);

    if working_set.parse_errors.is_empty() {
        let delta = working_set.render();
        engine_state.merge_delta(delta)?;

        Ok(block)
    } else {
        Err(NurError::NurParseErrors(working_set.parse_errors))
    }
}

pub fn execute_block(
    engine_state: &EngineState,
    stack: &mut Stack,
    block: &Block,
    input: PipelineData,
) -> NurResult<PipelineData> {
    nu_engine::eval_block(
        engine_state,
        stack,
        block,
        input,
        false,
        false,
    ).map_err(NurError::from)
}

pub fn eval<S: ToString>(
    engine_state: &mut EngineState,
    stack: &mut Stack,
    contents: S,
    input: PipelineData,
) -> NurResult<PipelineData> {
    let str_contents = contents.to_string();

    if str_contents.len() == 0 {
        return Ok(PipelineData::empty());
    }

    let block = parse_nu_script(
        engine_state,
        str_contents,
    )?;

    execute_block(&engine_state, stack, &block, input)
}

pub fn source<P: AsRef<Path>>(
    engine_state: &mut EngineState,
    stack: &mut Stack,
    file_path: P,
    input: PipelineData,
) -> NurResult<PipelineData> {
    let contents = fs::read_to_string(file_path)?;

    eval(engine_state, stack, contents, input)
}

// pub fn get_var<S: AsRef<str>>(
//     engine_state: &EngineState,
//     stack: &mut Stack,
//     name: S,
// ) -> Option<nu_protocol::Value> {
//     let name = name.as_ref();
//     let dollar_name = format!("${name}");
//     let var_id = engine_state.active_overlays(&vec![]).find_map(|o| {
//         o.vars
//             .get(dollar_name.as_bytes())
//             .or(o.vars.get(name.as_bytes()))
//     })?;
//     stack.get_var(*var_id, Span::new(0, 0)).ok()
// }

pub fn has_def<S: AsRef<str>>(
    engine_state: &EngineState,
    name: S,
) -> bool {
    engine_state
        .find_decl(name.as_ref().as_bytes(), &vec![])
        .is_some()
}

// pub fn call_def<S: AsRef<str>, I: IntoIterator<Item = A>, A: IntoArgument>(
//     engine_state: &EngineState,
//     stack: &mut Stack,
//     name: S,
//     args: I,
// ) -> NurResult<PipelineData> {
//     let args = args
//         .into_iter()
//         .map(|a| a.into_argument().into_nu_argument())
//         .collect::<Vec<_>>();
//
//     let decl_id = engine_state
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
//         &engine_state,
//         stack,
//         &call,
//         PipelineData::empty(),
//     )?;
//
//     Ok(data)
// }
