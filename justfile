default:
    just --list

cargo *args:
    cargo {{args}}

build *args: (cargo "build" args)
run *args: (cargo "run" args)

release: (cargo "build" "--release")
