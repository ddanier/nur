use nu_utils::get_default_config as nu_get_default_config;

pub(crate) fn get_default_nur_config() -> &'static str {
    // Just use nu default config for now
    nu_get_default_config()
}

pub(crate) fn get_default_nur_env() -> &'static str {
    include_str!("nu-scripts/default_nur_env.nu")
}
