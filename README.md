gtdtxt
======

> Getting Things Done (GTD) command-line application that parses human-readable to-do list text files.

![](screenshot.png)

gtdtxt is a command-line application that parses a to-do list in the form of human-readable text files.
It is designed for managing (Getting Things Done) workflow.

For a pragmatic introduction to GTD, see:

- https://hamberg.no/gtd/


[todotxt.com](http://todotxt.com/) and [ledger-cli.org](http://ledger-cli.org/) influenced the creation of gtdtxt.



## Usage

```
gtdtxt v0.10.0 (semver.org)
Alberto Leal <mailforalberto@gmail.com> (github.com/dashed)
Getting Things Done (GTD) command-line application that parses human-readable to-do list text files.

USAGE:
    gtdtxt [FLAGS] [OPTIONS] <path to gtdtxt file> [SUBCOMMAND]

FLAGS:
    -h, --help                        Prints help information
    -x, --hide-by-default             Hide tasks by default. Usage of flags / options are necessary to display tasks.
    -F, --hide-flagged                Hide flagged tasks.
    -u, --hide-headers                Hide headers. Shown by default.
    -I, --hide-incomplete             Hide incomplete tasks.
    -n, --hide-nonproject-tasks       Hide tasks not belonging to a project.
    -o, --hide-overdue                Hide overdue tasks.
    -r, --show-deferred               Reveal deferred tasks.
    -d, --show-done                   Show completed tasks.
    -e, --show-flagged                Show flagged tasks. Used with --hide-by-default
    -b, --show-incomplete             Show incomplete tasks. Used with --hide-by-default
    -i, --show-incubate               Show incubated tasks.
    -g, --show-nonproject-tasks       Show tasks that are not in a project. Used with --hide-by-default
    -f, --show-only-flagged           Show only flagged tasks.
    -a, --show-overdue                Show overdue tasks. Used with --hide-by-default
    -j, --show-project-tasks          Show tasks that are not in a project. Used with --hide-by-default
    -z, --sort-overdue-by-priority    Sort overdue tasks by priority. By default overdue tasks are shown from oldest
                                      due to recently due.
    -q, --validate                    Validate file and suppress any output.
    -V, --version                     Prints version information

OPTIONS:
    -w, --due-within <due-within>
        Display tasks due within a time duration.
        Example: 2 days 4 hrs
        
    -c, --only-with-context <only-with-context>
        Show only tasks that have any given list of comma separated contexts.
        Example: phone, computer, internet
        connection, office
        
    -p, --only-with-project <only-with-project>
        Show only tasks with given project path.
        Example: path / to / project
        
    -t, --only-with-tag <only-with-tag>
        Show only tasks that have any given list of comma separated tags.
        Example: chore, art, to watch
        
    -y, --show-priority <show-priority>
        Filter tasks by priority.
        Format: <operator><priority>
        <priority> is a signed integer.
        There may be
        whitespace between <operator> and <priority>.
        Operators: <=, <, >=, >, =, ==
        
        Example: >= 42 (show tasks
        greater or equal to 42)
        
    -s, --show-with-context <show-with-context>
        Show tasks with given list of comma separated contexts.
        Used with --hide-by-default
        Example: phone,
        computer, internet connection, office
        
    -k, --show-with-project <show-with-project>
        Show tasks with given project path.Used with --hide-by-default
        Example: path / to / project
        
    -m, --show-with-tag <show-with-tag>
        Show tasks with given list of comma separated tags.Used with --hide-by-default
        Example: chore, art, to watch

ARGS:
    <path to gtdtxt file>    Path to gtdtxt file.

SUBCOMMANDS:
    current    Display current task
    help       Prints this message or the help of the given subcommand(s)
    stats      Display statistics

```

## Install

Direct downloads are available through the [releases page](https://github.com/gtdtxt/gtdtxt/releases).

### OSX

If you're on OSX, you may install using [Homebrew](http://brew.sh/):

```
brew install https://raw.githubusercontent.com/gtdtxt/gtdtxt/master/gtdtxt.rb
```

## Examples

- [reference-todo.gtd](./reference-todo.gtd)
- [examples/todo.gtd](./examples/todo.gtd)

### Tutorial

Create an empty text file. Example: `todo.gtd`

*TBA*


## Issues, Questions, Comments, etc?

Feel free to open an issue within this GitHub repository: https://github.com/gtdtxt/gtdtxt/issues

License
=======

MIT.
