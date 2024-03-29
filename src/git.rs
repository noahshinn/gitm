use crate::fmt::{colorize_string, indent_string, Color};
use chrono::{DateTime, Utc};
use std::fmt::{Display, Formatter};
use std::hash::{Hash, Hasher};
use std::process::Command;
use unidiff::PatchSet;

pub struct Client;

#[derive(Debug, Clone)]
pub struct Commit {
    pub author: Author,
    pub date: DateTime<Utc>,
    pub title: String,
    pub body: String,
    pub sha: String,
    pub patch_set: PatchSet,
    pub display_mode: CommitDisplayMode,
}

impl Commit {
    pub fn mock_git_log_fmt(&self) -> String {
        format!(
            "{}
Author: {}{}
Date: {}

{}",
            colorize_string(format!("commit {}", self.sha).as_str(), Color::Yellow),
            self.author.name.clone().unwrap_or("".to_string()),
            self.author
                .email
                .as_ref()
                .map_or_else(|| "".to_string(), |email| format!(" <{}>", email)),
            self.date.to_rfc2822(),
            format!(
                "{}{}",
                indent_string(self.title.as_str(), 4),
                if self.body != "" {
                    indent_string(format!("\n\n{}", self.body).as_str(), 4)
                } else {
                    "".to_string()
                }
            )
        )
    }
}

#[derive(Debug, Clone)]
pub enum CommitDisplayMode {
    Title,
    Body,
    TitleAndBody,
    PatchSetAdd,
    PatchSetRemove,
    PatchSetAll,
}

impl Display for Commit {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self.display_mode {
            CommitDisplayMode::Title => write!(f, "{}", self.title),
            CommitDisplayMode::Body => write!(f, "{}", self.body),
            CommitDisplayMode::TitleAndBody => write!(f, "{}\n\n{}", self.title, self.body),
            CommitDisplayMode::PatchSetAdd => {
                let mut added_line_content = Vec::<String>::new();
                for file in self.patch_set.files() {
                    for hunk in file.hunks() {
                        for line in hunk.lines() {
                            if line.is_added() {
                                added_line_content.push(line.value.clone());
                            }
                        }
                    }
                }
                write!(f, "{}", added_line_content.join("\n"))
            }
            CommitDisplayMode::PatchSetRemove => {
                let mut removed_line_content = Vec::<String>::new();
                for file in self.patch_set.files() {
                    for hunk in file.hunks() {
                        for line in hunk.lines() {
                            if line.is_removed() {
                                removed_line_content.push(line.value.clone());
                            }
                        }
                    }
                }
                write!(f, "{}", removed_line_content.join("\n"))
            }
            CommitDisplayMode::PatchSetAll => write!(f, "{}", self.patch_set),
        }
    }
}

#[derive(Debug, Clone)]
pub struct Author {
    pub name: Option<String>,
    pub username: Option<String>,
    pub email: Option<String>,
}

impl PartialEq for Author {
    fn eq(&self, other: &Self) -> bool {
        self.name == other.name
    }
}

impl Eq for Author {}

impl Hash for Author {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.name.hash(state);
    }
}

const DELIMITER: &str = "|||";

const GIT_LOG_PARSE_FIELDS: [&str; 6] = ["%an", "%ae", "%aD", "%s", "%b", "%H"];
const DEFAULT_GIT_LOG_SINCE: &str = "3 months ago";
const MIN_LARGE_GIT_REPO_NUM_COMMITS: usize = 1000;

#[derive(Debug, Clone)]
pub struct FilterConfig {
    pub author: Option<Author>,
    pub date_range: Option<(Option<DateTime<Utc>>, Option<DateTime<Utc>>)>,
    pub git_log_get_all: bool,
}

impl Client {
    pub fn new() -> Client {
        Client
    }

    pub fn get_number_of_commits(&self) -> Result<usize, Box<dyn std::error::Error>> {
        let output = Command::new("git")
            .arg("rev-list")
            .arg("--count")
            .arg("HEAD")
            .output()?;
        if !output.status.success() {
            return Err("Failed to get git rev-list".into());
        }
        let stdout = String::from_utf8(output.stdout)?;
        let count = stdout.trim().parse::<usize>()?;
        Ok(count)
    }

    pub fn get_all_commits(
        &self,
        config: Option<FilterConfig>,
    ) -> Result<Vec<Commit>, Box<dyn std::error::Error>> {
        let format = format!("--pretty=format:{}", GIT_LOG_PARSE_FIELDS.join(DELIMITER));
        let mut binding = Command::new("git");
        let cmd = binding.arg("log").arg("-uz").arg(format);
        if let Some(config) = &config {
            if !config.git_log_get_all {
                let num_commits = self.get_number_of_commits()?;
                if num_commits > MIN_LARGE_GIT_REPO_NUM_COMMITS {
                    cmd.arg(format!("--since=\"{}\"", DEFAULT_GIT_LOG_SINCE));
                }
            }
        }
        let output = cmd.output()?;
        if !output.status.success() {
            return Err("Failed to get git log".into());
        }
        let stdout = String::from_utf8(output.stdout)?;
        let commit_data_split = stdout.split("\0").collect::<Vec<&str>>();
        let mut commits = Vec::with_capacity(commit_data_split.len());
        for commit_data in commit_data_split {
            let (udiff_strings, remaining_log) = split_raw_git_log_output(commit_data);
            if udiff_strings.is_empty() || remaining_log.is_empty() {
                continue;
            }
            let mut patch_set = PatchSet::new();
            patch_set.parse(udiff_strings.join("\n")).unwrap();
            let mut parts = remaining_log
                .split(DELIMITER)
                .collect::<Vec<&str>>()
                .into_iter();
            if parts.len() != GIT_LOG_PARSE_FIELDS.len() {
                continue;
            }
            let author_name = parts.next().unwrap().to_string();
            let author_email: Option<String> = match parts.next().unwrap() {
                "" => None,
                email => Some(email.to_string()),
            };
            let date_raw = parts.next().unwrap().trim().to_string();
            let title = parts.next().unwrap().trim().to_string();
            let body = parts.next().unwrap().trim().to_string();
            let sha = parts.next().unwrap().trim().to_string();
            let date = DateTime::parse_from_rfc2822(&date_raw)?.with_timezone(&Utc);
            let author = Author {
                name: Some(author_name),
                username: None,
                email: author_email,
            };
            if let Some(config) = &config {
                if let Some(filter_author) = &config.author {
                    if filter_author != &author {
                        continue;
                    }
                }
                if let Some((date_since, date_after)) = config.date_range {
                    if let Some(start_date) = date_since {
                        if date < start_date {
                            continue;
                        }
                    }
                    if let Some(end_date) = date_after {
                        if date > end_date {
                            continue;
                        }
                    }
                }
            }
            commits.push(Commit {
                author,
                date,
                title,
                body,
                sha,
                patch_set,
                display_mode: CommitDisplayMode::TitleAndBody,
            });
        }
        commits.reverse();
        Ok(commits)
    }

    pub fn get_all_authors(&self) -> Result<Vec<Author>, Box<dyn std::error::Error>> {
        let output = Command::new("git")
            .arg("log")
            .arg(format!("--pretty=format:%an{}%ae", DELIMITER))
            .output()?;
        if !output.status.success() {
            return Err("Failed to get git shortlog".into());
        }
        let stdout = String::from_utf8(output.stdout)?;
        let mut authors = Vec::new();
        for line in stdout.lines() {
            let mut parts = line.split(DELIMITER);
            if parts.clone().count() != 2 {
                continue;
            }
            let name = parts.next().unwrap().trim().to_string();
            let email: Option<String> = match parts.next().unwrap() {
                "" => None,
                email => Some(email.to_string()),
            };
            authors.push(Author {
                name: Some(name),
                username: None,
                email,
            });
        }
        Ok(authors)
    }
}

fn split_raw_git_log_output(s: &str) -> (Vec<String>, String) {
    let mut udiff_strings = Vec::with_capacity(s.matches("diff --git").count());
    let mut current_udiff = String::with_capacity(s.len());
    let mut in_udiff = false;
    let mut remaining_log = String::with_capacity(s.len());
    for line in s.lines() {
        if line.starts_with("diff --git") {
            if in_udiff && !current_udiff.is_empty() {
                udiff_strings.push(std::mem::take(&mut current_udiff));
                current_udiff.clear();
            }
            in_udiff = true;
        }
        if in_udiff {
            current_udiff.push_str(line);
            current_udiff.push('\n');
        } else {
            remaining_log.push_str(line);
            remaining_log.push('\n');
        }
    }
    if in_udiff && !current_udiff.is_empty() {
        udiff_strings.push(current_udiff);
    }
    (udiff_strings, remaining_log)
}
