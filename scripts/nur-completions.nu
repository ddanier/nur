def "nu-complete nur task-names" [] {
  ^nur --list | lines | str trim
}

# nur - a taskrunner based on nu shell.
export extern nur [
  --help(-h)  # Display the help message for this command
  --version  # Output version number and exit
  --list  # List available tasks and then just exit
  --quiet  # Do not output anything but what the task produces
  --stdin  # Attach stdin to called nur task
  task_name: string@"nu-complete nur task-names"  # Name of the task to run (optional)
  ...args  # Parameters to the executed task
]
