# nur - the simple nu based task runner

`nur` is a simple, yet very powerful task runner. It borrows ideas from [`b5`](https://github.com/team23/b5)
and [`just`](https://github.com/casey/just), but uses [`nu` scripting](https://www.nushell.sh/book/programming_in_nu.md)
to define the tasks. This allows for very powerful, yet well-structured tasks.

## Warning / disclaimer

In its current state `nur` is more or less a **beta** software. I wanted to put it out there, so I may
receive some feedback. But I am only just starting using this in some production setup myself.
So feel free to poke around with this, but be aware this is far from being finished or anything.
Meaning: There might be dragons!

## Usage example

`nur` allows you to execute tasks define in a file called `nurfile`. It will look through your
current working directory and all its parents to look for this file. When it has found the `nurfile`
it will change to the directory the file was found in and then `source` the file into `nu` script.
You can define tasks like this:

```
# Just tell anybody or the "world" hello
def "nur hello" [
    name: string = "world"  # The name to say hello to
] {
    print $"hello ($name)"
}
```

The important bit is that you define your tasks as subcommands for "nur". If you then execute
`nur hello` it will print "hello world", meaning it did execute the task `hello` in your `nurfile`.
You can also use `nur --help` to get some details on how to use `nur` and `nur --help hello` to
see what this `hello` task accepts as parameters.

You may also pass arguments to your `nur` tasks, like using `nur hello bob` to pass "bob"
as the name to the "hello" task. This supports all parameter variants normal `nu` scripts could also
handle. You may use `nur --help <task-name>` to see the help for an available command.

Your tasks then can do whatever you want them to do in `nu` script. This allows for very structured
usage of for example docker to run/manage your project needs. But it can also execute simple commands
like you would normally do in your shell (like `npm ci` or something). `nur` is not tight to any
programming language, packaging system or anything. As in the end the `nurfile` is basically a
normal `nu` script you can put into this whatever you like.

See `nu` [custom commands](https://www.nushell.sh/book/custom_commands.html) for details on how to define
tasks and at least read through the [nu quick tour](https://www.nushell.sh/book/quick_tour.html) to
understand some basics and benefits about `nu` scripting.

## Installing `nur`

As of now `nur` is not available in common package managers. This is however no issue as `cargo`
allows you to install packages into your own user directory.

**Note:** You need to have `cargo` installed for this to work. See [cargo install docs](https://doc.rust-lang.org/cargo/getting-started/installation.html)
for details on getting `cargo` running.

Just run `cargo install nur` to install `nur` for your current user. The `nur` binary will be
added in `$HOME/.cargo/bin` (or `$"($env.HOME)/.cargo/bin"` in `nu` shell). Make sure to add
this to `$PATH` (or `$env.PATH` in `nu` shell).

Shell example (like Bash, zsh, ...):
```
cargo install nur
export PATH="$HOME/.cargo/bin:$PATH"  # put this into your .bashrc, .zshrc or similar
nur --version
```

`nu` shell example:
```
cargo install nur
$env.PATH = ($env.PATH | split row (char esep) | prepend [$'($nu.home-path)/bin'])  # put this into $nu.env-path
nur --version
```

## Working with `nur`

As shown above you can use subcommands to `"nur"` to add your tasks. This section will give
you some more details and some hints how to do this in the best way possible.

### About the `nurfile`

Your tasks are defined in a file called `nurfile`. This file is a normal `nu` script and may
use `nu` commands to define `nur` tasks. All tasks must be defined as subcommands to `"nur"`, you
still will be able to define any other commands and use those as helpers in your tasks. Only
subcommands to `"nur"` will be exposed by running `nur`.

In addition you may add a file called `nurfile.local` to define personal, additional tasks. I
recommend adding the `nurfile` to git, while `nurfile.local` should be ignored. This allows
each developer to have their own additional set of tasks.

### Some basics that `nur` provides

* `nur` tasks will always be run inside the directory the file `nurfile` was found in. If you 
  place a `nurfile` in your project root (git root) you will be able to call tasks from anywhere
  inside the project.
* `nur` will provide the internal state and config in the variable `$nur`, containing:
  - `$nur.run-path`: The path `nur` was executed in
  - `$nur.project-path`: The path `nur` executes the tasks in, this means the path the `nurfile` was found
  - `$nur.task-name`: The name of the task being executed, if any

### Adding some arguments to your tasks

I highly recommend reading `nu` [custom commands](https://www.nushell.sh/book/custom_commands.html) for details, but I will try to show you the
most important bits right here. I will use the term "`nur` tasks" to talk about "`nu` (sub)commands" in
the following section.

`nur` tasks can receive three different kinds of arguments:
* Named, positional arguments: `def "nur taskname" [argument1, argument2] { ... }`
  - Adding a `?` after the parameter name makes it optional
  - Above example provides the variables `$argument1` and `$argument2` in the task
* Flags as parameters: `def "nur taskname" [--argument1: string, --argument-number2: int] { ... }`
  - If you want to have named flags that can actually receive any values, you need to add a type
    (see below for typing)
  - Flags are always optional, default value will be `null` unless defined otherwise
    (see below for default values)
  - Flags will provide variables names without the leading `--`
  - Flags will be available in your task code as variables with all `-` replaced by `_`
  - Above example provides the variables `$argument1` and `$argument_number2` in the task
* Boolean flags: `def "nur taskname" [--switch] { ... }`
  - Boolean flags must NOT be typed
  - Those can only receive the values `true`/`false`, with `false` being the default
  - Above example provides the variable `$switch` in the task
* Rest parameters might consume the rest of the arguments: `def "nur taskname" [...rest] { ... }`
  - Above example provides the variable `$rest` in the task

Arguments can (and should) be typed, you can use `argument_name: type` for doing so. A typed
argument could look like this:  
`def "nur taskname" [argument1: string, argument2: int] { ... }`  
(see [parameter types](https://www.nushell.sh/book/custom_commands.html#parameter-types) for a full list of available types)

Also arguments can have a default value, you can use `argument_name = "value"` to set the default value.
An example using a default value could look like this:  
`def "nur taskname" [argument1 = "value", argument2 = 10] { ... }`

### Adding docs to your command

You may add docs by adding commands to your `nur` tasks. See the usage example above and
the `nu` [command documentation](https://www.nushell.sh/book/custom_commands.html#documenting-your-command) section.

Basic rule is that the commend right above your task will be used as a description for that task.
Comments next to any argument will be used to document that argument.

### Calling system commands from `nur`

If you want to run external commands you might run into the issue that `nu` itself provides some
[builtin commands](https://www.nushell.sh/commands/) that might match the name of the command
you want to run. This for example is the case for `sort`, where `nu` has it's own version (see
[sort command](https://www.nushell.sh/commands/docs/sort.html)). Most of the time it makes sense
to use the versions `nu` provides as those implement all the [pipeline improvements](https://www.nushell.sh/book/pipelines.html) of `nu`.
If you want to call the external command and not the builton function by `nu` use `^sort` instead
of `sort` in your `nur` tasks.

The same rule applies to your user defined functions, you would for example provide a function
named `grep` (`def grep [] { ... }`) which could call the `grep` command using `^grep`.

### Provide `nur` tasks for running normal shell commands

If you want to use a `nur` to run any normal command - for example to ensure you can run this in
any subdirectory of your project - I recommend using the following schema (using the `poetry`
package manager as an example):

```
def --wrapped "nur poetry" [...args] {
    poetry ...$args
}
```

The important bit is using `--wrapped`, so the `nu` parser will not try to match flags starting with
`-` into your `nur` task.

See the [docs for def](https://www.nushell.sh/commands/docs/def.html) for some more details.

## Some notes about pipelines and how `nu` handles those

Normal UNIX shells always use text to pass data from `stdout` (or `stderr`) to the next command via
`stdin`. This is pretty easy to implement and a very slim contract to follow. `nu` however works quite
different from this. Instead of passing test when using pipelines it tried to use structured data -
think of this like passing JSON between the different command. This increases the flexibility and
structured way to work with the data in a great way.

For example getting the ID of a running container in docker would look somewhat like this in a normal
UNIX shell:  
`docker ps | grep some-name | head -n 1 | awk '{print $1}'`

This works for most of the cases, but might produce errors for example of a container named
`this-also-contains-some-name-in-its-name` exists. This issue exists as we are parsing
text data, not some actual structured data. So having the name anywhere in a line will result in
that line being used. (Note: I know about `docker ps --filter ...`, this is just to explain the
overall issue of parsing text data)

`nu` works on structured data and provides commands to filter, sort or restructure that data in
any way you like. Also `nu` provides mechanics to import text data into this structured format.
Getting the `docker ps` text data input `nu` can for example be done using `docker ps | from ssv`
("ssv" stands for "space-separated values"), see the [command `from`](https://www.nushell.sh/commands/docs/from.html)
for more possible input formats.

To get the first container matching using the image `some-name` you could use this command:  
`docker ps | from ssv | where IMAGE == "some-name" | get "CONTAINER ID" | first`

This is using the [where command](https://www.nushell.sh/commands/docs/where.html) to match only
a single row and then the [get command](https://www.nushell.sh/commands/docs/get.html) to reduce the
row to just one column. There are also many more commands to work with structured data.

This way of working with command data in a very structured form is very much superior to
how normal shells used to work. This is especially good when you are creating more complex
scripts and thus also true for the tasks you will write in your task runner. This is why I did
choose `nu` for creating `nur`.

I recommend reading [thinking in nu](https://www.nushell.sh/book/thinking_in_nu.html#nushell-isn-t-bash) to
get a grasp about this concept and start using `nu` script in `nur` in a very structured way.

## Why `nur` + some history

For me `nur` is the next logical step after I created `b5`. `b5` is based on running bash code and
allowing users to do this in a somewhat ordered matter. Initially `b5` even was just some bash script,
but then eventually I figured bash is just not enough to handle my requirements. So I switched to
using Python, but `b5` was still based on bash, as it would generate bash code and then just execute
the code. One issue I always had with this approach was that again bash isn't that nice to write
complex things without introducing issues everywhere. Look for example at parameter handling.

Then along came `just`, which did implement its own language you could use to write your `justfile`.
This language was inspired by what a `Makefile` would look like, still without the issues `Makefile`'s
impose when using those as your task runner. Also, it did include a very nice way to define task arguments,
parse those, care about validation etc. Still the way `just` works is either to execute the task line
by line (and not having any context between those commands) or define some script language to execute
the full command (meaning using something like bash again). So `just` - at least for me - is a great
step forward, but still not what I had in mind when creating `b5` and what I would like to do with a
task runner.

Then I came across `nu`, especially the nu shell. This did become my default shell after a while, and
I am using it as of now. `nu` feels nicely designed, has a very structured way to execute commands and
also handle their "response" data (stdout/err) - as everything is structured data there. This is way
better than the original UNIX approach of always passing text data. Also `nu` allows you to have simple
functions, that - as with `just` - handle argument parsing for you. So this did look like the perfect
combination for something like a task runner.

Of course, you could just define some `nu` functions to completely create a task runner and that would
already be better than `b5` or `just`. But this would also mean that every dev using this task runner
would need to switch to `nu` first. So I decided to try the hard route and create my own rust based
cli tool that would parse a `nu` script and then execute tasks defined in this script.

This is what you are seeing here. `nur` will load the `nurfile` defined in your project directory and
then allows you to execute tasks from this file. As it is its own binary you can easily use `nur` from
bash, zsh and possibly even PowerShell - whatever you prefer. Still you will be able to have the `nu`
superpowers inside your defined tasks.

## About the name

`nur` stands for "nu run". Basically it should be "nu run task", which would lead to "nurt" - but then I
decided for just "nur" as:
* `nur` is very fast to type (one less character 💪)
* `nur` is the reverse of `run`, which I like as a side effect 🥳
* and then as a nice and also weird side effect: You could translate "just" to "nur" in german 😂

## Contributing

If you want to contribute to this project, feel free to just fork the project, create a dev
branch in your fork and then create a pull request (PR). If you are unsure about whether
your changes really suit the project please create an issue first, to talk about this.
