# Override some $env options again, as the default env does set those to nu versions
# after already been set to nur versions in main.rs. 🤷‍
$env.NU_LIB_DIRS = [
    $nur.default-lib-dir
]
$env.NU_PLUGIN_DIRS = [
    $nur.default-plugin-dir
]
