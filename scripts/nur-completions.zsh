#compdef nur

_nur_tasks() {
    [[ $PREFIX = -* ]] && return 1
    local tasks; tasks=(
        "${(@f)$(_call_program commands nur --list)}"
    )

    _describe 'nur tasks' tasks
}

_nur() {
    local curcontext="$curcontext" state line ret=1
    typeset -A opt_args

    _arguments -C \
        '--help[Display the help message for this command]' \
        '--version[Output version number and exit]' \
        '--list[List available tasks and then just exit]' \
        '--quiet[Do not output anything but what the task produces]' \
        '--stdin[Attach stdin to called nur task]' \
        '::optional arg:_nur_tasks' \
        '*: :->args' \
        && ret=0

    return ret
}

compdef _nur nur
