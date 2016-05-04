gtdtxt
======

> CLI app to parse a human-readable text file for managing GTD workflow.

![](screenshot.png)

```
gtdtxt v0.6.0 (semver.org)
Alberto Leal <mailforalberto@gmail.com> (github.com/dashed)
CLI app to parse a human-readable text file for managing GTD workflow

USAGE:
    gtdtxt [FLAGS] [OPTIONS] <path to gtdtxt file>

FLAGS:
    -h, --help                        Prints help information
    -F, --hide-flagged                Hide flagged tasks.
    -I, --hide-incomplete             Hide incomplete tasks.
    -n, --hide-nonproject-tasks       Hide tasks not belonging to a project.
    -o, --hide-overdue                Hide overdue tasks.
    -r, --reveal-deferred             Reveal deferred tasks.
    -d, --show-done                   Show completed tasks.
    -i, --show-incubate               Show incubated tasks.
    -f, --show-only-flagged           Show only flagged tasks.
    -z, --sort-overdue-by-priority    Sort overdue tasks by priority.
    -q, --validate                    Validate file and suppress any output.
    -V, --version                     Prints version information

OPTIONS:
    -w, --due-within <due-within>
        Display tasks due within a time duration.
        Example: 2 days 4 hrs
    -c, --filter-by-context <filter-by-context>
        Filter using given context or list of comma separated contexts.
        Example: phone, computer, internet connection, office
    -p, --filter-by-project <filter-by-project>
        Filter using given project path.
        Example: path / to / project
    -t, --filter-by-tag <filter-by-tag>
        Filter using given tag or list of comma separated tags.
        Example: chore, art, to watch

ARGS:
    <path to gtdtxt file>    Path to gtdtxt file.

```


Inspired by [todotxt.com](http://todotxt.com/) and [ledger-cli.org](http://ledger-cli.org/).

## Install

Direct downloads are available through the [releases page](https://github.com/gtdtxt/gtdtxt/releases).

If you're on OSX, you may install using [Homebrew](http://brew.sh/):

```
brew install https://raw.githubusercontent.com/gtdtxt/gtdtxt/master/gtdtxt.rb
```

## Examples

- [reference-todo.gtd](./reference-todo.gtd)
- [examples/todo.gtd](./examples/todo.gtd)


## Issues, Questions, Comments, etc?

Feel free to open an issue within this GitHub repository: https://github.com/gtdtxt/gtdtxt/issues

License
=======

MIT
