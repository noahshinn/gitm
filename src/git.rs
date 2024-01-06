use chrono::{DateTime, Utc};
use std::fmt::{Display, Formatter};
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
}

impl Display for Commit {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}\n\n{}", self.title, self.body)
    }
}

#[derive(Debug, Clone)]
pub struct Author {
    pub name: Option<String>,
    pub username: Option<String>,
    pub email: Option<String>,
}

const DELIMITER: &str = "|||";

const GIT_LOG_PARSE_FIELDS: [&str; 6] = ["%an", "%ae", "%aD", "%s", "%b", "%H"];

impl Client {
    pub fn new() -> Client {
        Client
    }

    pub fn get_all_commits(&self) -> Result<Vec<Commit>, Box<dyn std::error::Error>> {
        let format = format!("--pretty=format:{}", GIT_LOG_PARSE_FIELDS.join(DELIMITER));
        let output = Command::new("git")
            .arg("log")
            .arg("-uz")
            .arg(format)
            .output()?;
        if !output.status.success() {
            return Err("Failed to get git log".into());
        }
        let stdout = String::from_utf8(output.stdout)?;
        let mut commits = Vec::new();
        for commit_data in stdout.split("\0") {
            let (udiff_strings, remaining_log) = split_raw_git_log_output(commit_data.to_string());
            if udiff_strings.is_empty() || remaining_log.is_empty() {
                continue;
            }
            let mut patch_set = PatchSet::new();
            patch_set.parse(udiff_strings.join("\n")).unwrap();
            let mut parts = remaining_log.split(DELIMITER);
            if parts.clone().count() != GIT_LOG_PARSE_FIELDS.len() {
                continue;
            }
            let author_name = parts.next().unwrap().to_string();
            let author_email: Option<String> = match parts.next().unwrap() {
                "" => None,
                email => Some(email.to_string()),
            };
            let date_raw = parts.next().unwrap().to_string();
            let title = parts.next().unwrap().to_string();
            let body = parts.next().unwrap().to_string();
            let sha = parts.next().unwrap().to_string();
            let date = DateTime::parse_from_rfc2822(&date_raw)?.with_timezone(&Utc);
            commits.push(Commit {
                author: Author {
                    name: Some(author_name),
                    username: None,
                    email: author_email,
                },
                date,
                title,
                body,
                sha,
                patch_set,
            })
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

fn split_raw_git_log_output(s: String) -> (Vec<String>, String) {
    let mut udiff_strings = Vec::new();
    let mut current_udiff = String::new();
    let mut in_udiff = false;
    let mut remaining_log = String::new();
    for line in s.lines() {
        if line.starts_with("diff --git") {
            if in_udiff && !current_udiff.is_empty() {
                udiff_strings.push(current_udiff.clone());
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
