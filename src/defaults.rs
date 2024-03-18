use nu_utils::{
    get_default_config as nu_get_default_config, get_default_env as nu_get_default_env,
};

pub(crate) fn get_default_config() -> &'static str {
    nu_get_default_config()
}

pub(crate) fn get_default_env() -> &'static str {
    nu_get_default_env()
}

pub(crate) fn get_default_nur_env() -> &'static str {
    include_str!("nu-scripts/default_nur_env.nu")
}
