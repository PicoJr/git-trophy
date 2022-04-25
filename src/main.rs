#[macro_use]
extern crate clap;
extern crate anyhow;

mod cli;

use std::collections::{HashMap, HashSet};
use std::path::{PathBuf};
use std::str::FromStr;
use anyhow::bail;
use chrono::{Datelike, TimeZone, Utc};
use git2::Repository;
use json::object;

type Year = i32;
type DaysYear = i64;
type CommitYearDay = (Year, DaysYear);
// year, num_days_year
type CommitCount = usize;

fn commit_time_as_date(time: &git2::Time) -> chrono::DateTime<Utc> {
    Utc.timestamp(time.seconds(), 0)
}

fn build_history_heightmap(repo: &Repository, year_selected: Option<i32>) -> anyhow::Result<Vec<CommitCount>> {
    let mut walker = repo.revwalk()?;
    walker.push_glob("*")?;

    let mut commit_history: HashMap<CommitYearDay, CommitCount> = HashMap::new();
    let mut years: HashSet<Year> = HashSet::new();

    for oid in walker {
        if let Ok(oid) = oid {
            if let Ok(commit) = repo.find_commit(oid) {
                let commit_datetime = commit_time_as_date(&commit.time());
                let (commit_year, commit_month, commit_day) = (commit_datetime.year(), commit_datetime.month(), commit_datetime.day());
                if year_selected == Some(commit_year) || year_selected.is_none() {
                    years.insert(commit_year);
                }
                let num_days_year = (Utc.ymd(commit_year, commit_month, commit_day) - Utc.ymd(commit_year, 1, 1)).num_days();
                if let Some(count) = commit_history.get_mut(&(commit_year, num_days_year)) {
                    *count += 1;
                } else {
                    commit_history.insert((commit_year, num_days_year), 1);
                }
            }
        }
    }

    let commit_heightmap: Vec<CommitCount> = (0..365).map(|year_day|
        years.iter().map(
            |year| {
                commit_history.get(&(*year, year_day)).unwrap_or(&0)
            }
        ).sum()
    ).collect();

    Ok(commit_heightmap)
}

fn main() -> anyhow::Result<()> {
    let matches = cli::get_command().get_matches();
    let repository_path = matches.value_of("repository").expect("compulsory argument");
    let year = matches.value_of("year").map(|s| i32::from_str(s));
    let year_selected = match year {
        None => None,
        Some(Err(e)) => bail!("{}", e),
        Some(Ok(y)) => Some(y)
    };
    let repository_path = PathBuf::from_str(repository_path)?;
    let repo = match Repository::open(repository_path.as_path()) {
        Ok(repo) => repo,
        Err(e) => panic!("failed to open: {}", e),
    };

    let commit_heightmap = build_history_heightmap(&repo, year_selected)?;

    let instantiated = object! {
        // quotes on keys are optional
        "year": year_selected,
        commits: commit_heightmap,
    };

    println!("json: {:?}", json::stringify(instantiated));

    Ok(())
}