use chrono::prelude::*;
use chrono::{DateTime, Duration, Local, TimeZone};
use std::process::Command;
use structopt::StructOpt;

#[derive(StructOpt)]
struct GitCalendar {
    #[structopt(short = "a", long = "author")]
    author: Option<String>,
}

#[derive(Copy, Clone)]
enum CommitFreq {
    No,
    Low,
    Mid,
    High,
    VeryHigh,
}

fn first_day() -> DateTime<Local> {
    let today = Local::now();
    let one_year_ago = Local
        .ymd(today.year() - 1, today.month(), today.day())
        .and_hms(0, 0, 0);
    one_year_ago - Duration::days((one_year_ago.weekday() as i64) + 1)
}

fn collect_commit_days(author: &Option<String>) -> Result<Vec<DateTime<Local>>, String> {
    let mut args: Vec<String> = vec![
        "log".to_string(),
        "--no-merges".to_string(),
        "--pretty=format:%at".to_string(),
        "--since=13 months".to_string(),
    ];
    if let Some(name) = author {
        args.push(format!("--author={}", name));
    }

    let ret = Command::new("git").args(&args).output();
    let output = match ret {
        Ok(o) => o,
        Err(e) => return Err(e.to_string()),
    };

    if !output.status.success() {
        return Err("git log returns error".to_string());
    }

    let output_str = match String::from_utf8(output.stdout) {
        Ok(str) => str,
        Err(e) => return Err(e.to_string()),
    };

    let commit_days: Vec<DateTime<Local>> = output_str
        .lines()
        .filter_map(|s| s.parse::<i64>().ok())
        .map(|epoch| Local.timestamp(epoch, 0))
        .collect();

    Ok(commit_days)
}

fn count_commits_per_day(commit_days: &Vec<DateTime<Local>>) -> Vec<i32> {
    let first = first_day();
    let today = Local::now();
    let last = Local
        .ymd(today.year(), today.month(), today.day())
        .and_hms(23, 59, 59);
    let len = last.signed_duration_since(first).num_days();

    let mut ret: Vec<i32> = vec![0; (len + 1) as usize];
    for &day in commit_days {
        let diff = last.signed_duration_since(day).num_days();
        if diff >= len {
            continue;
        }

        ret[(len - diff) as usize] += 1;
    }

    ret
}

fn normalize_commits(commits: &Vec<i32>) -> Vec<CommitFreq> {
    let &max = commits.iter().max().unwrap();
    commits
        .iter()
        .map(|&num| {
            let val = (num as f64) / (max as f64);
            if val == 0.0 {
                CommitFreq::No
            } else if val < 0.25 {
                CommitFreq::Low
            } else if val < 0.5 {
                CommitFreq::Mid
            } else if val < 0.75 {
                CommitFreq::High
            } else {
                CommitFreq::VeryHigh
            }
        })
        .collect()
}

fn print_square(freq: CommitFreq) {
    let color = match freq {
        CommitFreq::No => 237,
        CommitFreq::Low => 139,
        CommitFreq::Mid => 40,
        CommitFreq::High => 190,
        CommitFreq::VeryHigh => 1,
    };
    let square = '\u{25fc}';

    print!("\x1b[38;5;{}m{} \x1b[0m", color, square);
}

impl GitCalendar {
    fn display(&self) -> Result<(), String> {
        let commit_days = collect_commit_days(&self.author)?;
        let commits = count_commits_per_day(&commit_days);
        let freqs = normalize_commits(&commits);

        let first = first_day();
        let last = Local::now();

        let diff = last.signed_duration_since(first);
        let days = diff.num_days() + 1;
        let weeks = if days % 7 == 0 {
            days / 7
        } else {
            days / 7 + 1
        };

        let months = vec![
            "Jan", "Feb", "Mar", "Apr", "May", "Jun", "Jul", "Aug", "Sep", "Oct", "Nov", "Dec",
        ];
        print!("    ");
        print!("{}", months[(first.month() - 1) as usize]);

        let month_week = if first.day() % 7 == 0 {
            first.day() / 7
        } else {
            first.day() / 7 + 1
        };
        print!("{}", " ".repeat(4_usize - month_week as usize));

        let mut month_str: Vec<&str> = vec![];
        for i in 0..12 {
            let index = (first.month() + i as u32) % 12;
            month_str.push(months[index as usize]);
        }

        println!("{}", month_str.join("      "));

        for weekday in 0..7 {
            if weekday == 1 {
                print!("Mon ");
            } else if weekday == 3 {
                print!("Wed ");
            } else if weekday == 5 {
                print!("Fri ");
            } else {
                print!("    ");
            }

            for week in 0..weeks {
                let index = (weekday + week * 7) as usize;
                if index >= freqs.len() {
                    continue;
                }

                print_square(freqs[index]);
            }

            println!("");
        }

        Ok(())
    }
}

fn main() -> Result<(), String> {
    let cal = GitCalendar::from_args();
    cal.display()?;

    Ok(())
}
