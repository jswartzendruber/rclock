use std::fs::OpenOptions;
use std::io::prelude::*;
use std::time::Duration;
use std::time::SystemTime;

extern crate shellexpand;

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
        Duration::new(
            string
                .parse::<u64>()
                .expect(&format!("Bad unix timestamp in ~/.rclock-{}", self.name)),
            0,
        )
    }

    fn is_line_started(&self, line: Option<&str>) -> (bool, String) {
        let last = match line {
            Some(l) => l,
            None => "",
        };

        let chunks: Vec<&str> = last.split(',').collect();
        if chunks.len() == 2 {
            return (true, chunks[0].to_string());
        } else {
            return (false, chunks[0].to_string());
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
        .open(shellexpand::tilde(&format!("~/.rclock-{}", project.name)).to_string())
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

    let time_from_epoch = SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .expect("Failure getting unix time.")
        .as_secs();

    let mut contents = String::new();
    file.read_to_string(&mut contents)
        .expect(&format!("Failed to read ~/.rclock-{} file.", project.name));
    let (already_begun, last_start_timestamp) = project.is_line_started(contents.lines().last());

    match project.action {
        Action::Begin => {
            if already_begun {
                println!("Error: Clock is already started!");
            } else {
                println!("Clock started.");
                file.write(format!("{},", time_from_epoch).as_bytes())
                    .expect("Failed to write to file.");
            }
        }
        Action::End => {
            if !already_begun {
                println!("Error: Clock has not been started!");
            } else {
                println!("Clock stopped.");
                file.write(format!("{}\n\r", time_from_epoch).as_bytes())
                    .expect("Failed to write to file.");
                let begin = project.parse_timestamp(last_start_timestamp.trim());
                let end = Duration::new(time_from_epoch, 0);
                display_duration(end.saturating_sub(begin), "Time tracked this session:");
            }
        }
        Action::Summarize => {
            let one_week_ago = time_from_epoch.saturating_sub(86_400 * 7);
            let one_day_ago = time_from_epoch.saturating_sub(86_400);

            let mut last_week = vec![];
            let mut all_time = vec![];
            let mut last_day = vec![];

            for line in contents.lines() {
                let chunks: Vec<&str> = line.split(',').collect();
                if chunks.len() == 2 {
                    if chunks[1] == "" {
                        // Incomplete line
                        let begin = project.parse_timestamp(chunks[0]);
                        let end = Duration::new(time_from_epoch, 0);
                        let session = end.saturating_sub(begin);

                        all_time.push(session);
                        if begin > Duration::new(one_week_ago, 0) {
                            last_week.push(session);
                        }
                        if begin > Duration::new(one_day_ago, 0) {
                            last_day.push(session);
                        }
                    } else {
                        // Complete line
                        let begin = project.parse_timestamp(chunks[0].trim());
                        let end = project.parse_timestamp(chunks[1].trim());
                        let session = end.saturating_sub(begin);

                        all_time.push(session);
                        if begin > Duration::new(one_week_ago, 0) {
                            last_week.push(session);
                        }
                        if begin > Duration::new(one_day_ago, 0) {
                            last_day.push(session);
                        }
                    }
                }
            }

            display_duration(all_time.iter().sum(), "Total time tracked:");
            display_duration(last_week.iter().sum(), "Last week:");
            display_duration(last_day.iter().sum(), "Today:");
        }
    }
}

fn display_duration(duration: Duration, prefix: &str) {
    let mut seconds = duration.as_secs();
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
