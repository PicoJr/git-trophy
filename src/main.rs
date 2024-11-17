#[macro_use]
extern crate clap;
extern crate anyhow;

mod cli;
mod model;

use crate::model::{build_trophy, GeometryConfig};
use anyhow::{anyhow, bail};
use chrono::{Datelike, MappedLocalTime, TimeZone, Utc};
use git2::Repository;
use std::cmp::min;
use std::collections::{HashMap, HashSet};
use std::path::{Path, PathBuf};
use std::str::FromStr;

type Year = i32;
type DaysYear = i64;
type CommitYearDay = (Year, DaysYear);
// year, num_days_year
type CommitCount = usize;

fn commit_time_as_date(time: &git2::Time) -> anyhow::Result<chrono::DateTime<Utc>> {
    match Utc.timestamp_opt(time.seconds(), 0) {
        MappedLocalTime::Single(s) => Ok(s),
        MappedLocalTime::Ambiguous(_, _) => Err(anyhow!("ambiguous timestamp")),
        MappedLocalTime::None => Err(anyhow!("failed to convert timestamp to date")),
    }
}

fn build_history_heightmap(
    repo: &Repository,
    year_selected: Option<i32>,
    commiter_names_selected: &Option<Vec<String>>,
) -> anyhow::Result<Vec<CommitCount>> {
    let mut walker = repo.revwalk()?;
    walker.push_glob("*")?;

    let mut commit_history: HashMap<CommitYearDay, CommitCount> = HashMap::new();
    let mut years: HashSet<Year> = HashSet::new();

    for oid in walker.flatten() {
        if let Ok(commit) = repo.find_commit(oid) {
            let commit_datetime = commit_time_as_date(&commit.time())?;
            let (commit_year, commit_month, commit_day) = (
                commit_datetime.year(),
                commit_datetime.month(),
                commit_datetime.day(),
            );
            if year_selected == Some(commit_year) || year_selected.is_none() {
                years.insert(commit_year);
            }
            let commit_date = Utc.with_ymd_and_hms(commit_year, commit_month, commit_day, 0, 0, 0);
            let first_january = Utc.with_ymd_and_hms(commit_year, 1, 1, 0, 0, 0);
            let num_days_year = match (commit_date, first_january) {
                (MappedLocalTime::None, _) => Err(anyhow!("days for commit")),
                (_, MappedLocalTime::None) => Err(anyhow!("days for commit")),
                _ => Ok((commit_date.unwrap() - first_january.unwrap()).num_days()),
            }?;

            let committer_selected = match (commiter_names_selected, commit.committer().name()) {
                (None, _) => true,
                (_, None) => false,
                (Some(names), Some(name)) => names.contains(&String::from(name)),
            };
            if committer_selected {
                if let Some(count) = commit_history.get_mut(&(commit_year, num_days_year)) {
                    *count += 1;
                } else {
                    commit_history.insert((commit_year, num_days_year), 1);
                }
            }
        }
    }

    let commit_heightmap: Vec<CommitCount> = (0..365)
        .map(|year_day| {
            years
                .iter()
                .map(|year| *commit_history.get(&(*year, year_day)).unwrap_or(&0))
                .sum()
        })
        .collect();

    Ok(commit_heightmap)
}

fn main() -> anyhow::Result<()> {
    let matches = cli::get_command().get_matches();
    let repositories_paths = matches
        .values_of("repository")
        .expect("compulsory argument");
    let year = matches.value_of("year").map(i32::from_str);
    let clip = matches.value_of("clip").map(usize::from_str);
    let output_path = matches.value_of("output").unwrap_or("trophy");
    let output_path = PathBuf::from_str(output_path)?;

    let side_text = matches.value_of("side_text").map(String::from);
    let font_path = matches.value_of("ttf_font_path").map(Path::new);

    if side_text.is_some() && font_path.is_none() {
        bail!("a side text was provided but no font was provided, please provide a font to use using `--font`");
    }

    let commiter_names_selected: Option<Vec<String>> = matches
        .values_of("names")
        .map(|names| names.into_iter().map(String::from).collect());
    let year_selected = match year {
        None => None,
        Some(Err(e)) => bail!("{}", e),
        Some(Ok(y)) => Some(y),
    };
    let clip_commit = match clip {
        None => None,
        Some(Err(e)) => bail!("{}", e),
        Some(Ok(y)) => Some(y),
    };
    let commit_heightmaps: anyhow::Result<Vec<Vec<CommitCount>>> = repositories_paths
        .into_iter()
        .map(|repository_path| {
            let repository_path = PathBuf::from_str(repository_path)?;
            let repo = match Repository::open(repository_path.as_path()) {
                Ok(repo) => repo,
                Err(e) => panic!("failed to open: {}", e),
            };
            build_history_heightmap(&repo, year_selected, &commiter_names_selected)
        })
        .collect::<anyhow::Result<Vec<Vec<CommitCount>>>>();

    let commit_heightmaps: Vec<Vec<CommitCount>> = commit_heightmaps?;

    let commit_heightmap: Vec<CommitCount> = (0..365)
        .map(|year_day| {
            commit_heightmaps
                .iter()
                .map(|heightmap| {
                    let commits = *heightmap.get(year_day).unwrap_or(&0);
                    match &clip_commit {
                        None => commits,
                        Some(max_commits) => min(*max_commits, commits),
                    }
                })
                .sum()
        })
        .collect();

    build_trophy(
        &commit_heightmap,
        side_text,
        font_path,
        &output_path,
        GeometryConfig::default(),
    )?;

    Ok(())
}
