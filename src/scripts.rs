pub(crate) fn get_default_nur_env() -> &'static str {
    include_str!("nu-scripts/default_nur_env.nu")
}

pub(crate) fn get_default_nur_config() -> &'static str {
    include_str!("nu-scripts/default_nur_config.nu")
}
