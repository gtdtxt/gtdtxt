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
gtdtxt v0.12.0 (semver.org 2.0)
Alberto Leal <mailforalberto@gmail.com> (github.com/dashed)
Getting Things Done (GTD) command-line application that parses human-readable to-do list text files.

USAGE:
    gtdtxt [FLAGS] [OPTIONS] <path to gtdtxt file> [SUBCOMMAND]

FLAGS:
    -h, --help                                Prints help information
    -x, --hide-by-default                     Hide tasks by default. Usage of flags / options are necessary to display
                                              tasks.
    -F, --hide-flagged                        Hide flagged tasks.
    -u, --hide-headers                        Hide headers. Shown by default.
    -I, --hide-incomplete                     Hide incomplete tasks.
    -n, --hide-nonproject-tasks               Hide tasks not belonging to a project.
        --hide-notes                          Hide notes of tasks. Notes are shown by default.
    -o, --hide-overdue                        Hide overdue tasks.
    -r, --show-deferred                       Reveal deferred tasks.
    -d, --show-done                           Show completed tasks.
    -e, --show-flagged                        Show flagged tasks. Used with --hide-by-default
    -b, --show-incomplete                     Show incomplete tasks. Used with --hide-by-default
    -i, --show-incubate                       Show incubated tasks.
        --show-line-num-with-file-location    Show line number location of task with file location path.
                                              Example:
                                              /path/to/file:line_number
                                              
    -g, --show-nonproject-tasks               Show tasks that are not in a project. Used with --hide-by-default
    -f, --show-only-flagged                   Show only flagged tasks.
    -a, --show-overdue                        Show overdue tasks. Used with --hide-by-default
    -j, --show-project-tasks                  Show tasks that are not in a project. Used with --hide-by-default
    -z, --sort-overdue-by-priority            Sort overdue tasks by priority. By default overdue tasks are shown from
                                              oldest due to recently due.
    -q, --validate                            Validate file and suppress any output.
    -V, --version                             Prints version information

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
        Format of filter: <operator><priority>
        <priority> is a signed integer.
        There
        may be whitespace between <operator> and <priority>.
        Operators: <=, <, >=, >, =, ==
        You may combine
        filters with: and, &, &&, or, |, ||
        You may wrap filter expressions in parentheses.
        
        Example: >= 42
        (show tasks greater or equal to 42)
        
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


Specification
=============

*TBA*

## Tasks

A task is a block of attributes with nothing between them but attributes (e.g. no blank lines, lines of whitespace, comments, or horizontal rules).
On the other hand, tasks are separated by anything other than attributes.

### Task attributes

Task attributes are key-value (or attribute-value) pairs. 
The key and the value are separated by a colon, `:`, and they have this form: `attribute_name: value`.

Tasks may have attributes that repeats (which may be useful). But in the case when the task can only have one value of an attribute type, then the last attribute that appears in the task block will be the final value. For example:

```
task: buy milk
task: buy oranges
```

The above task will have the title 'buy oranges', since `task: buy oranges` appeared last in its task block.

Task attribute names are **case-insensitive**.

#### `task` attribute (required)

The task name/title.

**Example:**

```
task: buy milk
```

**Aliases:**

- `title`, `todo`, `action`, `item`

#### `status` attribute (optional)

*TBA*

## Directives

Directives are flags/options that are applied to tasks, or apply an operation (e.g. including tasks from a file). 
Some directives are sometimes useful such as enforcing checks on tasks, or injecting default attribute values on tasks.

### `include` directive

Include tasks from file indicated by the given path. 

Paths may be absolute or relative; and this convention depends on the operating system. 
If the given path is relative, then it is relative to the file's directory of which this directive is contained in.

**Usage:**

`include: path/to/file.gtd`


### `default` directives

`default` directives apply default values to tasks appearing after those directives. Attribute values in tasks are not *overwritten*.
These directives are only applied to tasks that appear in the **same file**.

#### `default:status`

Apply default `status` attribute value to tasks appearing after this directive.
Value is only applied if those tasks do not have explicit `status` attribute.

**Usage:**

`default:status: incubate`

* If value is any one of the `Status` values, then this directive adds `status` attribute with the same value to any tasks that does not have an explicit `status` attribute. 

**Values:**

- **Status:** 
    - **Done:** done, complete, finished, finish, fin
    - **Not Done:** not done, active, progress, in progress, in-progress, pending, is active
    - **Incubate:** incubate, hide, hidden, later, someday, inactive, not active

### `require` directives

`require` directives enforce attribute values to tasks appearing after those directives. It is an error if any of those tasks are missing the required attribute, or have the incorrect attribute value.

#### `require:status`

Require tasks appearing after this directive to **have** this `status` attribute value.

**Usage:**

`require:status: incubate`


* If value is one of `True` values, then any tasks appearing after this `require:status` directive shall be required to have an **explicit** `status` attribute.

* If value is one of `False` values, then this directive has no effect (e.g. no-op).

* If value is any one of the `Status` values, then this directive enforces any tasks appearing after this directive to have the same/similar `status` value.

**Values:**

- **Boolean:** 
    - **True:** yes, y, true
    - **False:** no, n, false
- **Status:** 
    - **Done:** done, complete, finished, finish, fin
    - **Not Done:** not done, active, progress, in progress, in-progress, pending, is active
    - **Incubate:** incubate, hide, hidden, later, someday, inactive, not active

#### `require:exclude:status`

Require tasks appearing after this directive to **not have** this `status` attribute value.

**Usage:**

`require:status: done`

*TBA*

#### `require:project:prefix`

*TBA*

#### `require:project:`

*TBA*

### `inject` directives

*TBA*

#### `inject:project:prefix`

*TBA*

### `ensure` directives

*TBA*

#### `ensure:project:prefix`

*TBA*

## Comments

*TBA*

## Horizontal Rule

*TBA*

Issues, Questions, Comments, etc?
=================================

Feel free to open an issue within this GitHub repository: https://github.com/gtdtxt/gtdtxt/issues

License
=======

MIT.
