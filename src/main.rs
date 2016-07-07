#![recursion_limit="100"]
// Above is used for chomp macros

#[macro_use]
extern crate version;

#[macro_use]
extern crate debug_unreachable;

#[macro_use]
extern crate chomp;
extern crate chrono;
extern crate colored;
extern crate clap;


use std::path::{Path, PathBuf};
use std::fs::{File};
use std::collections::{HashMap, HashSet, BTreeMap};
use std::ascii::{AsciiExt};
use std::env;
use std::process;
use std::marker::PhantomData;


use colored::*;

use clap::{Arg, App, SubCommand, AppSettings};

// use chrono::*;
use chrono::offset::local::Local;
use chrono::naive::datetime::NaiveDateTime;
use chrono::naive::date::NaiveDate;
use chrono::naive::time::NaiveTime;
use chrono::duration::Duration;

use chomp::{SimpleResult, Error, ParseResult};
use chomp::primitives::{InputBuffer};
use chomp::{Input, U8Result, parse_only};
use chomp::buffer::{Source, Stream, StreamError};

use chomp::{token};
use chomp::parsers::{string, eof, any, satisfy};
use chomp::combinators::{or, many_till, many, many1, skip_many, skip_many1, look_ahead};
use chomp::ascii::{is_whitespace, decimal, digit};
// use chomp::*;


fn main() {

    let version: &str = &format!("v{} (semver.org)", version!());

    let cmd_matches = App::new("gtdtxt")
        .setting(AppSettings::ArgRequiredElseHelp)
        .setting(AppSettings::GlobalVersion)
        .version(version) // semver semantics
        .about("Getting Things Done (GTD) command-line application that parses human-readable to-do list text files.")
        .author("Alberto Leal <mailforalberto@gmail.com> (github.com/dashed)")
        .arg(
            Arg::with_name("hide-headers")
            .help("Hide headers. Shown by default.")
            .short("u")
            .long("hide-headers")
            .required(false)
        )
        .arg(
            Arg::with_name("due-within")
            .next_line_help(true)
            .help("Display tasks due within a time duration.{n}\
                Example: 2 days 4 hrs{n}")
            .short("w")
            .long("due-within")
            .required(false)
            .takes_value(true)
            .multiple(false)
        )
        .arg(
            Arg::with_name("hide-by-default")
            .help("Hide tasks by default. Usage of flags / options are necessary to display tasks.")
            .short("x")
            .long("hide-by-default")
            .required(false)
        )
        .arg(
            Arg::with_name("show-overdue")
            .help("Show overdue tasks. Used with --hide-by-default")
            .short("a")
            .long("show-overdue")
            .required(false)
        )
        .arg(
            Arg::with_name("show-incomplete")
            .help("Show incomplete tasks. Used with --hide-by-default")
            .short("b")
            .long("show-incomplete")
            .required(false)
        )
        .arg(
            Arg::with_name("show-flagged")
            .help("Show flagged tasks. Used with --hide-by-default")
            .short("e")
            .long("show-flagged")
            .required(false)
        )
        .arg(
            Arg::with_name("show-nonproject-tasks")
            .help("Show tasks that are not in a project. Used with --hide-by-default")
            .short("g")
            .long("show-nonproject-tasks")
            .required(false)
        )
        .arg(
            Arg::with_name("show-project-tasks")
            .help("Show tasks that are not in a project. Used with --hide-by-default")
            .short("j")
            .long("show-project-tasks")
            .required(false)
        )
        .arg(
            Arg::with_name("hide-overdue")
            .help("Hide overdue tasks.")
            .short("o")
            .long("hide-overdue")
            .required(false)
        )
        .arg(
            Arg::with_name("show-done")
            .help("Show completed tasks.")
            .short("d")
            .long("show-done")
            .required(false)
        )
        .arg(
            Arg::with_name("show-deferred")
            .help("Reveal deferred tasks.")
            .short("r")
            .long("show-deferred")
            .required(false)
        )
        .arg(
            Arg::with_name("show-incubate")
            .help("Show incubated tasks.")
            .short("i")
            .long("show-incubate")
            .required(false)
        )
        .arg(
            Arg::with_name("hide-incomplete")
            .help("Hide incomplete tasks.")
            .short("I")
            .long("hide-incomplete")
            .required(false)
        )
        .arg(
            Arg::with_name("validate")
            .help("Validate file and suppress any output.")
            .short("q")
            .long("validate")
            .required(false)
        )
        .arg(
            Arg::with_name("hide-nonproject-tasks")
            .help("Hide tasks not belonging to a project.")
            .short("n")
            .long("hide-nonproject-tasks")
            .required(false)
        )
        .arg(
            Arg::with_name("show-only-flagged")
            .help("Show only flagged tasks.")
            .short("f")
            .long("show-only-flagged")
            .required(false)
        )
        .arg(
            Arg::with_name("hide-flagged")
            .help("Hide flagged tasks.")
            .short("F")
            .long("hide-flagged")
            .required(false)
        )
        .arg(
            Arg::with_name("sort-overdue-by-priority")
            .help("Sort overdue tasks by priority. By default overdue tasks are shown from oldest due to recently due.")
            .short("z")
            .long("sort-overdue-by-priority")
            .required(false)
        )
        .arg(
            Arg::with_name("only-with-project")
            .next_line_help(true)
            .help("Show only tasks with given project path.{n}\
                Example: path / to / project{n}")
            .short("p")
            .long("only-with-project")
            .required(false)
            .takes_value(true)
            .multiple(true)
            .validator(|path| {
                let path = path.trim();
                if path.len() <= 0 {
                    return Err(String::from("invalid project path"));
                }
                return Ok(());
            })
        )
        .arg(
            Arg::with_name("show-with-project")
            .next_line_help(true)
            .help("Show tasks with given project path.\
                Used with --hide-by-default{n}\
                Example: path / to / project{n}")
            .short("k")
            .long("show-with-project")
            .required(false)
            .takes_value(true)
            .multiple(true)
            .validator(|path| {
                let path = path.trim();
                if path.len() <= 0 {
                    return Err(String::from("invalid project path"));
                }
                return Ok(());
            })
        )
        .arg(
            Arg::with_name("only-with-tag")
            .next_line_help(true)
            .help("Show only tasks that have any given list of comma separated tags.{n}\
                Example: chore, art, to watch{n}")
            .short("t")
            .long("only-with-tag")
            .required(false)
            .takes_value(true)
            .multiple(true)
            .validator(|tag| {
                let tag = tag.trim();
                if tag.len() <= 0 {
                    return Err(String::from("invalid tag"));
                }
                return Ok(());
            })
        )
        .arg(
            Arg::with_name("show-with-tag")
            .next_line_help(true)
            .help("Show tasks with given list of comma separated tags.\
                Used with --hide-by-default{n}\
                Example: chore, art, to watch")
            .short("m")
            .long("show-with-tag")
            .required(false)
            .takes_value(true)
            .multiple(true)
            .validator(|tag| {
                let tag = tag.trim();
                if tag.len() <= 0 {
                    return Err(String::from("invalid tag"));
                }
                return Ok(());
            })
        )
        .arg(
            Arg::with_name("only-with-context")
            .next_line_help(true)
            .help("Show only tasks that have any given list of comma separated contexts.{n}\
                Example: phone, computer, internet connection, office{n}")
            .short("c")
            .long("only-with-context")
            .required(false)
            .takes_value(true)
            .multiple(true)
            .validator(|context| {
                let context = context.trim();
                if context.len() <= 0 {
                    return Err(String::from("invalid context"));
                }
                return Ok(());
            })
        )
        .arg(
            Arg::with_name("show-with-context")
            .next_line_help(true)
            .help("Show tasks with given list of comma separated contexts.{n}\
                Used with --hide-by-default{n}\
                Example: phone, computer, internet connection, office{n}")
            .short("s")
            .long("show-with-context")
            .required(false)
            .takes_value(true)
            .multiple(true)
            .validator(|tag| {
                let tag = tag.trim();
                if tag.len() <= 0 {
                    return Err(String::from("invalid context"));
                }
                return Ok(());
            })
        )
        .arg(
            Arg::with_name("show-priority")
            .next_line_help(true)
            .use_delimiter(false)
            .help("Filter tasks by priority.{n}\
                Format of filter: <operator><priority>{n}\
                <priority> is a signed integer.{n}\
                There may be whitespace between <operator> and <priority>.{n}\
                Operators: <=, <, >=, >, =, =={n}\
                You may combine filters with: and, &, &&, or, |, ||{n}\
                You may wrap filter expressions in parentheses.{n}\
                {n}\
                Example: >= 42 (show tasks greater or equal to 42){n}")
            .short("y")
            .long("show-priority")
            .required(false)
            .takes_value(true)
            .multiple(false)
            .validator(|filter| {
                let filter = filter.trim();
                if filter.len() <= 0 {
                    return Err(String::from("invalid 'show-priority' filter"));
                }
                return Ok(());
            })
        )
        .arg(
            Arg::with_name("path to gtdtxt file")
            .help("Path to gtdtxt file.")
            .required(true)
            .index(1)
            .validator(|gtdtxt_file| {
                let gtdtxt_file = gtdtxt_file.trim();
                if gtdtxt_file.len() <= 0 {
                    return Err(String::from("invalid path to file"));
                } else {
                    return Ok(());
                }
            })
        )
        .subcommand(
            SubCommand::with_name("stats")
                .about("Display statistics")
        )
        .subcommand(
            SubCommand::with_name("current")
                .about("Display current task")
        ).get_matches();

    let path_to_file: String = cmd_matches.value_of("path to gtdtxt file")
                                                .unwrap()
                                                .trim()
                                                .to_string();

    let base_root = format!("{}", env::current_dir().unwrap().display());
    let mut journal = GTD::new(base_root);

    // priority range filter
    if let Some(show_priority) = cmd_matches.value_of("show-priority") {

        let show_priority = show_priority.trim();

        match parse_only(|i| parse_show_priority(i), show_priority.as_bytes()) {
            Ok(result) => {
                journal.filter_priority = Some(result);
            },
            Err(_) => {
                println!("Unable to parse value to option `--show-priority`: {}", show_priority);
                process::exit(1);
                // panic!("{:?}", e);
            }
        }
    }

    // due within filter
    if let Some(due_within) = cmd_matches.value_of("due-within") {

        let due_within = due_within.trim();

        match parse_only(|i| parse_times_ranges(i), due_within.as_bytes()) {
            Ok(result) => {
                journal.due_within = Duration::seconds(result as i64);
            },
            Err(_) => {
                println!("Unable to parse value to option `--due-within`: {}", due_within);
                process::exit(1);
                // panic!("{:?}", e);
            }
        }
    }

    // project path filters
    if let Some(project_paths) = cmd_matches.values_of("only-with-project") {
        for project_path in project_paths {

            match parse_only(|i| parse_string_lists(i, b'/'), project_path.as_bytes()) {
                Ok(mut result) => {
                    journal.add_project_only_filter(&mut result);
                },
                Err(_) => {
                    println!("Unable to parse project path `--only-with-project`: {}", project_path);
                    process::exit(1);
                    // panic!("{:?}", e);
                }
            }
        }
    }

    if let Some(project_paths) = cmd_matches.values_of("show-with-project") {
        for project_path in project_paths {

            match parse_only(|i| parse_string_lists(i, b'/'), project_path.as_bytes()) {
                Ok(mut result) => {
                    journal.add_project_whitelist(&mut result);
                },
                Err(_) => {
                    println!("Unable to parse project path `--show-with-project`: {}", project_path);
                    process::exit(1);
                    // panic!("{:?}", e);
                }
            }
        }
    }

    // tag filters
    if let Some(tags) = cmd_matches.values_of("only-with-tag") {

        for tag in tags {

            match parse_only(|i| parse_string_lists(i, b','), tag.as_bytes()) {
                Ok(result) => {

                    if result.len() > 0 {
                        journal.filter_by_only_tags = true;
                    }

                    journal.add_tag_only_filters(result);
                },
                Err(_) => {
                    println!("Unable to parse tags `--only-with-tag`: {}", tag);
                    process::exit(1);
                    // panic!("{:?}", e);
                }
            }
        }
    }

    if let Some(tags) = cmd_matches.values_of("show-with-tag") {

        for tag in tags {

            match parse_only(|i| parse_string_lists(i, b','), tag.as_bytes()) {
                Ok(result) => {

                    if result.len() > 0 {
                        journal.filter_by_include_tags = true;
                    }

                    journal.add_tag_include_filters(result);
                },
                Err(_) => {
                    println!("Unable to parse tags `--show-with-tag`: {}", tag);
                    process::exit(1);
                    // panic!("{:?}", e);
                }
            }
        }
    }

    // context filters
    if let Some(contexts) = cmd_matches.values_of("only-with-context") {

        for context in contexts {

            match parse_only(|i| parse_string_lists(i, b','), context.as_bytes()) {
                Ok(result) => {

                    if result.len() > 0 {
                        journal.filter_by_only_contexts = true;
                    }

                    journal.add_context_only_filters(result);
                },
                Err(_) => {
                    println!("Unable to parse contexts `--only-with-context`: {}", context);
                    process::exit(1);
                    // panic!("{:?}", e);
                }
            }
        }
    }

    if let Some(contexts) = cmd_matches.values_of("show-with-context") {

        for context in contexts {

            match parse_only(|i| parse_string_lists(i, b','), context.as_bytes()) {
                Ok(result) => {

                    if result.len() > 0 {
                        journal.filter_by_include_contexts = true;
                    }

                    journal.add_context_include_filters(result);
                },
                Err(_) => {
                    println!("Unable to parse contexts `--show-with-context`: {}", context);
                    process::exit(1);
                    // panic!("{:?}", e);
                }
            }
        }
    }

    // flags

    let show_headers: bool = !cmd_matches.is_present("hide-headers");

    journal.sort_overdue_by_priority = cmd_matches.is_present("sort-overdue-by-priority");
    journal.hide_flagged = cmd_matches.is_present("hide-flagged");
    journal.show_only_flagged = cmd_matches.is_present("show-only-flagged");
    journal.show_done = cmd_matches.is_present("show-done");
    journal.show_incubate = cmd_matches.is_present("show-incubate");
    journal.show_deferred = cmd_matches.is_present("show-deferred");
    journal.hide_overdue = cmd_matches.is_present("hide-overdue");
    journal.hide_nonproject_tasks = cmd_matches.is_present("hide-nonproject-tasks");
    journal.hide_incomplete = cmd_matches.is_present("hide-incomplete");

    journal.hide_tasks_by_default = cmd_matches.is_present("hide-by-default");
    journal.show_overdue = cmd_matches.is_present("show-overdue");
    journal.show_incomplete = cmd_matches.is_present("show-incomplete");
    journal.show_flagged = cmd_matches.is_present("show-flagged");
    journal.show_nonproject_tasks = cmd_matches.is_present("show-nonproject-tasks");
    journal.show_project_tasks = cmd_matches.is_present("show-project-tasks");


    parse_file(None, path_to_file.clone(), &mut journal);
    let journal: GTD = journal;

    if cmd_matches.is_present("validate") {
        println!("{:>20} {}", "Tasks found".purple(), format!("{}", journal.tasks.len()).bold().purple());

        println!("File(s) validated.");

        return;
    }

    if let Some(_matches) = cmd_matches.subcommand_matches("current") {

        match journal.current_task {
            None => {

                println!("No current task found.");

            },
            Some(task_id) => {

                let task: &Task = journal.tasks.get(&task_id).unwrap();
                print_task(&journal, task);
            }
        };

        return;

    } else if let Some(_matches) = cmd_matches.subcommand_matches("stats") {

        println!("{}", "Statistics by file".bold().purple().underline());
        println!("");

        if journal.file_stats.len() <= 0 {
            println!("No files parsed.");
            return;
        }

        let mut print_line: bool = false;

        // for (path, file_stats) in journal.file_stats {
        for path in &journal.file_stats_stack {

            let file_stats = journal.file_stats.get(path).unwrap();

            if print_line {
                println!("");
            } else {
                print_line = true;
            }

            let path = match Path::new(&path).strip_prefix(&journal.base_root) {
                Err(_) => {
                    format!("{}", path)
                },
                Ok(path) => {
                    format!("./{}", path.display())
                }
            };

            println!("{:>11} {}",
                "Path:".bold().blue(),
                path);

            let total = file_stats.overdue_tasks.len() +
                file_stats.inbox_tasks.len() +
                file_stats.incubate_tasks.len() +
                file_stats.deferred_tasks.len() +
                file_stats.completed_tasks.len();

            if total > 0 {
                println!("{:>11} {}",
                    "Total:".bold().blue(),
                    total);
            }

            if file_stats.overdue_tasks.len() > 0 {
                println!("{:>11} {}",
                    "Overdue:".bold().blue(),
                    file_stats.overdue_tasks.len());
            }

            if file_stats.inbox_tasks.len() > 0 {
                println!("{:>11} {}",
                    "Inbox:".bold().blue(),
                    file_stats.inbox_tasks.len());
            }

            if file_stats.incubate_tasks.len() > 0 {
                println!("{:>11} {}",
                    "Incubated:".bold().blue(),
                    file_stats.incubate_tasks.len());
            }

            if file_stats.deferred_tasks.len() > 0 {
                println!("{:>11} {}",
                    "Deferred:".bold().blue(),
                    file_stats.deferred_tasks.len());
            }

            if file_stats.completed_tasks.len() > 0 {
                println!("{:>11} {}",
                    "Completed:".bold().blue(),
                    file_stats.completed_tasks.len());
            }

            if file_stats.have_tags() {
                println!("{:>11} {}",
                    "Tags:".bold().blue(),
                    file_stats.print_tags());
            }

            if file_stats.have_contexts() {
                println!("{:>11} {}",
                    "Contexts:".bold().blue(),
                    file_stats.print_contexts());
            }

            if file_stats.have_projects() {
                let mut first = true;
                for path in &file_stats.project_paths {

                    if first {
                        first = false;
                        println!("{:>11} {}",
                            "Projects:".bold().blue(), path);
                        continue;
                    }

                    println!("{:>11} {}", "", path);
                }
            }

        }

        return;
    }

    // Display tasks

    let mut display_divider = false;

    if journal.filter_priority.is_some() {

        let tree_art = priority_pretty_tree_art(journal.filter_priority.as_ref().unwrap());

        println!("{:>11} {} {}",
            "",
            "Filtering tasks by priority".bold().white(),
            tree_art
        );

        display_divider = true;
    }

    if journal.due_within.num_seconds() > 0 {

        println!("{:>11} {} {}",
            "",
            "Displaying tasks due within".bold().white(),
            Timerange::new(journal.due_within.num_seconds() as u64).print(10).white().bold()
        );

        display_divider = true;
    }

    if journal.show_only_flagged {

        println!("{:>11} {}",
            "",
            "Displaying only flagged tasks.".bold().white()
        );
        display_divider = true;

    } else if journal.hide_flagged {

        println!("{:>11} {}",
            "",
            "Hiding flagged tasks.".bold().white()
        );
        display_divider = true;
    }

    if display_divider {
        println!("");
    }


    let mut print_line: bool = false;
    let mut num_displayed = 0;
    let num_overdue;
    let num_inbox;
    let num_deferred;
    let num_done;


    // display tasks that are overdue
    let mut header_display: bool = show_headers;
    num_overdue = count_tasks(&journal.overdue);
    for (_, bucket) in journal.overdue.iter() {

        if bucket.len() <= 0 {
            continue;
        }

        if !journal.hide_overdue {

            if print_line {
                println!("");
            }

            if header_display {
                header_display = false;
                println!("{}{}",
                    "Overdue".white().bold().underline(),
                    format!(" ({})", num_overdue).white().bold().underline());
                println!("");
            }

            num_displayed = num_displayed + print_vector_of_tasks(&journal, bucket);

            if !print_line && num_displayed > 0 {
                print_line = true;
            }
        }
    }

    // display inbox ordered by priority.
    // incubated tasks are not included
    let mut header_display: bool = show_headers;
    num_inbox = count_tasks(&journal.inbox);
    for (_, inbox) in journal.inbox.iter() {

        if inbox.len() <= 0 {
            continue;
        }

        if print_line {
            println!("");
        }

        if header_display {
            header_display = false;
            println!("{}{}",
                "Inbox".white().bold().underline(),
                format!(" ({})", num_inbox).white().bold().underline());
            println!("");
        }

        num_displayed = num_displayed + print_vector_of_tasks(&journal, inbox);

        if !print_line && num_displayed > 0 {
            print_line = true;
        }

    }

    // display deferred tasks ordered by priority
    let mut header_display: bool = show_headers;
    num_deferred = count_tasks(&journal.deferred);
    for (_, deferred) in journal.deferred.iter() {

        if deferred.len() <= 0 {
            continue;
        }

        if journal.show_deferred || journal.hide_tasks_by_default {

            if print_line {
                println!("");
            }

            if header_display {
                header_display = false;
                println!("{}{}",
                    "Deferred".white().bold().underline(),
                    format!(" ({})", num_deferred).white().bold().underline());
                println!("");
            }

            num_displayed = num_displayed + print_vector_of_tasks(&journal, deferred);

            if !print_line && num_displayed > 0 {
                print_line = true;
            }

        }

    }


    // display completed tasks
    let mut header_display: bool = show_headers;
    num_done = count_tasks(&journal.done);
    for (_, done) in journal.done.iter() {

        if done.len() <= 0 {
            continue;
        }

        if journal.show_done || journal.hide_tasks_by_default {

            if print_line {
                println!("");
            }

            if header_display {
                header_display = false;
                println!("{}{}",
                    "Done".white().bold().underline(),
                    format!(" ({})", num_done).white().bold().underline());
                println!("");
            }

            num_displayed = num_displayed + print_vector_of_tasks(&journal, done);

            if !print_line && num_displayed > 0 {
                print_line = true;
            }

        }
    }

    if num_displayed > 0 {
        println!("");
    }

    println!(" {}",
        "Tasks completed in the past week (tracked using `done:`)".purple().bold()
    );


    let mut days_ago = 0;
    loop {

        print!("{:>11} {}",
            format!("{} {}", days_ago, "days ago").purple(),
            format!("|").purple()
        );

        if days_ago >= 7 {
            break;
        }

        days_ago = days_ago + 1;

    }

    println!("");

    let mut days_ago = 0;
    loop {

        let items_num = match journal.pulse.get(&days_ago) {
            None => 0,
            Some(bucket) => {
                (*bucket).len()
            }
        };

        print!("{:>11} {}",
            format!("{}", items_num).bold().purple(),
            format!("|").purple()
        );

        if days_ago >= 7 {
            break;
        }

        days_ago = days_ago + 1;

    }

    println!("");
    println!("");

    println!("{:>20} {}",
        "Tasks overdue".purple(),
        format!("{}", num_overdue).bold().purple()
    );

    println!("{:>20} {}",
        "Tasks inbox".purple(),
        format!("{}", num_inbox).bold().purple()
    );

    println!("{:>20} {}",
        "Tasks deferred".purple(),
        format!("{}", num_deferred).bold().purple()
    );

    println!("{:>20} {}",
        "Tasks complete".purple(),
        format!("{}", num_done).bold().purple()
    );

    println!("{:>20} {}",
        "Tasks found".purple(),
        format!("{}", journal.tasks.len()).bold().purple()
    );

    println!("{:>20} {}",
        "Tasks not displayed".purple(),
        format!("{}", journal.tasks.len() as u64 - num_displayed).bold().purple()
    );

    println!("{:>20} {}",
        "Tasks displayed".purple(),
        format!("{}", num_displayed).bold().purple()
    );

    println!("{:>20} {}",
        "Executed at".purple(),
        format!("{}", Local::now().naive_local().format("%B %-d, %Y %-l:%M:%S %p")).purple()
    );

}

/* printers */

fn print_vector_of_tasks(journal: &GTD, inbox: &Vec<u64>) -> u64 {

    let mut print_line: bool = false;
    let mut num_displayed = 0;

    for task_id in inbox {

        if print_line {
            println!("");
        }

        let task: &Task = journal.tasks.get(task_id).unwrap();

        print_task(journal, task);
        num_displayed = num_displayed + 1;

        if !print_line {
            print_line = true;
        }

    }

    num_displayed
}

fn print_task(journal: &GTD, task: &Task) {
    _print_task(journal, task, true);
}

fn _print_task(journal: &GTD, task: &Task, require_title: bool) {

    if task.current {
        println!("{:>11} ",
            "CURRENT".bold().purple().italic()
        );
    }

    if task.flag && !journal.show_only_flagged {
            println!("{:>11} ",
                "Flagged".bold().yellow()
            );
    }

    match task.title {
        None => {

            if require_title {
                println!("Missing task title (i.e. `task: <title>`) in task block found {}",
                    task.debug_range_string()
                );
                println!("Captured:");
                _print_task(journal, task, false);
                process::exit(1);
            }

        },

        Some(ref title) => {
            println!("{:>11} {}", "Task:".blue().bold(), title);
        }
    }

    match task.status {
        None => {},
        Some(ref status) => {
            let status_string = match status {
                &Status::Done => {
                    "Done".green()
                },
                &Status::NotDone => {
                    "Not Done".red().bold()
                },
                &Status::Incubate => {
                    "Incubate".purple()
                }
            };
            println!("{:>11} {}", "Status:".bold().blue(), status_string);
        }
    }

    match task.created_at {
        None => {},
        Some(ref created_at) => {

            let rel_time = relative_time(created_at.timestamp(), Local::now().naive_local().timestamp());

            let rel_time = match rel_time {
                RelativeTime::Now(_, rel_time) => {
                    format!("({})", rel_time)
                },
                RelativeTime::Past(_, rel_time) => {
                    format!("({})", rel_time)
                },
                RelativeTime::Future(_, rel_time) => {
                    format!("({})", rel_time)
                }
            };

            println!("{:>11} {} {}",
                "Added at:".bold().blue(),
                created_at.format("%B %-d, %Y %-l:%M %p"),
                rel_time
            );
        }
    }

    match task.done_at {
        None => {},
        Some(ref done_at) => {

            let rel_time = relative_time(done_at.timestamp(), Local::now().naive_local().timestamp());

            let rel_time = match rel_time {
                RelativeTime::Now(_, rel_time) => {
                    format!("({})", rel_time)
                },
                RelativeTime::Past(_, rel_time) => {
                    format!("({})", rel_time)
                },
                RelativeTime::Future(_, rel_time) => {
                    format!("({})", rel_time)
                }
            };

            println!("{:>11} {} {}",
                "Done at:".bold().blue(),
                done_at.format("%B %-d, %Y %-l:%M %p"),
                rel_time
            );
        }
    }

    match task.defer {
        None => {},
        Some(ref defer) => {

            match defer {
                &Defer::Forever => {
                    println!("{:>11} {}",
                        "Defer till:".bold().blue(),
                        "Forever".bold().green()
                    );
                },
                &Defer::Until(defer_till) => {

                    let rel_time = relative_time(defer_till.timestamp(), Local::now().naive_local().timestamp());

                    let rel_time = match rel_time {
                        RelativeTime::Now(_, rel_time) => {
                            let rel_time = format!("({})", rel_time);
                            rel_time.red()
                        },
                        RelativeTime::Past(_, rel_time) => {
                            let rel_time = format!("({})", rel_time);
                            rel_time.bold().red()
                        },
                        RelativeTime::Future(_, rel_time) => {
                            let rel_time = format!("({})", rel_time);
                            rel_time.bold().green()
                        }
                    };

                    println!("{:>11} {} {}",
                        "Defer till:".bold().blue(),
                        defer_till.format("%B %-d, %Y %-l:%M %p"),
                        rel_time
                    );
                }
            }


        }
    }

    match task.due_at {
        None => {},
        Some(ref due_at) => {
            let rel_time = relative_time(due_at.timestamp(), Local::now().naive_local().timestamp());

            let rel_time = match rel_time {
                RelativeTime::Now(_, rel_time) => {
                    let rel_time = format!("({})", rel_time);
                    rel_time.red()
                },
                RelativeTime::Past(_, rel_time) => {
                    let rel_time = format!("({})", rel_time);
                    rel_time.bold().red()
                },
                RelativeTime::Future(_, rel_time) => {
                    let rel_time = format!("({})", rel_time);
                    rel_time.bold().green()
                }
            };

            println!("{:>11} {} {}",
                "Due at:".bold().blue(),
                due_at.format("%B %-d, %Y %-l:%M %p"),
                rel_time
            );
        }
    }

    match task.source_file {
        None => unsafe { debug_unreachable!() },
        Some(ref path) => {

            let path = match Path::new(path).strip_prefix(&journal.base_root) {
                Err(_) => {
                    format!("{}", path)
                },
                Ok(path) => {
                    format!("./{}", path.display())
                }
            };

            println!("{:>11} {}",
                "File:".bold().blue(),
                path
            );
        }
    };

    if task.task_block_range_start != task.task_block_range_end {
        println!("{:>11} Lines {} to {}",
            "Located:".bold().blue(),
            task.task_block_range_start,
            task.task_block_range_end
        );
    } else {
        println!("{:>11} Line {}",
            "Located:".bold().blue(),
            task.task_block_range_start
        );
    }

    match task.tags {
        None => {},
        Some(ref tags) => {
            println!("{:>11} {}",
                "Tags:".bold().blue(),
                tags.join(", ")
            );
        }
    }

    match task.contexts {
        None => {},
        Some(ref contexts) => {
            println!("{:>11} {}",
                "Contexts:".bold().blue(),
                contexts.join(", ")
            );
        }
    }

    match task.project {
        None => {},
        Some(ref project_path) => {
            println!("{:>11} {}",
                "Project:".bold().blue(),
                project_path.join(" / ")
            );
        }
    }

    if task.time > 0 {
        println!("{:>11} {}",
            "Time spent:".bold().blue(),
            Timerange::new(task.time).print(2)
        );

    }

    if task.has_chain() {
        let chain_at: NaiveDateTime = task.get_chain();

        let rel_time = relative_time(chain_at.timestamp(), Local::now().naive_local().timestamp());

        let rel_time = match rel_time {
            RelativeTime::Now(_, rel_time) => {
                let rel_time = format!("({})", rel_time);
                rel_time.red()
            },
            RelativeTime::Past(_, rel_time) => {
                let rel_time = format!("({})", rel_time);
                rel_time.bold().red()
            },
            RelativeTime::Future(_, rel_time) => {
                let rel_time = format!("({})", rel_time);
                rel_time.bold().green()
            }
        };

        println!("{:>11} {} {}",
            "Last chain:".bold().blue(),
            chain_at.format("%B %-d, %Y %-l:%M %p"),
            rel_time
        );
    }

    if task.priority != 0 {
        println!("{:>11} {}", "Priority:".bold().blue(), task.priority);
    }

    match task.note {
        None => {},
        Some(ref note) => {
            println!("{:>11} {}",
                "Notes:".bold().blue(),
                note
            );
        }
    }

}

/* data structures */

#[derive(Debug)]
enum NodeType {
    Node(Tree),
    Leaf
}

// index project filters
type Tree = HashMap<String, NodeType>;

#[derive(Debug)]
enum Status {
    Done,
    Incubate,
    NotDone
}

#[derive(Debug)]
struct Task {

    /* debug*/
    task_block_range_start: u64,
    task_block_range_end: u64,

    /* props */
    current: bool,
    title: Option<String>,
    note: Option<String>,
    created_at: Option<NaiveDateTime>,
    done_at: Option<NaiveDateTime>,
    chains: Option<BTreeMap<NaiveDateTime, bool>>,
    due_at: Option<NaiveDateTime>,
    defer: Option<Defer>,
    status: Option<Status>,
    project: Option<Vec<String>>,
    contexts: Option<Vec<String>>,
    tags: Option<Vec<String>>,
    priority: i64,
    time: u64,
    flag: bool,
    source_file: Option<String>
}

impl Task {

    fn new(task_block_range_start: u64) -> Task {
        Task {

            task_block_range_start: task_block_range_start,
            task_block_range_end: task_block_range_start,

            /* props */
            current: false,
            title: None,
            note: None,
            created_at: None,
            done_at: None,
            chains: None,
            due_at: None,
            defer: None,
            status: None,
            project: None,
            contexts: None,
            tags: None,
            priority: 0,
            time: 0,
            flag: false,
            source_file: None
        }
    }

    fn is_done(&self) -> bool {

        match self.status {
            None => {},
            Some(ref status) => {

                match status {
                    &Status::Done => {
                        return true;
                    },
                    _ => {}
                }
            }
        };

        return false;
    }

    fn has_chain(&self) -> bool {

        if self.chains.is_none() {
            return false;
        }

        match self.chains {
            None => unsafe { debug_unreachable!() },
            Some(ref tree) => {
                return tree.len() > 0;
            }
        }
    }

    fn get_chain(&self) -> NaiveDateTime {

        match self.chains {
            None => unsafe { debug_unreachable!() },
            Some(ref tree) => {
                // see: http://stackoverflow.com/a/33699340/412627
                // let (key, _) = tree.iter().last().unwrap();
                let (key, _) = tree.iter().next_back().unwrap();

                return key.clone();
            }
        }
    }

    fn debug_range_string(&self) -> String {

        if self.task_block_range_start == self.task_block_range_end {
            return format!("on line {}", self.task_block_range_start);
        }

        return format!("between lines {} and {}",
            self.task_block_range_start,
            self.task_block_range_end
        );
    }
}

#[derive(Debug)]
struct GTD {

    /* debug */
    // the line of the last task block line parsed
    previous_task_block_line: u64,

    /* flag/switches */
    hide_flagged: bool,
    show_only_flagged: bool,
    show_done: bool,
    show_incubate: bool,
    show_deferred: bool,
    hide_overdue: bool,
    hide_nonproject_tasks: bool,
    hide_incomplete: bool,
    project_only_filter: Tree,
    project_whitelist: Tree,
    sort_overdue_by_priority: bool,
    filter_by_only_tags: bool,
    filter_by_only_contexts: bool,
    due_within: Duration,
    filter_priority: Option<PriorityFilterTree>,
    hide_tasks_by_default: bool,
    show_overdue: bool,
    show_incomplete: bool,
    show_flagged: bool,
    show_nonproject_tasks: bool,
    show_project_tasks: bool,
    filter_by_include_tags: bool,
    include_tags: HashSet<String>,
    filter_by_include_contexts: bool,
    include_contexts: HashSet<String>,

    /* data */

    current_task: Option<u64>,

    base_root: String,

    // track files opened
    opened_files: HashSet<String>,

    // This tracks the order of the inserted path to file_stats. Used for output.
    file_stats_stack: Vec<String>,
    // path to file -> FileStats
    file_stats: HashMap<String, FileStats>,

    pulse: HashMap<i64, Vec<u64>>,

    only_tags: HashSet<String>,

    only_contexts: HashSet<String>,

    // lookup table for tasks
    tasks: HashMap<u64, Task>,

    // this contains any tasks that are overdue
    // timestamp difference -> task id
    overdue: BTreeMap<i64, Vec<u64>>,

    // this contains any tasks that are either due soon
    // timestamp difference -> task id
    // due_soon: BTreeMap<i64, Vec<i32>>,

    // inbox contain any tasks that do not have a project
    // priority -> vector of task ids ordered by recent appearance
    inbox: BTreeMap<i64, Vec<u64>>,

    // this contains any tasks that are inactive
    // priority -> vector of task ids ordered by recent appearance
    deferred: BTreeMap<i64, Vec<u64>>,

    // this contains any tasks that are compelted
    // priority -> vector of task ids ordered by recent appearance
    done: BTreeMap<i64, Vec<u64>>
}

impl GTD {
    fn new(base_root: String) -> GTD {

        let mut inbox = BTreeMap::new();
        // inbox at priority 0
        inbox.insert(0, Vec::new());
        let inbox = inbox;

        let mut done = BTreeMap::new();
        // done bucket at priority 0
        done.insert(0, Vec::new());
        let done = done;

        let mut deferred = BTreeMap::new();
        // deferred bucket at priority 0
        deferred.insert(0, Vec::new());
        let deferred = deferred;

        GTD {

            /* error output */
            previous_task_block_line: 0,

            /* options */
            hide_flagged: false,
            show_only_flagged: false,
            show_done: false,
            show_incubate: false,
            show_deferred: false,
            hide_overdue: false,
            hide_nonproject_tasks: false,
            hide_incomplete: false,
            project_only_filter: HashMap::new(),
            project_whitelist: HashMap::new(),
            sort_overdue_by_priority: false,
            filter_by_only_tags: false,
            filter_by_only_contexts: false,
            due_within: Duration::seconds(0),
            filter_priority: None,
            hide_tasks_by_default: false,
            show_overdue: false,
            show_incomplete: false,
            show_flagged: false,
            show_nonproject_tasks: false,
            show_project_tasks: false,
            filter_by_include_tags: false,
            include_tags: HashSet::new(),
            filter_by_include_contexts: false,
            include_contexts: HashSet::new(),

            /* data */

            current_task: None,

            base_root: base_root,
            opened_files: HashSet::new(),

            file_stats_stack: Vec::new(),
            file_stats: HashMap::new(),

            pulse: HashMap::new(),

            only_tags: HashSet::new(),
            only_contexts: HashSet::new(),

            tasks: HashMap::new(),
            inbox: inbox,
            done: done,
            deferred: deferred,
            overdue: BTreeMap::new()
        }
    }

    fn add_tag_only_filters(&mut self, tags: Vec<String>) {
        for tag in tags {
            self.only_tags.insert(tag);
        }
    }

    fn add_tag_include_filters(&mut self, tags: Vec<String>) {
        for tag in tags {
            self.include_tags.insert(tag);
        }
    }

    fn have_only_tags(&mut self, tags: &Vec<String>) -> bool {
        for tag in tags {
            if self.only_tags.contains(tag) {
                return true;
            }
        }

        return false;
    }

    fn have_include_tags(&mut self, tags: &Vec<String>) -> bool {
        for tag in tags {
            if self.include_tags.contains(tag) {
                return true;
            }
        }

        return false;
    }

    fn add_context_only_filters(&mut self, contexts: Vec<String>) {
        for context in contexts {
            self.only_contexts.insert(context);
        }
    }

    fn add_context_include_filters(&mut self, contexts: Vec<String>) {
        for context in contexts {
            self.include_contexts.insert(context);
        }
    }

    fn have_only_contexts(&mut self, contexts: &Vec<String>) -> bool {
        for context in contexts {
            if self.only_contexts.contains(context) {
                return true;
            }
        }

        return false;
    }

    fn have_include_contexts(&mut self, contexts: &Vec<String>) -> bool {
        for context in contexts {
            if self.include_contexts.contains(context) {
                return true;
            }
        }

        return false;
    }

    fn add_project_only_filter(&mut self, path: &mut Vec<String>) {
        traverse(path, &mut self.project_only_filter);
    }

    fn add_project_whitelist(&mut self, path: &mut Vec<String>) {
        traverse(path, &mut self.project_whitelist);
    }

    fn has_project_only_filters(&mut self) -> bool {
        self.project_only_filter.len() > 0
    }

    fn has_project_whitelist(&mut self) -> bool {
        self.project_whitelist.len() > 0
    }

    fn should_only_filter_project(&mut self, path: &Vec<String>) -> bool {
        return path_satisfies_tree(&(self.project_only_filter), path);
    }

    fn should_whitelist_project(&mut self, path: &Vec<String>) -> bool {
        return path_satisfies_tree(&(self.project_whitelist), path);
    }

    fn add_task(&mut self, task: Task, directive_switch: &DirectiveSwitches) {

        // TODO: is this the best placement for this?
        let mut task = task;
        task.task_block_range_end = self.previous_task_block_line;
        let task = task;

        // validation

        if !directive_switch.pass_validation(&task, self) {
            // TODO: pass_validation prints errors and therefore produces side-effects; refactor
            return;
        }

        if task.title.is_none() {

            println!("Missing task title (i.e. `task: <title>`) in task block found {}",
                task.debug_range_string()
            );
            println!("Captured:");
            _print_task(self, &task, false);
            process::exit(1);
        }

        let new_id: u64 = self.next_task_id();

        if task.current {

            match self.current_task {
                Some(first_task_id) => {

                    println!("Found at least two current tasks.");
                    println!("Only one task can be marked as current.");
                    println!("");

                    println!("First task found to be current:");
                    let first_task: &Task = self.tasks.get(&first_task_id).unwrap();
                    print_task(self, first_task);

                    println!("");

                    println!("Second task found to be current:");
                    print_task(self, &task);

                    process::exit(1);
                },
                None => {
                    self.current_task = Some(new_id);
                }
            };
        }

        // if let Some(ref title) = task.title {
        //     print_task(self, &task);
        //     println!("------");
        //     // println!("task.title: {}", title);
        // }


        match task.done_at {
            None => {},
            Some(ref done_at) => {

                if !task.is_done() {

                    println!("In file: {}", task.source_file.as_ref().unwrap());
                    println!("Task is incorrectly given a `done` datetime found at {}",
                        task.debug_range_string()
                    );
                    println!("Mayhaps you forgot to add: 'status: done'");
                    process::exit(1);
                } else {
                    self.add_to_pulse(done_at, new_id);
                }

            }
        };

        match task.source_file {
            None => unsafe { debug_unreachable!() },
            Some(ref source_file) => {

                match task.status {
                    None => {

                        if self.is_overdue(&task) {
                            let file_stats = self.file_stats.get_mut(source_file).unwrap();
                            file_stats.add_overdue_task_id(new_id);
                        } else if !self.should_defer(&task) {
                            let file_stats = self.file_stats.get_mut(source_file).unwrap();
                            // add task to inbox
                            file_stats.add_inbox_task_id(new_id);
                        } else {
                            let file_stats = self.file_stats.get_mut(source_file).unwrap();
                            file_stats.add_deferred_task_id(new_id);
                        }

                    },
                    Some(ref status) => {

                        match status {
                            &Status::NotDone => {
                                if self.is_overdue(&task) {
                                    let file_stats = self.file_stats.get_mut(source_file).unwrap();
                                    file_stats.add_overdue_task_id(new_id);
                                } else if !self.should_defer(&task) {
                                    let file_stats = self.file_stats.get_mut(source_file).unwrap();
                                    // add task to inbox
                                    file_stats.add_inbox_task_id(new_id);
                                } else {
                                    let file_stats = self.file_stats.get_mut(source_file).unwrap();
                                    file_stats.add_deferred_task_id(new_id);
                                }
                            },
                            &Status::Incubate => {

                                if self.is_overdue(&task) {
                                    let file_stats = self.file_stats.get_mut(source_file).unwrap();
                                    file_stats.add_overdue_task_id(new_id);
                                } else if !self.should_defer(&task) {
                                    let file_stats = self.file_stats.get_mut(source_file).unwrap();
                                    file_stats.add_incubate_task_id(new_id);
                                } else {
                                    let file_stats = self.file_stats.get_mut(source_file).unwrap();
                                    file_stats.add_deferred_task_id(new_id);
                                }

                            },
                            &Status::Done => {
                                let file_stats = self.file_stats.get_mut(source_file).unwrap();
                                file_stats.add_finished_task_id(new_id);
                            }
                        }
                    }
                };

                let file_stats = self.file_stats.get_mut(source_file).unwrap();

                match task.tags {
                    None => {},
                    Some(ref tags) => {
                        for tag in tags {
                            file_stats.add_tag(tag.clone());
                        }

                    }
                };

                match task.contexts {
                    None => {},
                    Some(ref contexts) => {

                        for context in contexts {
                            file_stats.add_context(context.clone());
                        }

                    }
                };

                match task.project {
                    None => {},
                    Some(ref project_path) => {
                        file_stats.add_project_path(project_path.clone());
                    }
                };

            }
        };

        // sort tasks into various data structures (e.g. overdue, inbox, etc) that shall be displayed
        // to the user

        if self.hide_tasks_by_default {

            // hide task unless it satisfy [whitelist] filters

            self.add_task_default_hidden(&task, new_id);

        } else {

            // default behaviour

            self.add_task_default(&task, new_id);
        }

        // add task to look-up table
        self.tasks.insert(new_id, task);

    }

    fn add_task_default_hidden(&mut self, task: &Task, new_id: u64) {

        if self.should_hide_task(&task) {
            return;
        }

        let mut shall_show: bool =
            self.filter_by_only_tags && task.tags.is_some() ||
            self.filter_by_only_contexts && task.contexts.is_some() ||
            self.has_project_only_filters() && task.project.is_some() ||
            self.show_only_flagged && task.flag ||
            self.show_flagged && task.flag ||
            self.show_nonproject_tasks && task.project.is_none() ||
            self.show_project_tasks && task.project.is_some();


        if self.has_project_whitelist() {
            match task.project {
                Some(ref project_path) => {
                    if self.should_whitelist_project(project_path) {
                        shall_show = true;
                    }
                },
                None => {}
            };
        };

        if self.filter_by_include_tags {
            match task.tags {
                None => {},
                Some(ref tags) => {
                    if self.have_include_tags(tags) {
                        shall_show = true;
                    }
                }
            }
        }

        if self.filter_by_include_contexts {
            match task.contexts {
                None => {},
                Some(ref contexts) => {
                    if self.have_include_contexts(contexts) {
                        shall_show = true;
                    }
                }
            }
        }

        // sort task by status and priority
        match task.status {
            None => {

                if self.hide_incomplete {
                    // hide task
                } else if self.is_overdue(&task) {

                    if self.show_overdue || shall_show {
                        self.add_to_overdue(&task, new_id);
                    }

                } else if !self.should_defer(&task) {

                    if self.show_incomplete || shall_show {
                        // add task to inbox
                        self.add_to_inbox(task.priority, new_id);
                    }

                } else {

                    if self.show_deferred || shall_show {
                        self.add_to_deferred(task.priority, new_id);
                    }

                }

            },
            Some(ref status) => {

                match status {
                    &Status::NotDone => {

                        if self.hide_incomplete {
                            // hide task
                        } else if self.is_overdue(&task) {

                            if self.show_overdue || shall_show {
                                self.add_to_overdue(&task, new_id);
                            }

                        } else if !self.should_defer(&task) {

                            if self.show_incomplete || shall_show {
                                // add task to inbox
                                self.add_to_inbox(task.priority, new_id);
                            }

                        } else {

                            if self.show_deferred || shall_show {
                                self.add_to_deferred(task.priority, new_id);
                            }
                        }
                    },
                    &Status::Incubate => {

                        if self.hide_incomplete {
                            // hide task
                        } else if self.is_overdue(&task) {

                            if self.show_overdue || shall_show {
                                self.add_to_overdue(&task, new_id);
                            }

                        } else if !self.should_defer(&task) {

                            if self.show_incomplete || shall_show {
                                // add task to inbox
                                self.add_to_inbox(task.priority, new_id);
                            }

                        } else {

                            if self.show_deferred || shall_show {
                                self.add_to_deferred(task.priority, new_id);
                            }
                        }
                    },
                    &Status::Done => {

                        if self.show_done || shall_show {
                            self.add_to_done(task.priority, new_id);
                        }

                    }
                }
            }
        }

    }

    fn add_task_default(&mut self, task: &Task, new_id: u64) {

        if self.should_hide_task(&task) {
            return;
        }

        // sort task by status and priority
        match task.status {
            None => {

                if self.hide_incomplete {
                    // hide task
                } else if self.is_overdue(&task) {
                    self.add_to_overdue(&task, new_id);
                } else if !self.should_defer(&task) {
                    // add task to inbox
                    self.add_to_inbox(task.priority, new_id);
                } else {
                    self.add_to_deferred(task.priority, new_id);
                }

            },
            Some(ref status) => {

                match status {
                    &Status::NotDone => {

                        if self.hide_incomplete {
                            // hide task
                        } else if self.is_overdue(&task) {
                            self.add_to_overdue(&task, new_id);
                        } else if !self.should_defer(&task) {
                            // add task to inbox
                            self.add_to_inbox(task.priority, new_id);
                        } else {
                            self.add_to_deferred(task.priority, new_id);
                        }
                    },
                    &Status::Incubate => {

                        if self.hide_incomplete {
                            // hide task
                        } else if self.is_overdue(&task) {
                            self.add_to_overdue(&task, new_id);
                        } else if !self.should_defer(&task) {

                            if self.show_incubate {

                                // add task to inbox
                                self.add_to_inbox(task.priority, new_id);

                            }

                        } else {
                            self.add_to_deferred(task.priority, new_id);
                        }
                    },
                    &Status::Done => {
                        self.add_to_done(task.priority, new_id);
                    }
                }
            }
        }

    }

    fn should_hide_task(&mut self, task: &Task) -> bool {

        if self.hide_nonproject_tasks &&!task.project.is_some() {
            return true;
        }

        if self.filter_by_only_tags {
            match task.tags {
                None => {
                    // TODO: need flag to control this
                    return true;
                },
                Some(ref tags) => {
                    if !self.have_only_tags(tags) {
                        return true;
                    }
                }
            }
        }

        if self.filter_by_only_contexts {
            match task.contexts {
                None => {
                    // TODO: need flag to control this
                    return true;
                },
                Some(ref contexts) => {
                    if !self.have_only_contexts(contexts) {
                        return true;
                    }
                }
            }
        }

        // invariant: task belongs to a project

        // if necessary, apply any project path apply filters

        if self.has_project_only_filters() {

            let should_filter: bool = match task.project {
                Some(ref project_path) => {
                    !self.should_only_filter_project(project_path)
                },
                // TODO: need flag to control this
                None => true
            };

            if should_filter {
                return true;
            }

        }

        if self.show_only_flagged {
            return !task.flag;
        }

        if self.show_flagged && task.flag {
            return false;
        }

        if self.hide_flagged {
            return task.flag;
        }

        if self.filter_priority.is_some() {

            let priority_filter = self.filter_priority.as_ref().unwrap();

            return !priority_satisfy_tree(priority_filter, task.priority);

        }

        // TODO: redundant; remove
        // if self.show_project_tasks && task.project.is_some() {
        //     return false;
        // }

        return false;
    }

    fn should_defer(&self, task: &Task) -> bool {

        // TODO: necessary??
        // if self.show_deferred {
        //     return false;
        // }

        match task.defer {
            None => {
                return false;
            },
            Some(ref defer) => {

                match defer {
                    &Defer::Forever => {
                        return true;
                    },
                    &Defer::Until(defer_till) => {
                        return defer_till.timestamp() > Local::now().naive_local().timestamp();
                    }
                }


            }
        }

        return false;
    }

    fn add_to_pulse(&mut self, done_at: &NaiveDateTime, task_id: u64) {

        let diff = Local::now().naive_local().timestamp() - done_at.timestamp();

        if !(0 <= diff && diff <= chrono::Duration::days(7).num_seconds()) {
            return;
        }

        let diff = diff as f64;

        let sec_per_minute: f64 = 60f64;
        let sec_per_hour: f64 = sec_per_minute * 60f64;
        let sec_per_day: f64 = sec_per_hour * 24f64;

        let days_ago = (diff / sec_per_day).floor() as i64;

        if !self.pulse.contains_key(&days_ago) {
            self.pulse.insert(days_ago, Vec::new());
        }

        match self.pulse.get_mut(&days_ago) {
            None => unsafe { debug_unreachable!("journal.overdue missing expected bucket") },
            Some(bucket) => {
                (*bucket).push(task_id);
            }
        }

    }

    fn is_overdue(&self, task: &Task) -> bool {

        match task.due_at {
            None => {
                return false;
            },
            Some(ref due_at) => {
                return (Local::now().naive_local().timestamp() + self.due_within.num_seconds()) >= due_at.timestamp();
            }
        }

    }

    fn add_to_overdue(&mut self, task: &Task, task_id: u64) {

        match task.due_at {
            None => {
                return;
            },
            Some(ref due_at) => {

                // sort by oldest due to most recently due

                let rel_time = due_at.timestamp() - Local::now().naive_local().timestamp();

                let encoded_key = if self.sort_overdue_by_priority {

                    // override to sort by priority

                    GTD::encode_priority(task.priority) as i64
                } else {
                    // largest negative numbers appear first
                    -rel_time
                };

                if !self.overdue.contains_key(&encoded_key) {
                    self.overdue.insert(encoded_key, Vec::new());
                }

                match self.overdue.get_mut(&encoded_key) {
                    None => unsafe { debug_unreachable!("journal.overdue missing expected bucket") },
                    Some(bucket) => {
                        (*bucket).push(task_id);
                    }
                }

            }
        }

    }

    fn add_to_inbox(&mut self, task_priority: i64, task_id: u64) {

        self.ensure_priority_inbox(task_priority);

        let task_priority: i64 = GTD::encode_priority(task_priority);

        match self.inbox.get_mut(&task_priority) {
            None => unsafe { debug_unreachable!("add_to_inbox: expected priority bucket not found") },
            Some(inbox) => {
                (*inbox).push(task_id);
            }
        }
    }

    fn add_to_deferred(&mut self, task_priority: i64, task_id: u64) {

        self.ensure_priority_deferred(task_priority);

        let task_priority: i64 = GTD::encode_priority(task_priority);

        match self.deferred.get_mut(&task_priority) {
            None => unsafe { debug_unreachable!("add_to_deferred: expected priority bucket not found") },
            Some(deferred) => {
                (*deferred).push(task_id);
            }
        }
    }

    fn add_to_done(&mut self, task_priority: i64, task_id: u64) {

        self.ensure_priority_done(task_priority);

        let task_priority: i64 = GTD::encode_priority(task_priority);

        match self.done.get_mut(&task_priority) {
            None => unsafe { debug_unreachable!("add_to_done: expected priority bucket not found") },
            Some(done) => {
                (*done).push(task_id);
            }
        }
    }


    fn next_task_id(&mut self) -> u64 {
        to_task_id(self.tasks.len() + 1) as u64
    }

    // TODO: refactor

    fn ensure_priority_inbox(&mut self, priority: i64) {

        let priority = GTD::encode_priority(priority);

        if !self.inbox.contains_key(&priority) {
            self.inbox.insert(priority, Vec::new());
        }
    }

    fn ensure_priority_deferred(&mut self, priority: i64) {

        let priority = GTD::encode_priority(priority);

        if !self.deferred.contains_key(&priority) {
            self.deferred.insert(priority, Vec::new());
        }
    }

    fn ensure_priority_done(&mut self, priority: i64) {

        let priority = GTD::encode_priority(priority);

        if !self.done.contains_key(&priority) {
            self.done.insert(priority, Vec::new());
        }
    }

    // TODO: refactor

    fn encode_priority(priority: i64) -> i64 {
        -priority
    }

    // NOTE: unused
    // fn decode_priority(priority: i64) -> i64 {
    //     -priority
    // }
}

/* gtdtxt file parser */

fn parse_file(parent_file: Option<String>, path_to_file_str: String, journal: &mut GTD) {

    let path_to_file: &Path = Path::new(&path_to_file_str);

    if !path_to_file.is_file() {
        // TODO: return Err(...)

        match parent_file {
            None => {},
            Some(parent_file) => {
                println!("In file: {}",
                    parent_file
                );
            }
        };

        println!("Path is not a file: {}",
            path_to_file_str
        );
        process::exit(1);
    }

    // fetch path to file
    let tracked_path = match path_to_file.canonicalize() {
        Ok(resolved) => {
            let resolved: PathBuf = resolved;
            format!("{}", resolved.display())
        },
        Err(e) => {
            panic!("{:?}", e);
        }
    };

    if journal.opened_files.contains(&tracked_path) {
        println!("Cyclic includes detected; file already opened: {}", tracked_path);
        process::exit(1);
    }

    let file: File  = File::open(path_to_file).ok().expect("Failed to open file");

    // track this opened file to ensure we're not opening the same file twice
    journal.opened_files.insert(tracked_path.clone());


    // save current working directory
    let old_working_directory = format!("{}", env::current_dir().unwrap().display());

    // set new current working dir
    let parent_dir: String = {
        let parent_dir = Path::new(&tracked_path).parent().unwrap();
        format!("{}", parent_dir.display())
    };

    if !env::set_current_dir(&parent_dir).is_ok() {
        println!("Unable to change working directory to: {}", parent_dir);
        process::exit(1);
    }

    journal.file_stats.insert(tracked_path.clone(), FileStats::new());
    journal.file_stats_stack.push(tracked_path.clone());

    let mut num_of_lines_parsed = 0;

    // parse gtdtxt file

    let mut input = Source::new(file);

    // directive switches
    let mut directive_switch = DirectiveSwitches::new();

    // initial state
    let mut previous_state: ParseState = ParseState::Start;

    loop {

        let mut n = Numbering::new(LineNumber::new(), line_token_parser);
        // If we could implement FnMut for Numbering then we would be good, but we need to wrap now:
        let m = |i| n.parse(i);

        match input.parse(m) {
            Ok((lines_parsed, line)) => {

                // amend behaviour of newline counting
                let lines_parsed = if lines_parsed == 0 {
                    1
                } else {
                    lines_parsed
                };

                num_of_lines_parsed += lines_parsed;

                match line {

                    LineToken::Task(task_block_line) => {

                        // mark this line as previous task block seen
                        journal.previous_task_block_line = num_of_lines_parsed;

                        let current_task: &mut Task = match previous_state {
                            ParseState::Task(ref mut task) => {
                                task
                            },
                            _ => {
                                let mut new_task: Task = Task::new(num_of_lines_parsed);
                                new_task.source_file = Some(tracked_path.clone());
                                previous_state = ParseState::Task(new_task);

                                // TODO: possible to refactor this in a better way?
                                match previous_state {
                                    ParseState::Task(ref mut task) => {
                                        task
                                    },
                                    _ => unsafe { debug_unreachable!() }
                                }
                            }
                        };

                        match task_block_line {
                            TaskBlock::Current => {
                                current_task.current = true;
                            },
                            TaskBlock::Title(title) => {
                                current_task.title = Some(title);
                            },
                            TaskBlock::Note(note) => {
                                current_task.note = Some(note);
                            },
                            TaskBlock::Project(project) => {

                                if project.len() > 0 {
                                    current_task.project = Some(project);
                                } else {
                                    current_task.project = None;
                                }

                            },
                            TaskBlock::Created(created_at) => {
                                let created_at: NaiveDateTime = created_at;
                                current_task.created_at = Some(created_at);
                            },
                            TaskBlock::Done(done_at) => {
                                let done_at: NaiveDateTime = done_at;
                                current_task.done_at = Some(done_at);
                            },
                            TaskBlock::Chain(chain_at) => {
                                let chain_at: NaiveDateTime = chain_at;
                                match current_task.chains {
                                    None => {

                                        let mut tree = BTreeMap::new();
                                        tree.insert(chain_at, true);

                                        current_task.chains = Some(tree);

                                    },
                                    Some(ref mut tree) => {
                                        tree.insert(chain_at, true);
                                    }
                                };
                            },
                            TaskBlock::Status(status) => {

                                current_task.status = Some(status);
                            },
                            TaskBlock::Due(due_at) => {
                                let due_at: NaiveDateTime = due_at;
                                current_task.due_at = Some(due_at);
                            },
                            TaskBlock::Defer(defer) => {
                                current_task.defer = Some(defer);
                            },
                            TaskBlock::Contexts(contexts) => {

                                if contexts.len() > 0 {
                                    current_task.contexts = Some(contexts);
                                } else {
                                    current_task.contexts = None;
                                }
                            },
                            TaskBlock::Tags(tags) => {

                                if tags.len() > 0 {
                                    current_task.tags = Some(tags);
                                } else {
                                    current_task.tags = None;
                                }
                            },
                            TaskBlock::Time(time) => {
                                current_task.time += time;
                            },
                            TaskBlock::ID(_id) => {
                                // println!("id: '{}'", id);
                                // TODO: complete
                            },
                            TaskBlock::Priority(priority) => {
                                current_task.priority = priority
                            },
                            TaskBlock::Flag(flag) => {
                                current_task.flag = flag;
                            }
                        };

                    },

                    LineToken::Directive(directive_line) => {

                        match previous_state {
                            ParseState::Task(task) => {
                                journal.add_task(task, &directive_switch);
                            },
                            _ => {}
                        };

                        previous_state = ParseState::Directive;

                        match directive_line {
                            Directive::Include(path_to_file) => {
                                parse_file(Some(tracked_path.clone()), path_to_file, journal);
                            },
                            Directive::ShouldNotContainCompletedTasks(result) => {
                                directive_switch.require_no_completed_tasks = Some(result);
                            }
                            Directive::RequiredProjectPrefix(result) => {
                                directive_switch.required_project_prefix = Some(result);
                            }
                        };

                    },

                    LineToken::PreBlock => {

                        // println!("preblock");

                        match previous_state {
                            ParseState::Task(task) => {
                                journal.add_task(task, &directive_switch);
                            },
                            _ => {}
                        };

                        previous_state = ParseState::PreBlock;

                    },

                    LineToken::TaskSeparator => {

                        // println!("TaskSeparator");

                        match previous_state {
                            ParseState::Task(task) => {
                                journal.add_task(task, &directive_switch);
                            },
                            _ => {}
                        };

                        previous_state = ParseState::TaskSeparator;
                    }
                };

            },
            Err(StreamError::Retry) => {
                // Needed to refill buffer when necessary
            },
            Err(StreamError::EndOfInput) => {
                break;
            },
            Err(_err) => {

                // println!("{:?}", e);

                // match e {
                //     StreamError::ParseError(input, _) => {
                //     // ParseError::Error(input, _) => {
                //         println!("StreamError::ParseError {}",  String::from_utf8_lossy(input));
                //     },
                //     _ => {}
                // };

                match previous_state {
                    ParseState::Task(task) => {
                        println!("Error occured when parsing a task.");
                        println!("The following was captured:");
                        print_task(journal, &task);
                    },
                    _ => {}
                };


                println!("Error parsing starting at line {} in file: {}", num_of_lines_parsed + 1, tracked_path);
                process::exit(1);
            }
        }
    };

    match previous_state {
        ParseState::Task(task) => {
            journal.add_task(task, &directive_switch);
        },
        _ => {}
    };

    match journal.file_stats.get(&tracked_path) {
        None => unsafe { debug_unreachable!() },
        Some(file_stats) => {

            match directive_switch.require_no_completed_tasks {
                None => {},
                Some(require_no_completed_tasks) => {

                    let ref tasks = file_stats.completed_tasks;

                    if tasks.len() > 0 && require_no_completed_tasks {
                        println!("Found {} completed tasks that are not supposed to be in file: {}",
                            tasks.len(),
                            tracked_path);

                        let task: &Task = journal.tasks.get(tasks.first().unwrap()).unwrap();

                        println!("Found a completed task at lines: {} to {}",
                            task.task_block_range_start,
                            task.task_block_range_end
                        );
                        println!("");

                        _print_task(journal, task, false);

                        process::exit(1);
                    }
                }
            };

        }
    }

    journal.opened_files.remove(&tracked_path);

    // restore current working dir
    if !env::set_current_dir(&old_working_directory).is_ok() {
        println!("Unable to change working directory to: {}", old_working_directory);
        process::exit(1);
    }

}

/* parsers */

// state machine:
// Start = PreBlock | Task | Directive | TaskSeparator
// PreBlock = PreBlock | Task | Directive | TaskSeparator
// TaskSeparator = PreBlock | Task | Directive | TaskSeparator
// Task = Task | PreBlock | TaskSeparator
// Directive = Directive | PreBlock | TaskSeparator
#[derive(Debug)]
enum ParseState {
    Start,
    PreBlock,
    Task(Task),
    Directive,
    TaskSeparator
}

#[derive(Debug)]
enum LineToken {
    Task(TaskBlock),
    Directive(Directive),
    PreBlock,
    TaskSeparator
}


fn line_token_parser(input: Input<u8>) -> U8Result<LineToken> {

    or(input,
        |i| parse!{i;

            // this line shall not begin with any whitespace
            look_ahead(|i| satisfy(i, |c| !is_whitespace(c)));

            let line: LineToken = task_seperators() <|>
            task_block() <|>
            directives();

            ret line
        },
        |i| pre_block(i)
    )

}

/* preblock */

fn pre_block(i: Input<u8>) -> U8Result<LineToken> {
    parse!{i;

        /*
        consume comment blocks or whitespace till
        one line comments or terminating
         */
        let _line: Vec<()> = many_till(
            |i| or(i,
                |i| whitespace(i),
                |i| comments_block(i)
            ),
            |i| or(i,
                |i| comments_one_line(i),
                |i| terminating(i)
            )
        );

        ret LineToken::PreBlock;
    }
}


/* task block */

#[derive(Debug)]
enum Defer {
    Forever,
    Until(NaiveDateTime)
}

// tokens from parser
#[derive(Debug)]
enum TaskBlock {
    Current,
    Title(String),
    Created(NaiveDateTime),
    Done(NaiveDateTime),
    Chain(NaiveDateTime),
    Due(NaiveDateTime),
    Defer(Defer),
    Priority(i64),
    Time(u64),
    Project(Vec<String>),
    Status(Status),
    Contexts(Vec<String>),
    Tags(Vec<String>),
    Flag(bool),
    Note(String),

    // TODO: complete
    ID(String)
}

fn task_block(i: Input<u8>) -> U8Result<LineToken> {

    parse!{i;

        let line: TaskBlock =
            task_current() <|>
            task_title() <|>
            task_priority() <|>
            task_project() <|>
            task_flag() <|>
            task_created() <|>
            task_done() <|>
            task_chain() <|>
            task_status() <|>
            task_due() <|>
            task_defer() <|>
            task_tags() <|>
            task_contexts() <|>
            task_time() <|>
            // TODO: complete
            // task_id()
            task_note();

        ret LineToken::Task(line)
    }
}

fn task_current(input: Input<u8>) -> U8Result<TaskBlock> {

    parse!{input;

        string_ignore_case("current".as_bytes());

        let _line: Vec<()> = many_till(|i| space_or_tab(i), |i| terminating(i));

        ret TaskBlock::Current
    }

}

fn task_title(input: Input<u8>) -> U8Result<TaskBlock> {

    parse!{input;

        // aliases
        string_ignore_case("task".as_bytes()) <|>
        string_ignore_case("todo".as_bytes()) <|>
        string_ignore_case("action".as_bytes()) <|>
        string_ignore_case("item".as_bytes());

        token(b':');

        let line = non_empty_line();

        ret {
            let title: String = format!("{}", String::from_utf8_lossy(line.as_slice()).trim());
            TaskBlock::Title(title)
        }
    }
}

fn task_note(input: Input<u8>) -> U8Result<TaskBlock> {

    parse!{input;

        // aliases
        string_ignore_case("notes".as_bytes()) <|>
        string_ignore_case("note".as_bytes()) <|>
        string_ignore_case("description".as_bytes()) <|>
        string_ignore_case("desc".as_bytes());

        token(b':');

        skip_many(|i| space_or_tab(i));

        let line = or(
            |i| parse!{i;
                terminating();

                ret {
                    let line: Vec<u8> = vec![];
                    line
                }
            },
            // NOTE: this must be parsed last
            |i| non_empty_line(i)
        );

        let other_lines: Vec<String> = many(
            |i| or(i,
                |i| parse!{i;

                    space_or_tab();

                    let line = non_empty_line();

                    ret {
                        let line: String = format!("{:>11} {}",
                            "",
                            String::from_utf8_lossy(line.as_slice()).trim()
                        );
                        line
                    }
                },
                |i| parse!{i;

                    // allow empty lines in note

                    let nothing: Vec<()> = many(|i| parse!{i;
                        let _nothing: Vec<()> = many_till(|i| space_or_tab(i), |i| end_of_line(i));
                        ret ()
                    });

                    space_or_tab();

                    let line = non_empty_line();

                    ret {

                        let filler = String::from_utf8(vec![b'\n'; nothing.len()]).ok().unwrap();

                        let line: String = format!("{}{:>11} {}",
                            filler,
                            "",
                            String::from_utf8_lossy(line.as_slice()).trim()
                        );
                        line
                    }
                }
            )

        );

        ret {
            let line: String = format!("{}", String::from_utf8_lossy(line.as_slice()).trim());
            let other_lines = other_lines.join("\n");

            let note = if other_lines.len() > 0 {
                if line.len() > 0 {
                    format!("{}\n{}", line, other_lines)
                } else {
                    format!("{}", other_lines.trim())
                }

            } else {
                format!("{}", line)
            };

            TaskBlock::Note(note)
        }
    }

}

fn task_time(input: Input<u8>) -> U8Result<TaskBlock> {

    parse!{input;

        string_ignore_case("time".as_bytes());
        token(b':');

        look_ahead(|i| non_empty_line(i));

        skip_many(|i| space_or_tab(i));

        let time: u64 = multiple_time_range();


        let _nothing: Vec<()> = many_till(|i| space_or_tab(i), |i| terminating(i));

        ret TaskBlock::Time(time)
    }
}

fn parse_priority_number(input: Input<u8>) -> U8Result<i64> {
    parse!{input;
        let priority: i64 = signed_decimal() <|> decimal();
        ret priority
    }
}

fn task_priority(input: Input<u8>) -> U8Result<TaskBlock> {

    parse!{input;

        string_ignore_case("priority".as_bytes());
        token(b':');

        look_ahead(|i| non_empty_line(i));

        skip_many(|i| space_or_tab(i));

        let priority: i64 = parse_priority_number();

        let _nothing: Vec<()> = many_till(|i| space_or_tab(i), |i| terminating(i));

        ret TaskBlock::Priority(priority)
    }
}

fn task_project(input: Input<u8>) -> U8Result<TaskBlock> {

    parse!{input;

        string_ignore_case("project".as_bytes());
        token(b':');

        look_ahead(|i| non_empty_line(i));

        let list = string_list(b'/');

        ret TaskBlock::Project(list)
    }
}

fn task_flag(input: Input<u8>) -> U8Result<TaskBlock> {

    parse!{input;

        string_ignore_case("flag".as_bytes());
        token(b':');

        look_ahead(|i| non_empty_line(i));

        skip_many(|i| space_or_tab(i));

        let input = bool_option_parser();

        let _line: Vec<()> = many_till(|i| space_or_tab(i), |i| terminating(i));

        ret TaskBlock::Flag(input)
    }
}

fn task_created(input: Input<u8>) -> U8Result<TaskBlock> {

    parse!{input;

        string_ignore_case("created at".as_bytes()) <|>
        string_ignore_case("created".as_bytes()) <|>
        string_ignore_case("date".as_bytes()) <|>
        string_ignore_case("added at".as_bytes()) <|>
        string_ignore_case("added".as_bytes());

        token(b':');

        look_ahead(|i| non_empty_line(i));

        skip_many(|i| space_or_tab(i));

        let created_at = parse_datetime(false);

        let _line: Vec<()> = many_till(|i| space_or_tab(i), |i| terminating(i));

        ret TaskBlock::Created(created_at)
    }
}

fn task_done(input: Input<u8>) -> U8Result<TaskBlock> {

    parse!{input;

        string_ignore_case("done at".as_bytes()) <|>
        string_ignore_case("done".as_bytes()) <|>
        string_ignore_case("completed".as_bytes()) <|>
        string_ignore_case("complete".as_bytes());

        token(b':');

        look_ahead(|i| non_empty_line(i));

        skip_many(|i| space_or_tab(i));

        let done_at = parse_datetime(false);

        let _line: Vec<()> = many_till(|i| space_or_tab(i), |i| terminating(i));

        ret TaskBlock::Done(done_at)
    }
}

fn task_chain(input: Input<u8>) -> U8Result<TaskBlock> {

    parse!{input;

        string_ignore_case("chain".as_bytes());

        token(b':');

        look_ahead(|i| non_empty_line(i));

        skip_many(|i| space_or_tab(i));

        let chain_at = parse_datetime(false);

        let _line: Vec<()> = many_till(|i| space_or_tab(i), |i| terminating(i));

        ret TaskBlock::Chain(chain_at)
    }
}

fn parse_status(input: Input<u8>) -> U8Result<Status> {

    or(input,
        |i| parse!{i;

            string_ignore_case("done".as_bytes()) <|>
            string_ignore_case("complete".as_bytes()) <|>
            string_ignore_case("finished".as_bytes()) <|>
            string_ignore_case("finish".as_bytes()) <|>
            string_ignore_case("fin".as_bytes());

            ret Status::Done
        },
        |i| or(i,
            |i| parse!{i;

                string_ignore_case("hide".as_bytes()) <|>
                string_ignore_case("hidden".as_bytes()) <|>
                string_ignore_case("incubate".as_bytes()) <|>
                string_ignore_case("later".as_bytes()) <|>
                string_ignore_case("someday".as_bytes()) <|>
                string_ignore_case("inactive".as_bytes()) <|>
                string_ignore_case("not active".as_bytes());

                ret Status::Incubate
            },
            |i| parse!{i;

                string_ignore_case("active".as_bytes()) <|>
                string_ignore_case("not done".as_bytes()) <|>
                string_ignore_case("progress".as_bytes()) <|>
                string_ignore_case("in progress".as_bytes()) <|>
                string_ignore_case("in-progress".as_bytes()) <|>
                string_ignore_case("pending".as_bytes()) <|>
                string_ignore_case("is active".as_bytes());

                ret Status::NotDone
            }
        )
    )
}

fn task_status(input: Input<u8>) -> U8Result<TaskBlock> {

    parse!{input;

        string_ignore_case("status".as_bytes());
        token(b':');

        look_ahead(|i| non_empty_line(i));

        skip_many(|i| space_or_tab(i));

        let status = parse_status();

        let _line: Vec<()> = many_till(|i| space_or_tab(i), |i| terminating(i));

        ret TaskBlock::Status(status)
    }
}

fn task_due(input: Input<u8>) -> U8Result<TaskBlock> {

    parse!{input;

        string_ignore_case("due".as_bytes());
        token(b':');

        look_ahead(|i| non_empty_line(i));

        skip_many(|i| space_or_tab(i));

        let due_at = parse_datetime(true);

        let _line: Vec<()> = many_till(|i| space_or_tab(i), |i| terminating(i));

        ret TaskBlock::Due(due_at)
    }
}

fn task_defer(input: Input<u8>) -> U8Result<TaskBlock> {

    parse!{input;

        string_ignore_case("defer till".as_bytes()) <|>
        string_ignore_case("defer until".as_bytes()) <|>
        string_ignore_case("defer".as_bytes()) <|>
        string_ignore_case("hide until".as_bytes()) <|>
        string_ignore_case("hidden".as_bytes()) <|>
        string_ignore_case("hide till".as_bytes()) <|>
        string_ignore_case("hide".as_bytes());

        token(b':');

        look_ahead(|i| non_empty_line(i));

        skip_many(|i| space_or_tab(i));

        let defer = or(
            |i| parse!{i;
                string_ignore_case("forever".as_bytes());
                ret Defer::Forever
            },
            |i| parse!{i;
                let defer_till = parse_datetime(false);
                ret Defer::Until(defer_till)
            }
        );

        let _line: Vec<()> = many_till(|i| space_or_tab(i), |i| terminating(i));

        ret TaskBlock::Defer(defer)
    }
}

fn task_contexts(input: Input<u8>) -> U8Result<TaskBlock> {

    parse!{input;

        string_ignore_case("contexts".as_bytes()) <|>
        string_ignore_case("context".as_bytes());

        token(b':');

        look_ahead(|i| non_empty_line(i));

        let list = string_list(b',');

        ret TaskBlock::Contexts(list)
    }
}

fn task_tags(input: Input<u8>) -> U8Result<TaskBlock> {

    parse!{input;

        string_ignore_case("tags".as_bytes()) <|>
        string_ignore_case("tag".as_bytes());

        token(b':');

        look_ahead(|i| non_empty_line(i));

        let list = string_list(b',');

        ret TaskBlock::Tags(list)
    }
}

fn task_id(input: Input<u8>) -> U8Result<TaskBlock> {

    parse!{input;

        string_ignore_case("id".as_bytes());
        token(b':');

        let line = non_empty_line();

        ret {
            let id: String = format!("{}", String::from_utf8_lossy(line.as_slice()).trim());
            TaskBlock::ID(id)
        }
    }
}

/* directives */

struct DirectiveSwitches {
    require_no_completed_tasks: Option<bool>,
    required_project_prefix: Option<Vec<String>>
}

impl DirectiveSwitches {
    fn new() -> DirectiveSwitches {
        DirectiveSwitches {
            require_no_completed_tasks: None,
            required_project_prefix: None
        }
    }

    // TODO: this function produces side-effects; refactor
    fn pass_validation(&self, task: &Task, journal: &GTD) -> bool {

        match self.required_project_prefix {
            None => {},
            Some(ref required_project_prefix) => {

                let has_required_project_prefix: bool = match task.project {
                    None => false,
                    Some(ref project_path) => {

                        // ensure task.project has project_path as prefix

                        if project_path.len() < required_project_prefix.len() {
                            false
                        } else {

                            let mut matches = true;
                            let mut idx = required_project_prefix.len();

                            while idx > 0 {
                                idx = idx - 1;

                                if required_project_prefix[idx] != project_path[idx] {
                                    matches = false;
                                    break;
                                }
                            }

                            matches
                        }
                    }
                };

                if !has_required_project_prefix {

                    println!("The following task's project path does not begin with required project path prefix: {}",
                        required_project_prefix.join(" / "));

                    _print_task(journal, &task, false);
                    process::exit(1);
                }

            }
        };

        return true;
    }
}

#[derive(Debug)]
enum Directive {
    Include(String),
    ShouldNotContainCompletedTasks(bool),
    RequiredProjectPrefix(Vec<String>)
}

fn directives(input: Input<u8>) -> U8Result<LineToken> {

    parse!{input;

        let line: Directive = directive_include() <|>
            directive_not_contain_done_tasks() <|>
            directive_required_project_prefix();

        ret {
            LineToken::Directive(line)
        }
    }
}

fn directive_include(input: Input<u8>) -> U8Result<Directive> {

    parse!{input;

        string_ignore_case("include".as_bytes());
        token(b':');

        skip_many(|i| space_or_tab(i));

        let line = non_empty_line();

        ret {
            let path_to_file: String = format!("{}", String::from_utf8_lossy(line.as_slice()).trim());
            Directive::Include(path_to_file)
        }
    }
}

fn directive_not_contain_done_tasks(input: Input<u8>) -> U8Result<Directive> {

    parse!{input;

        string_ignore_case("file_no_done_tasks".as_bytes()) <|>
        string_ignore_case("require no done tasks".as_bytes()) <|>
        string_ignore_case("require no completed tasks".as_bytes()) <|>
        string_ignore_case("require no finished tasks".as_bytes());
        token(b':');

        skip_many(|i| space_or_tab(i));

        let input = bool_option_parser();

        let _nothing: Vec<()> = many_till(|i| space_or_tab(i), |i| terminating(i));

        ret Directive::ShouldNotContainCompletedTasks(input)
    }
}

fn directive_required_project_prefix(input: Input<u8>) -> U8Result<Directive> {

    parse!{input;

        string_ignore_case("required project prefix".as_bytes()) <|>
        string_ignore_case("require project prefix".as_bytes()) <|>
        string_ignore_case("require_project_prefix".as_bytes()) <|>
        string_ignore_case("required_project_prefix".as_bytes());
        token(b':');

        look_ahead(|i| non_empty_line(i));

        let list = string_list(b'/');


        ret Directive::RequiredProjectPrefix(list)
    }
}

/* line parsers */

enum Line {
    Empty,
    NonEmpty(Vec<u8>)
}

fn non_empty_line(i: Input<u8>) -> U8Result<Vec<u8>> {
    parse_line(i)
        .bind(parse_non_empty_line)
}

// TODO: bother moving as closure?
fn parse_non_empty_line(i: Input<u8>, above: Line) -> U8Result<Vec<u8>> {
    match above {
        Line::Empty => {
            // need at least one u8 token
            i.incomplete(1)
        },
        Line::NonEmpty(line) => {

            if line.len() <= 0 {
                return i.incomplete(1);
            }

            i.ret(line)
        }
    }
}

fn parse_line(i: Input<u8>) -> U8Result<Line> {

    // many_till(i, any, |i| terminating(i))
    or(i,
        |i| parse!{i;
            terminating();
            ret Line::Empty
        },
        |i| parse!{i;

            // lines with just whitespace are probably not interesting
            // TODO: consider space_or_tab?
            skip_many(|i| whitespace(i));

            let line: Vec<u8> = many_till(any, |i| terminating(i));
            ret Line::NonEmpty(line)
        }
    )

}

/* task separator */

fn task_seperators(input: Input<u8>) -> U8Result<LineToken> {
    parse!{input;

        parse_task_separator("-".as_bytes()) <|>
        parse_task_separator("=".as_bytes()) <|>
        parse_task_separator("_".as_bytes()) <|>
        // TODO: necessary?
        parse_task_separator("#".as_bytes()) <|>
        parse_task_separator("/".as_bytes()) <|>
        parse_task_separator(":".as_bytes()) <|>
        parse_task_separator("~".as_bytes()) <|>
        parse_task_separator("*".as_bytes());

        ret {
            LineToken::TaskSeparator
        }
    }
}

fn parse_task_separator<'a>(input: Input<'a, u8>, token: &[u8])
-> SimpleResult<'a, u8, ()> {

    parse!{input;

        match_four_tokens(token);
        skip_many(|i| string(i, token));
        let _line: Vec<()> = many_till(|i| space_or_tab(i), |i| terminating(i));

        ret ()
    }
}

/* comments parsers */

fn comments_one_line(i: Input<u8>) -> U8Result<()> {
    parse!{i;
        or(
            |i| string(i, "//".as_bytes()),
            |i| or(i,
                |i| string(i, "#".as_bytes()),
                |i| string(i, ";".as_bytes())
            )
        );

        let _line: Vec<u8> = many_till(|i| any(i), |i| terminating(i));
        ret ()
    }
}

fn comments_block(i: Input<u8>) -> U8Result<()> {
    parse!{i;
        string("/*".as_bytes());

        let _line: Vec<u8> = many_till(|i| any(i), |i| string(i, "*/".as_bytes()));
        ret ()
    }
}

/* delimited list parser */

fn parse_string_lists(input: Input<u8>, delim: u8) -> U8Result<Vec<String>> {
    parse!{input;
        skip_many(|i| space_or_tab(i));
        let result = string_list(b'/');
        skip_many(|i| space_or_tab(i));
        eof();
        ret result
    }
}

fn string_list(input: Input<u8>, delim: u8) -> U8Result<Vec<String>> {
    parse!{input;

        let line = non_empty_line();

        ret {
            let line: Vec<u8> = line;

            let list: Vec<String> = String::from_utf8_lossy(line.as_slice())
                .trim()
                .split(delim as char).map(|c| c.trim().to_string())
                .filter(|x| x.len() > 0)
                .collect();


            list
        }
    }
}

/* misc parsers */

fn bool_option_parser(i: Input<u8>) -> U8Result<bool> {
    or(i,
        |i| parse!{i;

            string_ignore_case("yes".as_bytes()) <|>
            string_ignore_case("true".as_bytes());

            ret true
        },
        |i| parse!{i;

            string_ignore_case("no".as_bytes()) <|>
            string_ignore_case("false".as_bytes());

            ret false
        }
    )
}

fn match_four_tokens<'a>(input: Input<'a, u8>, token: &[u8])
-> SimpleResult<'a, u8, ()> {

    parse!{input;
        string(token);
        string(token);
        string(token);
        string(token);

        ret ()
    }
}

fn whitespace(i: Input<u8>) -> U8Result<()> {
    parse!{i;
        satisfy(|c| is_whitespace(c));
        ret ()
    }
}

fn space_or_tab(input: Input<u8>) -> U8Result<()> {
    parse!{input;
        or(
            |i| token(i, b' '),
            |i| token(i, b'\t')
        );
        ret ()
    }
}

fn non_terminating(i: Input<u8>) -> U8Result<u8> {

    or(i,
        |i| parse!{i;
            terminating();
            ret None
        },
        |i| parse!{i;

            let something = any();

            ret Some(something)
        }
    )
    .bind(|i, above: Option<u8>| {

        match above {
            None => {
                return i.incomplete(1);
            },
            Some(c) => {
                return i.ret(c);
            }
        }

    })
}

// match eof or various eol
fn terminating(i: Input<u8>) -> U8Result<()> {
    or(i,
        |i| parse!{i;
            end_of_line();
            ret ()
        },
        // NOTE: eof should be matched last
        |i| eof(i)
    )
}

// Source: https://en.wikipedia.org/wiki/Newline#Unicode
fn end_of_line(i: Input<u8>) -> U8Result<&[u8]> {
    // TODO: bother to refactor using parse! macro with <|> operator?
    or(i,
        |i| parse!{i;
            token(b'\r');
            token(b'\n');
            ret "\r\n".as_bytes()
        },
        |i| or(i,
            |i| parse!{i;
                token(b'\n');
                ret "\n".as_bytes()
            },
            |i| or(i,
                |i| parse!{i;
                    token(b'\r');
                    ret "\r".as_bytes()
                },
                |i| or(i,
                    |i| parse!{i;
                        string("\u{2028}".as_bytes());
                        ret "\u{2028}".as_bytes()
                    },
                    |i| or(i,
                        |i| parse!{i;
                            string("\u{2029}".as_bytes());
                            ret "\u{2029}".as_bytes()
                        },
                        |i| or(i,
                            |i| parse!{i;
                                string("\u{000B}".as_bytes());
                                ret "\u{000B}".as_bytes()
                            },
                            |i| or(i,
                                |i| parse!{i;
                                    string("\u{000C}".as_bytes());
                                    ret "\u{000C}".as_bytes()
                                },
                                |i| parse!{i;
                                    string("\u{0085}".as_bytes());
                                    ret "\u{0085}".as_bytes()
                                }
                            )
                        )
                    )
                )
            )
        )
    )
}

fn string_ignore_case<'a>(i: Input<'a, u8>, s: &[u8])
    -> SimpleResult<'a, u8, &'a [u8]> {
    let b = i.buffer();

    if s.len() > b.len() {
        return i.incomplete(s.len() - b.len());
    }

    let d = &b[..s.len()];

    for j in 0..s.len() {

        if !(s[j]).eq_ignore_ascii_case(&(d[j])) {
            return i.replace(&b[j..]).err(Error::expected(d[j]))
        }
    }

    i.replace(&b[s.len()..]).ret(d)
}

fn signed_decimal(input: Input<u8>) -> U8Result<i64> {

    parse!{input;
        let sign: i64 = or(
            |i| parse!{i;
                token(b'-');
                ret -1
            },
            |i| parse!{i;
                token(b'+');
                ret 1
            }
        );

        let num: i64 = decimal();

        ret {
            sign * num
        }
    }
}

/* inequality parsers */

// TODO: move this somewhere else
type Priority = i64;

#[derive(Debug, Clone)]
enum Inequality {
    GreaterThan,
    GreaterThanOrEqual,
    LessThan,
    LessThanOrEqual,
    Equal
}

#[derive(Debug, Clone)]
struct PriorityFilter(Inequality, Priority);

#[derive(Debug, Clone)]
enum PriorityFilterTree {
    Union(Box<PriorityFilterTree>, Box<PriorityFilterTree>),
    Intersection(Box<PriorityFilterTree>, Box<PriorityFilterTree>),
    Leaf(PriorityFilter)
}

impl PriorityFilterTree {
    fn is_leaf(&self) -> bool {
        match *self {
            PriorityFilterTree::Leaf(_) => true,
            _ => false,
        }
    }
}

fn priority_pretty_tree_art(filter_tree: &PriorityFilterTree) -> String {

    match filter_tree {
        &PriorityFilterTree::Leaf(ref filter) => {

            let &PriorityFilter(ref inequality, priority) = filter;

            let placeholder = match inequality {
                &Inequality::GreaterThan => format!("> {}", priority),
                &Inequality::GreaterThanOrEqual => format!(">= {}", priority),
                &Inequality::LessThan => format!("< {}", priority),
                &Inequality::LessThanOrEqual => format!("<= {}", priority),
                &Inequality::Equal => format!("== {}", priority)
            };

            return placeholder;

        },
        &PriorityFilterTree::Union(ref left_tree, ref right_tree) => {

            let left_tree = if left_tree.is_leaf() {
                priority_pretty_tree_art(left_tree)
            } else {
                format!("({})", priority_pretty_tree_art(left_tree))
            };

            let right_tree = if right_tree.is_leaf() {
                priority_pretty_tree_art(right_tree)
            } else {
                format!("({})", priority_pretty_tree_art(right_tree))
            };

            return format!("{} or {}", left_tree, right_tree);
        },
        &PriorityFilterTree::Intersection(ref left_tree, ref right_tree) => {

            let left_tree = if left_tree.is_leaf() {
                priority_pretty_tree_art(left_tree)
            } else {
                format!("({})", priority_pretty_tree_art(left_tree))
            };

            let right_tree = if right_tree.is_leaf() {
                priority_pretty_tree_art(right_tree)
            } else {
                format!("({})", priority_pretty_tree_art(right_tree))
            };

            return format!("{} and {}", left_tree, right_tree);
        }
    }

}

fn priority_satisfy_tree(filter_tree: &PriorityFilterTree, task_priority: Priority) -> bool {

    match filter_tree {
        &PriorityFilterTree::Leaf(ref filter) => {

            let &PriorityFilter(ref inequality, priority) = filter;

            let result = match inequality {
                &Inequality::GreaterThan => task_priority > priority,
                &Inequality::GreaterThanOrEqual => task_priority >= priority,
                &Inequality::LessThan => task_priority < priority,
                &Inequality::LessThanOrEqual => task_priority <= priority,
                &Inequality::Equal => task_priority == priority
            };

            return result;

        },
        &PriorityFilterTree::Union(ref left_tree, ref right_tree) => {

            if priority_satisfy_tree(left_tree, task_priority) {
                return true;
            }

            if priority_satisfy_tree(right_tree, task_priority) {
                return true;
            }

            return false;
        },
        &PriorityFilterTree::Intersection(ref left_tree, ref right_tree) => {

            if !priority_satisfy_tree(left_tree, task_priority) {
                return false;
            }

            if !priority_satisfy_tree(right_tree, task_priority) {
                return false;
            }

            return true;
        }
    }

}

fn parse_show_priority(input: Input<u8>) -> U8Result<PriorityFilterTree> {

    parse!{input;

        skip_many(|i| space_or_tab(i));

        let result = parse_priority_filter_tree();

        skip_many(|i| space_or_tab(i));
        eof();

        ret result
    }

}

/*
Source: http://www.engr.mun.ca/~theo/Misc/exp_parsing.htm#classic

Original:
E --> T {or T}
T --> P {and P}
P --> "(" E ")" | leaf

Expanded:
E = T K | T
K = or T K | or T
T = P L | P
L = and P L | and P
P = "(" E ")" | leaf

Renamed variables:
tree = maybe_predicate_intersect union | maybe_predicate_intersect
union = or maybe_predicate_intersect union | or maybe_predicate_intersect
maybe_predicate_intersect = predicate intersect | predicate
intersect = and predicate intersect | and predicate
predicate = "(" tree ")" | leaf
 */

fn parse_priority_filter_tree(input: Input<u8>) -> U8Result<PriorityFilterTree> {
    or(input,
        |input| parse!{input;

            let left_node = parse_priority_filter_tree_maybe_predicate_intersect();

            skip_many(|i| space_or_tab(i));

            let right_node = parse_priority_filter_tree_union(left_node);
            ret right_node
        },
        |input| parse!{input;

            let right_node = parse_priority_filter_tree_maybe_predicate_intersect();
            ret right_node
        }
    )
}

fn parse_priority_filter_tree_union(input: Input<u8>, left_node: PriorityFilterTree)
-> U8Result<PriorityFilterTree> {
    or(input,
        |input| parse!{input;

            string("||".as_bytes()) <|>
            string("|".as_bytes()) <|>
            string("or".as_bytes());

            skip_many(|i| space_or_tab(i));

            let right_node = parse_priority_filter_tree_maybe_predicate_intersect();

            skip_many(|i| space_or_tab(i));

            let tree = parse_priority_filter_tree_union({
                let left_node = Box::new(left_node.clone());
                let right_node = Box::new(right_node);

                PriorityFilterTree::Union(left_node, right_node)
            });

            ret tree
        },
        |input| parse!{input;

            string("||".as_bytes()) <|>
            string("|".as_bytes()) <|>
            string("or".as_bytes());

            skip_many(|i| space_or_tab(i));

            let right_node = parse_priority_filter_tree_maybe_predicate_intersect();

            ret {
                let left_node = Box::new(left_node.clone());
                let right_node = Box::new(right_node);

                PriorityFilterTree::Union(left_node, right_node)
            }
        },
    )
}

fn parse_priority_filter_tree_maybe_predicate_intersect(input: Input<u8>)
-> U8Result<PriorityFilterTree> {

    or(input,
        |input| parse!{input;

            let left_node = parse_priority_filter_tree_predicate();

            skip_many(|i| space_or_tab(i));

            let right_node = parse_priority_filter_tree_intersect(left_node);
            ret right_node;

        },
        |input| parse!{input;

            let right_node = parse_priority_filter_tree_predicate();
            ret right_node;
        }
    )

}

fn parse_priority_filter_tree_intersect(input: Input<u8>, left_node: PriorityFilterTree)
-> U8Result<PriorityFilterTree> {
    or(input,
        |input| parse!{input;

            string("&&".as_bytes()) <|>
            string("&".as_bytes()) <|>
            string("and".as_bytes());

            skip_many(|i| space_or_tab(i));

            let right_node = parse_priority_filter_tree_predicate();

            skip_many(|i| space_or_tab(i));

            let tree = parse_priority_filter_tree_intersect({
                let left_node = Box::new(left_node.clone());
                let right_node = Box::new(right_node);

                PriorityFilterTree::Intersection(left_node, right_node)
            });

            ret tree
        },
        |input| parse!{input;

            string("&&".as_bytes()) <|>
            string("&".as_bytes()) <|>
            string("and".as_bytes());

            skip_many(|i| space_or_tab(i));

            let right_node = parse_priority_filter_tree_predicate();

            ret {
                let left_node = Box::new(left_node.clone());
                let right_node = Box::new(right_node);

                PriorityFilterTree::Intersection(left_node, right_node)
            }
        },
    )
}

fn parse_priority_filter_tree_predicate(input: Input<u8>) -> U8Result<PriorityFilterTree> {
    or(input,
        |input| parse!{input;

            token(b'(');
            skip_many(|i| space_or_tab(i));

            let tree = parse_priority_filter_tree();

            skip_many(|i| space_or_tab(i));
            token(b')');

            ret tree

        },
        |input| parse!{input;

            let filter = parse_priority_filter();

            ret PriorityFilterTree::Leaf(filter)

        },
    )
}

fn parse_priority_filter(input: Input<u8>) -> U8Result<PriorityFilter> {

    or(input,
        |input| parse!{input;

            let operator = parse_inequality();

            skip_many(|i| space_or_tab(i));

            let priority = parse_priority_number();

            ret PriorityFilter(operator, priority)
        },
        |input| parse!{input;

            let priority = parse_priority_number();

            ret PriorityFilter(Inequality::Equal, priority)
        }
    )
}

fn parse_inequality(input: Input<u8>) -> U8Result<Inequality> {
    parse!{input;

        let result = __parse_inequality(">=", Inequality::GreaterThanOrEqual) <|>
            __parse_inequality(">", Inequality::GreaterThan) <|>
            __parse_inequality("<=", Inequality::LessThanOrEqual) <|>
            __parse_inequality("<", Inequality::LessThan) <|>
            __parse_inequality("==", Inequality::Equal) <|>
            __parse_inequality("=", Inequality::Equal);

        ret result
    }
}

fn __parse_inequality<'a>(input: Input<'a, u8>, needle: &str, output: Inequality) -> SimpleResult<'a, u8, Inequality> {
    parse!{input;
        string_ignore_case(needle.as_bytes());
        ret output
    }
}

/* time range parsers */

fn parse_times_ranges(i: Input<u8>) -> U8Result<u64> {
    parse!{i;
        skip_many(|i| space_or_tab(i));
        let result: u64 = multiple_time_range();
        skip_many(|i| space_or_tab(i));
        eof();
        ret result
    }
}

fn multiple_time_range(i: Input<u8>) -> U8Result<u64> {

    parse!{i;

        let time: Vec<u64> = many1(
            |i| or(i,
                |i| parse!{i;
                    skip_many(|i| space_or_tab(i));
                    let range1: u64 = time_range();

                    space_or_tab();
                    skip_many(|i| space_or_tab(i));
                    string_ignore_case("and".as_bytes());
                    space_or_tab();
                    skip_many(|i| space_or_tab(i));

                    let range2: u64 = time_range();

                    ret {
                        range1 + range2
                    }
                },
                |i| parse!{i;
                    skip_many(|i| space_or_tab(i));
                    let range = time_range();
                    ret range
                }
            )
        );

        ret {
            let time = time.iter().fold(0, |mut sum, &val| {sum += val; sum});
            time
        }
    }
}

fn time_range(i: Input<u8>) -> U8Result<u64> {
    parse!{i;

        let range: u64 = decimal();

        skip_many(|i| space_or_tab(i));

        let multiplier = time_range_unit_minutes() <|>
            time_range_unit_hours() <|>
            time_range_unit_days() <|>
            time_range_unit_seconds();

        ret {
            range * multiplier
        }
    }
}

fn time_range_unit_seconds(i: Input<u8>) -> U8Result<u64> {
    parse!{i;

        string_ignore_case("seconds".as_bytes()) <|>
        string_ignore_case("second".as_bytes()) <|>
        string_ignore_case("secs".as_bytes()) <|>
        string_ignore_case("sec".as_bytes()) <|>
        string_ignore_case("s".as_bytes());

        ret 1
    }
}

fn time_range_unit_minutes(i: Input<u8>) -> U8Result<u64> {
    parse!{i;

        string_ignore_case("minutes".as_bytes()) <|>
        string_ignore_case("minute".as_bytes()) <|>
        string_ignore_case("mins".as_bytes()) <|>
        string_ignore_case("min".as_bytes()) <|>
        string_ignore_case("m".as_bytes());

        // 60 seconds in a minute
        ret 60
    }
}

fn time_range_unit_hours(i: Input<u8>) -> U8Result<u64> {
    parse!{i;

        string_ignore_case("hours".as_bytes()) <|>
        string_ignore_case("hour".as_bytes()) <|>
        string_ignore_case("hrs".as_bytes()) <|>
        string_ignore_case("hr".as_bytes()) <|>
        string_ignore_case("h".as_bytes());

        // 3600 seconds in an hour
        ret 3600
    }
}

fn time_range_unit_days(i: Input<u8>) -> U8Result<u64> {
    parse!{i;

        string_ignore_case("days".as_bytes()) <|>
        string_ignore_case("day".as_bytes()) <|>
        string_ignore_case("dys".as_bytes()) <|>
        string_ignore_case("dy".as_bytes()) <|>
        string_ignore_case("d".as_bytes());

        // 86400 seconds in a day
        ret 86400
    }
}


/* datetime parsers */

enum Meridiem {
    AM,
    PM
}

struct Time {
    // 24-hour format.
    // range from 0 to 23
    hour: u32,

    minute: u32
}

struct ParsedDate {

    // between 1 and 31
    day: u32,

    // between 1 and 12
    month: u32,

    // at least 1
    year: i32
}

struct ParsedDateTime {
    time: Time,
    date: ParsedDate
}

fn parse_datetime(i: Input<u8>, end_of_day: bool) -> U8Result<NaiveDateTime> {

    or(i,
        |i| parse!{i;

            let time = parse_time();
            skip_many1(|i| space_or_tab(i));
            let date = parse_date();


            ret ParsedDateTime {
                time: time,
                date: date
            }
        },
        |i| or(i,
            |i| parse!{i;


                let date = parse_date();
                skip_many1(|i| space_or_tab(i));
                let time = parse_time();

                ret ParsedDateTime {
                    time: time,
                    date: date
                }
            },
            |i| parse!{i;


                let date = parse_date();

                ret {
                    if end_of_day {
                        ParsedDateTime {
                            date: date,
                            time: Time {
                                hour: 23,
                                minute: 59
                            }
                        }
                    } else {
                        ParsedDateTime {
                            date: date,
                            time: Time {
                                hour: 0,
                                minute: 0
                            }
                        }
                    }
                }

            }
        )
    )
    .bind(|i, above: ParsedDateTime| {

        let date = NaiveDate::from_ymd(above.date.year, above.date.month, above.date.day);
        let time = NaiveTime::from_hms(above.time.hour, above.time.minute, 0);
        let date_time = NaiveDateTime::new(date, time);

        i.ret(date_time)
    })
}

fn parse_date(i: Input<u8>) -> U8Result<ParsedDate> {

    parse!{i;

        let month = parse_months();

        skip_many1(|i| space_or_tab(i));

        let day = parse_day();

        or(
            |i| parse!{i;
                skip_many(|i| space_or_tab(i));
                token(b',');
                skip_many(|i| space_or_tab(i));

                ret ()
            },
            |i| parse!{i;
                skip_many1(|i| space_or_tab(i));
                ret ()
            }
        );

        let year = parse_year();

        ret ParsedDate {
            month: month,
            day: day,
            year: year
        }
    }
}

// 5pm
// 5:00pm
// 17:00
fn parse_time(i: Input<u8>) -> U8Result<Time> {

    parse!{i;

        let time = simple_time() <|>
            parse_12_hour_clock() <|>
            parse_24_hour_clock();

        ret time
    }
}

fn simple_time(i: Input<u8>) -> U8Result<Time> {

    parse!{i;
        let hour = parse_12_hour();
        skip_many(|i| space_or_tab(i));
        let ampm: Meridiem = parse_am_pm();

        ret {

            let mut hour: u32 = hour;

            match ampm {
                Meridiem::AM => {
                    if hour == 12 {
                        hour = 0;
                    }
                },
                Meridiem::PM => {
                    if hour != 12 {
                        // 1 to 11
                        hour = hour + 12;
                    }
                }
            };

            Time {
                hour: hour,
                minute: 0
            }
        }
    }
}

fn parse_12_hour_clock(i: Input<u8>) -> U8Result<Time> {

    parse!{i;

        let hour = parse_12_hour();
        token(b':');
        let minute = parse_minute();
        skip_many(|i| space_or_tab(i));
        let ampm: Meridiem = parse_am_pm();

        ret {

            let mut hour: u32 = hour;

            match ampm {
                Meridiem::AM => {
                    if hour == 12 {
                        hour = 0;
                    }
                },
                Meridiem::PM => {
                    if hour != 12 {
                        // 1 to 11
                        hour = hour + 12;
                    }
                }
            };

            Time {
                hour: hour,
                minute: minute
            }
        }

    }
}

fn parse_am_pm(i: Input<u8>) -> U8Result<Meridiem> {
    or(i,
        |i| parse!{i;
            string_ignore_case("pm".as_bytes());
            ret Meridiem::PM;
        },
        |i| parse!{i;
            string_ignore_case("am".as_bytes());
            ret Meridiem::AM;
        }
    )
}

fn parse_24_hour_clock(i: Input<u8>) -> U8Result<Time> {

    or(i,
        |i| parse!{i;

            let hour: u32 = parse_24_hour();
            token(b':');
            let minute: u32 = parse_minute();

            ret Time {
                hour: hour,
                minute: minute
            }
        },
        |i| military_time(i)
    )


}

fn military_time(i: Input<u8>) -> U8Result<Time> {

    // TODO: refactor; haha...
    or(i,
        |i| parse!{i;
            let hour_2: u8 = digit();
            let hour_1: u8 = digit();
            let min_2: u8 = digit();
            let min_1: u8 = digit();

            ret {

                let hour_2: u32 = hour_2 as u32 - 48;
                let hour_1: u32 = hour_1 as u32 - 48;
                let hour = hour_2 * 10 + hour_1;

                let min_2: u32 = min_2 as u32 - 48;
                let min_1: u32 = min_1 as u32 - 48;
                let min = min_2 * 10 + min_1;

                Time {
                    hour: hour,
                    minute: min
                }
            }
        },
        |i| parse!{i;
            let hour_1: u8 = digit();
            let min_2: u8 = digit();
            let min_1: u8 = digit();

            ret {

                let hour_1: u32 = hour_1 as u32 - 48;
                let hour = hour_1;

                let min_2: u32 = min_2 as u32 - 48;
                let min_1: u32 = min_1 as u32 - 48;
                let min = min_2 * 10 + min_1;

                Time {
                    hour: hour,
                    minute: min
                }
            }
        }
    )
    .bind(|i, above:Time| {

        if 0 <= above.hour && above.hour <= 23 && 0 <= above.minute && above.minute <= 59  {
            return i.ret(above);
        }

        // TODO: right usize?
        return i.incomplete(1);
    })
}

fn parse_24_hour(i: Input<u8>) -> U8Result<u32> {

    up_to_two_digits(i)
    .bind(|i, above:u32| {

        if 0 <= above && above <= 23 {
            return i.ret(above);
        }

        // TODO: right usize?
        return i.incomplete(1);
    })

}

fn parse_12_hour(i: Input<u8>) -> U8Result<u32> {

    up_to_two_digits(i)
    .bind(|i, above:u32| {

        if 1 <= above && above <= 12 {
            return i.ret(above);
        }

        // TODO: right usize?
        return i.incomplete(1);
    })

}

fn parse_minute(i: Input<u8>) -> U8Result<u32> {

    two_digits(i)
    .bind(|i, above:u32| {

        if 0 <= above && above <= 59 {
            return i.ret(above);
        }

        // TODO: right usize?
        return i.incomplete(1);
    })

}

fn parse_year(i: Input<u8>) -> U8Result<i32> {

    decimal::<u32>(i)
        .bind(|i, above:u32| {

            if above <= 0 {
                // TODO: right usize?
                return i.incomplete(1);
            }

            i.ret(above as i32)
        })

}

fn parse_day(i: Input<u8>) -> U8Result<u32> {

    up_to_two_digits(i)
    .bind(|i, above:u32| {

        if above <= 0 || above >= 32 {
            // TODO: right usize?
            return i.incomplete(1);
        }

        i.ret(above)
    })

}

fn parse_months(i: Input<u8>) -> U8Result<u32> {

    parse!{i;

        let month: u32 =
            resolve_month("january", 1) <|>
            resolve_month("jan", 1) <|>

            resolve_month("february", 2) <|>
            resolve_month("feb", 2) <|>

            resolve_month("march", 3) <|>
            resolve_month("mar", 3) <|>

            resolve_month("april", 4) <|>
            resolve_month("apr", 4) <|>

            resolve_month("may", 5) <|>

            resolve_month("june", 6) <|>
            resolve_month("jun", 6) <|>

            resolve_month("july", 7) <|>
            resolve_month("jul", 7) <|>

            resolve_month("august", 8) <|>
            resolve_month("aug", 8) <|>

            resolve_month("september", 9) <|>
            resolve_month("sept", 9) <|>
            resolve_month("sep", 9) <|>

            resolve_month("october", 10) <|>
            resolve_month("oct", 10) <|>

            resolve_month("november", 11) <|>
            resolve_month("nov", 11) <|>

            resolve_month("december", 12) <|>
            resolve_month("dec", 12);

        ret month;
    }
}

fn resolve_month<'a>(i: Input<'a, u8>, month: &str, ret_val: u32) -> SimpleResult<'a, u8, u32> {
    parse!{i;
        string_ignore_case(month.as_bytes());
        ret ret_val
    }
}

fn up_to_two_digits(i: Input<u8>) -> U8Result<u32> {
    or(i,
        |i| parse!{i;
            let first_digit: u8 = digit();
            let second_digit: u8 = digit();

            ret {

                let first_digit: u32 = first_digit as u32 - 48;
                let second_digit: u32 = second_digit as u32 - 48;
                let resolved: u32 = first_digit * 10 + second_digit;

                resolved
            }
        },
        |i| parse!{i;
            let first_digit: u8 = digit();

            ret {

                let resolved: u32 = first_digit as u32 - 48;
                resolved
            }
        }
    )
}

fn two_digits(i: Input<u8>) -> U8Result<u32> {
    parse!{i;
        let first_digit: u8 = digit();
        let second_digit: u8 = digit();

        ret {

            let first_digit: u32 = first_digit as u32 - 48;
            let second_digit: u32 = second_digit as u32 - 48;
            let resolved: u32 = first_digit * 10 + second_digit;

            resolved
        }
    }
}

/* Filestats */

#[derive(Debug)]
struct FileStats {

    overdue_tasks: Vec<u64>,
    inbox_tasks: Vec<u64>,
    completed_tasks: Vec<u64>,
    deferred_tasks: Vec<u64>,
    incubate_tasks: Vec<u64>,

    tags: HashSet<String>,
    contexts: HashSet<String>,
    project_paths: HashSet<String>
}

impl FileStats {

    fn new() -> FileStats {

        FileStats {

            overdue_tasks: Vec::new(),
            inbox_tasks: Vec::new(),
            completed_tasks: Vec::new(),
            deferred_tasks: Vec::new(),
            incubate_tasks: Vec::new(),

            tags: HashSet::new(),
            contexts: HashSet::new(),
            project_paths: HashSet::new()
        }
    }

    fn have_tags(&self) -> bool {
        return self.tags.len() > 0;
    }

    fn have_contexts(&self) -> bool {
        return self.contexts.len() > 0;
    }

    fn have_projects(&self) -> bool {
        return self.project_paths.len() > 0;
    }

    fn add_incubate_task_id(&mut self, new_id: u64) {
        self.incubate_tasks.push(new_id);
    }

    fn add_overdue_task_id(&mut self, new_id: u64) {
        self.overdue_tasks.push(new_id);
    }

    fn add_inbox_task_id(&mut self, new_id: u64) {
        self.inbox_tasks.push(new_id);
    }

    fn add_finished_task_id(&mut self, new_id: u64) {
        self.completed_tasks.push(new_id);
    }

    fn add_deferred_task_id(&mut self, new_id: u64) {
        self.deferred_tasks.push(new_id);
    }

    fn add_tag(&mut self, tag: String) {
        self.tags.insert(tag);
    }

    fn add_context(&mut self, context: String) {
        self.contexts.insert(context);
    }

    fn add_project_path(&mut self, path: Vec<String>) {
        self.project_paths.insert(path.join(" / "));
    }

    fn print_tags(&self) -> String {

        let mut tags = Vec::new();

        for tag in self.tags.iter() {
            tags.push(tag.clone());
        }
        return tags.join(", ");
    }

    fn print_contexts(&self) -> String {

        let mut contexts = Vec::new();

        for context in self.contexts.iter() {
            contexts.push(context.clone());
        }
        return contexts.join(", ");
    }

}

/* helpers */

fn count_tasks(inbox: &BTreeMap<i64, Vec<u64>>) -> u64 {

    let mut count = 0;
    for (_, inbox) in inbox.iter() {
        count += inbox.len() as u64;
    }

    return count;
}

fn to_task_id(len: usize) -> i32 {
    len as i32
}

enum RelativeTime {
    Future(i64, String),
    Now(i64, String),
    Past(i64, String)
}

// src: http://stackoverflow.com/a/6109105/412627
fn relative_time(from: i64, to: i64) -> RelativeTime {

    let elapsed_num: u64 = (to - from).abs() as u64;
    let range = Timerange::new(elapsed_num).print(2);
    let elapsed_num = elapsed_num as i64;

    if to > from {
        return RelativeTime::Past(elapsed_num, format!("{} ago", range));
    } else if to == from {
        return RelativeTime::Now(elapsed_num, format!("{} ago", range));
    } else {
        return RelativeTime::Future(elapsed_num, format!("{} into the future", range));
    }
}

struct Timerange {
    range: u64
}

impl Timerange {

    fn new(range: u64) -> Timerange {
        Timerange {
            range: range
        }
    }

    fn floor_time_unit(&self) -> (u64, u64, String) {

        let sec_per_minute: f64 = 60f64;
        let sec_per_hour: f64 = sec_per_minute * 60f64;
        let sec_per_day: f64 = sec_per_hour * 24f64;
        let sec_per_month: f64 = sec_per_day * 30f64;
        let sec_per_year: f64 = sec_per_day * 365f64;

        let mut elapsed = self.range as f64;
        let mut remainder: f64 = 0f64;
        let unit;

        if elapsed < sec_per_minute {
            unit = "second";
        } else if elapsed < sec_per_hour {
            remainder = elapsed % sec_per_minute;
            elapsed = (elapsed / sec_per_minute).floor();
            unit = "minute"
        } else if elapsed < sec_per_day {
            remainder = elapsed % sec_per_hour;
            elapsed = (elapsed / sec_per_hour).floor();
            unit = "hour"
        } else if elapsed < sec_per_month {
            remainder = elapsed % sec_per_day;
            elapsed = (elapsed / sec_per_day).floor();
            unit = "day"
        } else if elapsed < sec_per_year {
            remainder = elapsed % sec_per_month;
            elapsed = (elapsed / sec_per_month).floor();
            unit = "month"
        } else {
            remainder = elapsed % sec_per_year;
            elapsed = (elapsed / sec_per_year).floor();
            unit = "year"
        }

        // pluralize
        let unit = if elapsed <= 1f64 {
            format!("{}", unit)
        } else {
            format!("{}s", unit)
        };

        let elapsed = elapsed as u64;
        let remainder = remainder as u64;

        return (elapsed, remainder, unit);
    }

    fn print(&self, depth: u32) -> String {

        let (elapsed, remainder, unit) = self.floor_time_unit();

        if remainder <= 0 || depth <= 1 {
            return format!("{} {}", elapsed, unit);
        }

        let pretty_remainder = Timerange::new(remainder).print(depth - 1);

        if remainder < 60 || depth <= 2 {
            return format!("{} {} and {}", elapsed, unit, pretty_remainder);
        }


        return format!("{} {} {}", elapsed, unit, pretty_remainder);

    }
}

// TODO: refactor
fn traverse(path: &mut [String], tree: &mut Tree) {

    if path.len() <= 0 {
        return;
    }

    match path.split_first_mut() {
        None => unsafe { debug_unreachable!() },
        Some((first, rest)) => {

            if !tree.contains_key(first) {
                tree.insert(first.clone(), NodeType::Leaf);
            }

            if rest.len() <= 0 {
                return;
            } else {

                let should_replace: bool = match tree.get_mut(first) {
                    None => unsafe { debug_unreachable!("add_project_filter: NodeType not found") },
                    Some(node_type) => {
                        match node_type {
                            &mut NodeType::Leaf => {
                                true
                            },
                            &mut NodeType::Node(_) => {
                                false
                            }
                        }
                    }
                };

                if should_replace {
                    let mut new_tree: Tree = HashMap::new();
                    {
                        let _new_tree = &mut new_tree;
                        traverse(rest, _new_tree);
                    };

                    tree.insert(first.clone(), NodeType::Node(new_tree));
                }
            }
        }
    }
}

// For any leaf in the tree, test if its path (from root to leaf) is a subpath of path.
fn path_satisfies_tree(tree: &Tree, path: &Vec<String>) -> bool {

    let mut current = tree;

    for path_item in path {

        if !current.contains_key(path_item) {
            return false;
        }

        match current.get(path_item) {
            None => {
                return false;
            },
            Some(node_type) => {
                match node_type {
                    &NodeType::Leaf => {
                        // path is super path
                        return true;
                    },
                    &NodeType::Node(ref tree) => {
                        current = tree;
                    }
                }
            }
        };
    }

    // None of the paths from root to leaf are subpaths of path

    return false;
}

/*
Adapted from: https://gist.github.com/m4rw3r/1f43559dcd73bf46e845
Thanks to github.com/m4rw3r for wrapping parsers for line number tracking!
*/

pub trait NumberingType {
    type Token;
    type Position;

    fn update(&mut self, &[Self::Token]);
    fn position(&self) -> Self::Position;
}

#[derive(Debug)]
pub struct LineNumber(u64);

// Semantics: count number of newlines
impl LineNumber {
    pub fn new() -> Self { LineNumber(0) }
}

impl NumberingType for LineNumber {
    type Token    = u8;
    type Position = u64;

    fn update(&mut self, b: &[Self::Token]) {
        self.0 = self.0 + b.iter().filter(|&&c| c == b'\n').count() as u64;
    }

    fn position(&self) -> Self::Position {
        self.0
    }
}

#[derive(Debug)]
pub struct Numbering<'i, T, P, R, E>
  where T: NumberingType,
        P: FnMut(Input<'i, T::Token>) -> ParseResult<'i, T::Token, R, E>,
        R: 'i,
        E: 'i,
        <T as NumberingType>::Token: 'i {
    parser:    P,
    numbering: T,
    _re:       PhantomData<&'i (R, E)>,
}

impl<'i, N, P, R, E> Numbering<'i, N, P, R, E>
  where N: NumberingType,
        P: FnMut(Input<'i, N::Token>) -> ParseResult<'i, N::Token, R, E>,
        R: 'i,
        E: 'i,
        <N as NumberingType>::Position: std::fmt::Debug,
        <N as NumberingType>::Token: 'i {
    pub fn new(n: N, p: P) -> Self {
        Numbering {
            parser:    p,
            numbering: n,
            _re:       PhantomData,
        }
    }

    pub fn parse(&mut self, i: Input<'i, N::Token>) -> ParseResult<'i, N::Token, (N::Position, R), E> {
        use chomp::primitives::InputBuffer;
        use chomp::primitives::InputClone;
        use chomp::primitives::IntoInner;
        use chomp::primitives::State;

        let buf = i.clone();

        match (self.parser)(i.clone()).into_inner() {
            State::Data(remainder, t) => {
                self.numbering.update(&buf.buffer()[..buf.buffer().len() - remainder.buffer().len()]);

                let pos = self.numbering.position();

                remainder.ret((pos, t))
            },
            State::Error(remainder, e) => {
                self.numbering.update(&buf.buffer()[..buf.buffer().len() - remainder.len()]);

                buf.replace(remainder).err(e)
            },
            State::Incomplete(n) => buf.incomplete(n)
        }
    }
}

// Source: https://gist.github.com/dashed/9d18b7e4cc351a7feabc89897a58baff
#[test]
fn line_numbering() {
    use chomp::take;
    use std::cell::Cell;
    use chomp::buffer::{IntoStream, Stream, StreamError};

    let mut data = b"abc\nc\n\ndef".into_stream();
    // Just some state to make sure we are called the correct number of times:
    let i = Cell::new(0);
    let p = |d| {
        i.set(i.get() + 1);
        take(d, 2)
    };
    let mut n = Numbering::new(LineNumber::new(), p);
    // If we could implement FnMut for Numbering then we would be good, but we need to wrap now:
    let mut m = |i| n.parse(i);

    assert_eq!(data.parse(&mut m), Ok((0, &b"ab"[..])));
    assert_eq!(i.get(), 1);
    assert_eq!(data.parse(&mut m), Ok((1, &b"c\n"[..])));
    assert_eq!(i.get(), 2);
    assert_eq!(data.parse(&mut m), Ok((2, &b"c\n"[..])));
    assert_eq!(i.get(), 3);
    assert_eq!(data.parse(&mut m), Ok((3, &b"\nd"[..])));
    assert_eq!(i.get(), 4);
    assert_eq!(data.parse(&mut m), Ok((3, &b"ef"[..])));
    assert_eq!(i.get(), 5);
    assert_eq!(data.parse(&mut m), Err(StreamError::EndOfInput));
    assert_eq!(i.get(), 5);
}
