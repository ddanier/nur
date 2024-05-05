def "nu-complete nur task-names" [] {
  ^nur --list | lines | str trim
}

# nur - a taskrunner based on nu shell.
export extern nur [
  --help(-h)  # display the help message for this command
  --version  # output version number and exit
  --list  # list available tasks and then just exit
  --quiet  # Do not output anything but what the task produces
  --stdin  # Attach stdin to called nur task
  task_name: string@"nu-complete nur task-names"  # name of the task to run (optional)
  ...args  # parameters to the executed task
]
