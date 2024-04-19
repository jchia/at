use chrono::{Duration, Local, NaiveTime, Timelike};
use nix::unistd::execvp;
use std::convert::Infallible;
use std::env;
use std::ffi::CString;
use std::process::ExitCode;
use std::thread::sleep;
use std::time::Duration as StdDuration;

// This program takes a duration or time-of-day, a command and zero or more
// arguments. It waits until the specified time and then executes the command
// with the specified arguments.

fn run() -> Result<Infallible, (u8, Box<str>)> {
    let args: Vec<String> = env::args().collect();
    if args.len() < 3 {
        return Err((
            1,
            format!("Usage: {} [+]<HH:MM[:SS]> <command...>", args[0]).into(),
        ));
    }

    let time_str = &args[1];
    let command_args = &args[2..];
    let is_offset = time_str.starts_with('+');
    let parsed_time_str = if is_offset { &time_str[1..] } else { time_str };

    let time_format = if parsed_time_str.len() > 5 {
        "%H:%M:%S"
    } else {
        "%H:%M"
    };

    let parsed_time = NaiveTime::parse_from_str(parsed_time_str, time_format)
        .map_err(|_| (2, "Invalid time format".into()))?;

    let wait_seconds = if is_offset {
        parsed_time.num_seconds_from_midnight() as u64
    } else {
        let now = Local::now().naive_local();
        let target_time = now.date().and_time(parsed_time);
        let wait_time = if target_time > now {
            target_time - now
        } else {
            target_time + Duration::days(1) - now
        };
        wait_time.num_seconds() as u64
    };

    let cstrings = command_args
        .iter()
        .map(|s| CString::new(s.as_bytes()).map_err(|_| (255, "CString::new failed".into())))
        .collect::<Result<Vec<CString>, (u8, Box<str>)>>()?;

    if wait_seconds >= 60 {
        println!("Waiting for {} seconds", wait_seconds);
    }
    sleep(StdDuration::from_secs(wait_seconds));

    execvp(&cstrings[0], &cstrings).map_err(|_| (3, "execvp() failed".into()))?;
    unreachable!();
}

fn main() -> ExitCode {
    match run() {
        Err((code, msg)) => {
            eprintln!("{}", msg);
            return ExitCode::from(code);
        }
        Ok(_) => unreachable!(),
    }
}
