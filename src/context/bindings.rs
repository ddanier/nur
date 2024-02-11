use nu_cmd_lang::*;
use nu_command::*;
use nu_protocol::engine::{EngineState, StateWorkingSet};

use crate::error::CrateResult;

macro_rules! bind_commands {
    ($engine_state:expr, $( $command:expr),* $(,)? ) => {
        bind($engine_state, |working_set| {
                $( working_set.add_decl(Box::new($command)); )*
        })
    };
}

pub fn bind_core_commands(engine_state: &mut EngineState) -> CrateResult<()> {
    bind_commands!(
        engine_state,
        Metadata,
        Alias,
        Break,
        Collect,
        Const,
        Continue,
        Def,
        DefEnv,
        Describe,
        Do,
        Echo,
        ErrorMake,
        ExportAlias,
        ExportCommand,
        ExportDef,
        ExportDefEnv,
        ExportExtern,
        ExportUse,
        Extern,
        For,
        Help,
        HelpAliases,
        HelpCommands,
        HelpModules,
        HelpExterns,
        HelpOperators,
        Hide,
        HideEnv,
        If,
        Ignore,
        Overlay,
        OverlayUse,
        OverlayList,
        OverlayNew,
        OverlayHide,
        Let,
        Loop,
        Match,
        Module,
        Mut,
        Return,
        Try,
        Use,
        Version,
        While,
    )
}

pub fn bind_debug_commands(engine_state: &mut EngineState) -> CrateResult<()> {
    bind_commands!(
        engine_state,
        Ast,
        Debug,
        Explain,
        Inspect,
        Metadata,
        Profile,
        TimeIt,
        View,
        ViewFiles,
        ViewSource,
        ViewSpan,
    )
}

pub fn bind_chart_commands(engine_state: &mut EngineState) -> CrateResult<()> {
    bind_commands!(engine_state, Histogram)
}

pub fn bind_filter_commands(engine_state: &mut EngineState) -> CrateResult<()> {
    bind_commands! {
    engine_state,
            All,
            Any,
            Append,
            Columns,
            Compact,
            Default,
            Drop,
            DropColumn,
            DropNth,
            Each,
            EachWhile,
            Empty,
            Enumerate,
            Every,
            Filter,
            Find,
            First,
            Flatten,
            Get,
            Group,
            GroupBy,
            Headers,
            Insert,
            Join,
            SplitBy,
            Take,
            Merge,
            Move,
            TakeWhile,
            TakeUntil,
            Last,
            Length,
            Lines,
            ParEach,
            Prepend,
            Range,
            Reduce,
            Reject,
            Rename,
            Reverse,
            Roll,
            RollDown,
            RollUp,
            RollLeft,
            RollRight,
            Rotate,
            Select,
            Shuffle,
            Skip,
            SkipUntil,
            SkipWhile,
            Sort,
            SortBy,
            SplitList,
            Transpose,
            Uniq,
            UniqBy,
            Upsert,
            Update,
            UpdateCells,
            Values,
            Where,
            Window,
            Wrap,
            Zip,
    }
}

pub fn bind_misc_commands(engine_state: &mut EngineState) -> CrateResult<()> {
    bind_commands!(engine_state, Tutor)
}

pub fn bind_path_commands(engine_state: &mut EngineState) -> CrateResult<()> {
    bind_commands! {
        engine_state,
            Path,
            PathBasename,
            PathDirname,
            PathExists,
            PathExpand,
            PathJoin,
            PathParse,
            PathRelativeTo,
            PathSplit,
            PathType,
    }
}

#[cfg(windows)]
pub fn bind_system_commands(engine_state: &mut EngineState) -> CrateResult<()> {
    bind_commands! {
        engine_state,
        Complete,
        External,
        NuCheck,
        Sys,
        Ps,
        Which,
        RegistryQuery
    }
}

#[cfg(unix)]
pub fn bind_system_commands(engine_state: &mut EngineState) -> CrateResult<()> {
    bind_commands! {
        engine_state,
        Complete,
        External,
        NuCheck,
        Sys,
        Ps,
        Which,
        Exec
    }
}

#[cfg(not(any(unix, windows)))]
pub fn bind_system_commands(engine_state: &mut EngineState) -> CrateResult<()> {
    bind_commands! {
        engine_state,
        Complete,
        External,
        NuCheck,
        Sys,
        Ps,
        Which
    }
}

pub fn bind_string_commands(engine_state: &mut EngineState) -> CrateResult<()> {
    bind_commands! {
        engine_state,
            Char,
            Decode,
            Encode,
            DecodeBase64,
            EncodeBase64,
            DecodeHex,
            EncodeHex,
            DetectColumns,
            Format,
            FileSize,
            Parse,
            Size,
            Split,
            SplitChars,
            SplitColumn,
            SplitRow,
            SplitWords,
            Str,
            StrCamelCase,
            StrCapitalize,
            StrContains,
            StrDistance,
            StrDowncase,
            StrEndswith,
            StrJoin,
            StrReplace,
            StrIndexOf,
            StrKebabCase,
            StrLength,
            StrPascalCase,
            StrReverse,
            StrScreamingSnakeCase,
            StrSnakeCase,
            StrStartsWith,
            StrSubstring,
            StrTrim,
            StrTitleCase,
            StrUpcase
    }
}

pub fn bind_bit_commands(engine_state: &mut EngineState) -> CrateResult<()> {
    bind_commands! {
        engine_state,
        Bits,
        BitsAnd,
        BitsNot,
        BitsOr,
        BitsXor,
        BitsRotateLeft,
        BitsRotateRight,
        BitsShiftLeft,
        BitsShiftRight,
    }
}

pub fn bind_byte_commands(engine_state: &mut EngineState) -> CrateResult<()> {
    bind_commands! {
        engine_state,
        Bytes,
        BytesLen,
        BytesStartsWith,
        BytesEndsWith,
        BytesReverse,
        BytesReplace,
        BytesAdd,
        BytesAt,
        BytesIndexOf,
        BytesCollect,
        BytesRemove,
        BytesBuild,
    }
}

pub fn bind_file_system_commands(engine_state: &mut EngineState) -> CrateResult<()> {
    bind_commands! {
        engine_state,
        Cd,
        Cp,
        Ls,
        Mkdir,
        Mv,
        Open,
        Rm,
        Save,
        Touch,
        Glob,
        Watch,
    }
}

pub fn bind_platform_commands(engine_state: &mut EngineState) -> CrateResult<()> {
    bind_commands! {
        engine_state,
        Ansi,
        AnsiGradient,
        AnsiStrip,
        Clear,
        Du,
        Input,
        Kill,
        Sleep,
        TermSize,
    }
}

pub fn bind_date_commands(engine_state: &mut EngineState) -> CrateResult<()> {
    bind_commands! {
        engine_state,
        Date,
        DateFormat,
        DateHumanize,
        DateListTimezones,
        DateNow,
        DateToRecord,
        DateToTable,
        DateToTimezone,
    }
}

pub fn bind_shell_commands(engine_state: &mut EngineState) -> CrateResult<()> {
    bind_commands! {
        engine_state,
        Exit,
    }
}

pub fn bind_format_commands(engine_state: &mut EngineState) -> CrateResult<()> {
    bind_commands! {
        engine_state,
            From,
            FromCsv,
            FromJson,
            FromNuon,
            FromOds,
            FromSsv,
            FromToml,
            FromTsv,
            FromUrl,
            FromXlsx,
            FromXml,
            FromYaml,
            FromYml,
            To,
            ToCsv,
            ToHtml,
            ToJson,
            ToMd,
            ToNuon,
            ToText,
            ToToml,
            ToTsv,
            Touch,
            Upsert,
            Where,
            ToXml,
            ToYaml,
    }
}

pub fn bind_viewer_commands(engine_state: &mut EngineState) -> CrateResult<()> {
    bind_commands! {
        engine_state,
        Griddle,
        Table,
            Explore,
    }
}

pub fn bind_conversion_commands(engine_state: &mut EngineState) -> CrateResult<()> {
    bind_commands! {
        engine_state,
        Fill,
        Fmt,
        Into,
        IntoBool,
        IntoBinary,
        IntoDatetime,
        IntoDecimal,
        IntoDuration,
        IntoFilesize,
        IntoInt,
        IntoRecord,
        IntoString,
    }
}

pub fn bind_environment_commands(engine_state: &mut EngineState) -> CrateResult<()> {
    bind_commands! {
        engine_state,
            ExportEnv,
            LetEnv,
            LoadEnv,
            SourceEnv,
            WithEnv,
        // nu config commands have been removed as editing isn't possible
        // in this environment
    }
}

pub fn bind_math_commands(engine_state: &mut EngineState) -> CrateResult<()> {
    bind_commands! {
        engine_state,
            Math,
            MathAbs,
            MathAvg,
            MathCeil,
            MathFloor,
            MathMax,
            MathMedian,
            MathMin,
            MathMode,
            MathProduct,
            MathRound,
            MathSqrt,
            MathStddev,
            MathSum,
            MathVariance,
            MathSin,
            MathCos,
            MathTan,
            MathSinH,
            MathCosH,
            MathTanH,
            MathArcSin,
            MathArcCos,
            MathArcTan,
            MathArcSinH,
            MathArcCosH,
            MathArcTanH,
            MathPi,
            MathTau,
            MathEuler,
            MathExp,
            MathLn,
            MathLog,
    }
}

pub fn bind_network_commands(engine_state: &mut EngineState) -> CrateResult<()> {
    bind_commands! {
        engine_state,
            Http,
            HttpDelete,
            HttpGet,
            HttpHead,
            HttpPatch,
            HttpPost,
            HttpPut,
            Url,
            UrlBuildQuery,
            UrlEncode,
            UrlJoin,
            UrlParse,
            Port,
    }
}

pub fn bind_random_commands(engine_state: &mut EngineState) -> CrateResult<()> {
    bind_commands! {
        engine_state,
        Random,
        RandomBool,
        RandomChars,
        RandomDecimal,
        RandomDice,
        RandomInteger,
        RandomUuid,
    }
}

pub fn bind_generator_commands(engine_state: &mut EngineState) -> CrateResult<()> {
    bind_commands! {
        engine_state,
        Cal,
        Seq,
        SeqDate,
        SeqChar,
    }
}

pub fn bind_hash_commands(engine_state: &mut EngineState) -> CrateResult<()> {
    bind_commands! {
        engine_state,
        Hash,
        HashMd5::default(),
        HashSha256::default(),
    }
}

pub fn bind_experimental_commands(engine_state: &mut EngineState) -> CrateResult<()> {
    bind_commands! {
        engine_state,
        IsAdmin,
    }
}

#[inline]
fn bind<F: Fn(&mut StateWorkingSet)>(
    engine_state: &mut EngineState,
    bind_fn: F,
) -> CrateResult<()> {
    let mut working_set = StateWorkingSet::new(engine_state);
    bind_fn(&mut working_set);
    let delta = working_set.render();
    engine_state.merge_delta(delta)?;
    Ok(())
}
