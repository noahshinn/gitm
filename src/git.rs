use chrono::{DateTime, Utc};
use std::fmt::{Display, Formatter};
use std::process::Command;

pub struct Client;

#[derive(Debug, Clone)]
pub struct Commit {
    pub author: Author,
    pub date: DateTime<Utc>,
    pub message: CommitMessage,
    pub sha: String,
}

impl Display for Commit {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}\n\n{}", self.message.title, self.message.body)
    }
}

#[derive(Debug, Clone)]
pub struct CommitMessage {
    pub title: String,
    pub body: String,
}

#[derive(Debug, Clone)]
pub struct Author {
    pub name: String,
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
        let output = Command::new("git").arg("log").arg(format).output()?;
        if !output.status.success() {
            return Err("Failed to get git log".into());
        }
        let stdout = String::from_utf8(output.stdout)?;
        let mut commits = Vec::new();
        for line in stdout.lines() {
            let mut parts = line.split(DELIMITER);
            if parts.clone().count() != GIT_LOG_PARSE_FIELDS.len() {
                continue;
            }
            let author_name = parts.next().unwrap().to_string();
            let author_email: Option<String> = match parts.next().unwrap() {
                "" => None,
                email => Some(email.to_string()),
            };
            let date_raw = parts.next().unwrap().to_string();
            let subject = parts.next().unwrap().to_string();
            let body = parts.next().unwrap().to_string();
            let sha = parts.next().unwrap().to_string();
            let date = DateTime::parse_from_rfc2822(&date_raw)?.with_timezone(&Utc);
            commits.push(Commit {
                author: Author {
                    name: author_name,
                    email: author_email,
                },
                date,
                message: CommitMessage {
                    title: subject,
                    body,
                },
                sha,
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
            authors.push(Author { name, email });
        }
        Ok(authors)
    }
}
