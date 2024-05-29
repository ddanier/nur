# Create nurfile from common task/command runner config
#
# Usage:
# > cd into/the/project/path
# > nurify  # Will create nurfile
# > nur --help
#
# The genrated nurify is not meant to replace your current task/commend runner, instead it
# will create a nurfile allowing you to use nur instead of the projects task runner. nur
# will then still execute the original task runner.
# Still this allows you to gradually convert your project to nur by replacing your tasks
# bit by bit with new nur variants. When switching to nur this is the recommended method:
# Create new nurfile to run old tasks using the old task runner, then convert task for task.

def prepare-nurfile [] {
    "# FILE GENERATED BY nurify COMMAND\n\n" | save -f nurfile
}

def nurify-from-b5 [] {
    let global_tasks = (
        if (glob ~/.b5/Taskfile | first | path exists) {
            cat ~/.b5/Taskfile | lines | filter {
                |it| $it starts-with "task:"
            } | each {
                |it| $it | parse --regex "^task:(?P<name>[^(]+).*" | get name | first
            }
        } else []
    )

    prepare-nurfile
    b5 --quiet help --tasks | lines | filter {
        |it| $it not-in $global_tasks
    } | each {
        |it| $"def --wrapped \"nur ($it | str replace --all ':' ' ')\" [...args] {\n    ^b5 --quiet \"($it)\" ...$args\n}\n"
    } | save -f -a nurfile
}

def nurify-from-just [] {
    prepare-nurfile
    ^just --unsorted --dump --dump-format json
        | from json
        | get recipes
        | transpose k v
        | each {
            |it| $"def --wrapped \"nur ($it.k)\" [...args] {\n    ^just \"($it.k)\" ...$args\n}\n"
        }  | save -f -a nurfile
}

def nurify-from-package-json [] {
    prepare-nurfile
    open package.json
        | get scripts
        | transpose k v
        | each {
            |it| $"def --wrapped \"nur ($it.k)\" [...args] {\n    ^npm run \"($it.k)\" ...$args\n}\n"
        }  | save -f -a nurfile
}

# Create nurfile from different task/command runners. The nurfile will contain tasks to wrap
# all tasks and then run original task/command runner. This can be used to gradually migrate to nur.
#
# Currently supports:
# * b5 task runner (when build/Taskfile is found)
# * just command runner (when justfile is found)
# * npm package.json scripts (when package.json is found)
export def main [] {
    if ("build/Taskfile" | path exists) {
        nurify-from-b5
    } else if ("justfile" | path exists) {
    	nurify-from-just
    } else if ("package.json" | path exists) {
    	nurify-from-package-json
    } else {
        error make {"msg": "Cound not find any existing task/command runner, please run nurify in project root"}
    }
}
