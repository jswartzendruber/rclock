use chrono::{Datelike, Duration, Local, TimeZone};
use shellexpand::tilde;
use std::fs::OpenOptions;
use std::io::prelude::*;

fn print_usage() {
    println!("Usage:");
    println!("  Argument 1: Project name");
    println!("  Argument 2: Action");
    println!("  Actions:");
    println!("    b = begin clock");
    println!("    e = end clock");
    println!("    s = summarize time");
    println!("Example: rclock project1 b");
}

struct Project {
    name: String,
    action: Action,
}

impl Project {
    fn new(name: String, action: Action) -> Self {
        Self { name, action }
    }

    fn parse_timestamp(&self, string: &str) -> Duration {
        Duration::seconds(
            string
                .parse::<i64>()
                .unwrap_or_else(|_| panic!("Bad unix timestamp in ~/.rclock-{}", self.name)),
        )
    }

    fn is_line_started(&self, line: Option<&str>) -> (bool, String) {
        let last = line.unwrap_or("");

        let chunks: Vec<&str> = last.split(',').collect();
        if chunks.len() == 2 {
            (true, chunks[0].to_string())
        } else {
            (false, chunks[0].to_string())
        }
    }
}

enum Action {
    Summarize,
    Begin,
    End,
}

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    if args.len() < 2 {
        error("Incorrect arguments.", true);
        std::process::exit(1);
    }

    let action = match args[1].as_str() {
        "s" => Action::Summarize,
        "b" => Action::Begin,
        "e" => Action::End,
        _ => {
            error("Unknown option.", true);
            std::process::exit(1);
        }
    };
    let project = Project::new(args[0].clone(), action);

    let mut file = match OpenOptions::new()
        .read(true)
        .append(true)
        .create(true)
        .open(tilde(&format!("~/.rclock-{}", project.name)).to_string())
    {
        Ok(f) => f,
        Err(e) => {
            error(
                &format!("Failed to open ~/.rclock-{} file: {}", project.name, e),
                false,
            );
            std::process::exit(1);
        }
    };

    let now = Local::now().timestamp();

    // Calculate seconds from 12:00AM this morning. Use it to show time spent today.
    let now_today = Local::now();
    let today = now_today
        .signed_duration_since(
            Local
                .with_ymd_and_hms(
                    now_today.year(),
                    now_today.month(),
                    now_today.day(),
                    0,
                    0,
                    0,
                )
                .unwrap(),
        )
        .num_seconds();
    let now_week = Local.timestamp_opt(now - (86400 * 6), 0).unwrap();
    let week_ago = now_today
        .signed_duration_since(
            Local
                .with_ymd_and_hms(now_week.year(), now_week.month(), now_week.day(), 0, 0, 0)
                .unwrap(),
        )
        .num_seconds();

    let mut contents = String::new();
    file.read_to_string(&mut contents)
        .unwrap_or_else(|_| panic!("Failed to read ~/.rclock-{} file.", project.name));
    let (already_begun, last_start_timestamp) = project.is_line_started(contents.lines().last());

    match project.action {
        Action::Begin => {
            if already_begun {
                println!("Error: Clock is already started!");
            } else {
                println!("Clock started.");
                file.write_all(format!("{},", now).as_bytes())
                    .expect("Failed to write to file.");
            }
        }
        Action::End => {
            if !already_begun {
                println!("Error: Clock has not been started!");
            } else {
                println!("Clock stopped.");
                file.write_all(format!("{}\n\r", now).as_bytes())
                    .expect("Failed to write to file.");
                let begin = project.parse_timestamp(last_start_timestamp.trim());
                let end = Duration::seconds(now);
                display_duration((end - begin).num_seconds(), "Time tracked this session:");
            }
        }
        Action::Summarize => {
            let one_week_ago = Duration::seconds(now - week_ago);
            let one_day_ago = Duration::seconds(now - today);

            let mut last_week = vec![];
            let mut all_time = vec![];
            let mut last_day = vec![];

            for line in contents.lines() {
                let chunks: Vec<&str> = line.split(',').collect();
                if chunks.len() == 2 {
                    if chunks[1].is_empty() {
                        // Incomplete line
                        let begin = project.parse_timestamp(chunks[0]);
                        let end = Duration::seconds(now);
                        let session = end - begin;

                        all_time.push(session);
                        if begin > one_week_ago {
                            last_week.push(session);
                        }
                        if begin > one_day_ago {
                            last_day.push(session);
                        }
                    } else {
                        // Complete line
                        let begin = project.parse_timestamp(chunks[0].trim());
                        let end = project.parse_timestamp(chunks[1].trim());
                        let session = end - begin;

                        all_time.push(session);
                        if begin > one_week_ago {
                            last_week.push(session);
                        }
                        if begin > one_day_ago {
                            last_day.push(session);
                        }
                    }
                }
            }

            display_duration(
                all_time.iter().map(|x| x.num_seconds()).sum(),
                "Total time tracked:",
            );
            display_duration(
                last_week.iter().map(|x| x.num_seconds()).sum(),
                "Last week:",
            );
            display_duration(last_day.iter().map(|x| x.num_seconds()).sum(), "Today:");
        }
    }
}

fn display_duration(secs: i64, prefix: &str) {
    let mut seconds = secs;
    let hours = seconds / 3600;
    seconds -= hours * 3600;
    let minutes = seconds / 60;
    seconds -= minutes * 60;
    println!(
        "{} {} hours, {} minutes, {} seconds.",
        prefix, hours, minutes, seconds
    );
}

fn error(message: &str, show_usage: bool) {
    println!("{}", message);
    if show_usage {
        print_usage();
    }
}
