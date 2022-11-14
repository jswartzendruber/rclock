use std::time::SystemTime;
use std::fs::OpenOptions;
use std::io::prelude::*;
use std::time::Duration;

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

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    if args.len() < 2 {
	println!("Incorrect arguments.");
	print_usage();
	std::process::exit(1);
    }
    let project = &args[0];

    let mut file = match OpenOptions::new()
	.read(true)
	.append(true)
	.create(true)
	.open(shellexpand::tilde(&format!("~/.rclock-{}", project)).to_string()) {
	    Ok(f) => f,
	    Err(e) => {
		println!("Failed to open ~/.rclock-{} file: {}", project, e);
		std::process::exit(1);
	    }
	};

    let time_from_epoch = SystemTime::now()
	.duration_since(SystemTime::UNIX_EPOCH)
	.expect("Failure getting unix time.")
	.as_secs();

    let mut contents = String::new();
    file.read_to_string(&mut contents).expect(&format!("Failed to read ~/.rclock-{} file.", project));
    let last = match contents.lines().last() {
	Some(l) => l,
	None => "",
    };

    let chunks: Vec<&str> = last.split(',').collect();
    let already_begun = if chunks.len() == 2 {
	true
    } else {
	false
    };

    match args[1].as_str() {
	"b" => {
	    if already_begun {
		println!("Error: Clock is already started!");
	    } else {
		println!("Clock started.");
		file.write(format!("{},", time_from_epoch).as_bytes()).expect("Failed to write to file.");
	    }
	},
	"e" => {
	    if !already_begun {
		println!("Error: Clock has not been started!");
	    } else {
		println!("Clock stopped.");
		file.write(format!("{}\n\r", time_from_epoch).as_bytes()).expect("Failed to write to file.");
		let begin = Duration::new(
		    chunks[0].trim().parse::<u64>()
			.expect(&format!("Bad unix timestamp in ~/.rclock-{}", project)), 0);
		let end = Duration::new(time_from_epoch, 0);
		display_duration(end.saturating_sub(begin), "Time tracked this session:");
	    }
	}
	"s" => {
	    let mut all_time_duration = Duration::new(0, 0);
	    for line in contents.lines() {
		let chunks: Vec<&str> = line.split(',').collect();
		if chunks.len() == 2 {
		    if chunks[1] == "" {
			// Incomplete line
			let unix_str_1 = chunks[0];
			let begin = Duration::new(
			    unix_str_1.parse::<u64>()
				.expect(&format!("Bad unix timestamp in ~/.rclock-{}", project)), 0);
			let end = Duration::new(time_from_epoch, 0);
			all_time_duration = all_time_duration.saturating_add(end.saturating_sub(begin));
		    } else {
			// Complete line
			let unix_str_1 = chunks[0].trim();
			let unix_str_2 = chunks[1].trim();
			let begin = Duration::new(
			    unix_str_1.parse::<u64>()
				.expect(&format!("Bad unix timestamp in ~/.rclock-{}", project)), 0);
			let end = Duration::new(
			    unix_str_2.parse::<u64>()
				.expect(&format!("Bad unix timestamp in ~/.rclock-{}", project)), 0);
			all_time_duration = all_time_duration.saturating_add(end.saturating_sub(begin));
		    }
		}
	    }
	    display_duration(all_time_duration, "Total time tracked:");
	}
	_ => {
	    println!("Unexpected argument '{}'!", args[0]);
	    print_usage();
	    std::process::exit(1);
	}
    }

    fn display_duration(duration: Duration, prefix: &str) {
	let mut seconds = duration.as_secs();
	let hours = seconds / 3600;
	seconds -= hours * 3600;
	let minutes = seconds / 60;
	seconds -= minutes * 60;
	println!("{} {} hours, {} minutes, {} seconds.", prefix, hours, minutes, seconds);
    }
}
