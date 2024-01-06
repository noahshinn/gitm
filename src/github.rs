use crate::git::Author;
use chrono::{DateTime, Utc};
use serde::Deserialize;
use std::fmt::Display;
use std::process::Command;

#[derive(Debug, Clone)]
pub struct Issue {
    pub title: String,
    pub body: String,
    pub author: Author,
    pub created_at: DateTime<Utc>,
    pub number: u64,
}

impl Display for Issue {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut output = String::new();
        output.push_str(&format!("{}\n", self.title));
        output.push_str(&format!("{}\n", self.body));
        output.push_str(&format!("{}\n", self.author.name.clone().unwrap()));
        output.push_str(&format!("{}\n", self.created_at));
        output.push_str(&format!("{}\n", self.number));
        write!(f, "{}", output)
    }
}

pub struct Client;

impl Client {
    pub fn new() -> Client {
        Client
    }

    pub fn get_all_issues(&self) -> Result<Vec<Issue>, Box<dyn std::error::Error>> {
        #[derive(Deserialize)]
        struct AuthorJson {
            login: String,
        }
        #[derive(Deserialize)]
        struct IssueJson {
            author: AuthorJson,
            number: u64,
            #[serde(alias = "createdAt")]
            created_at: String,
            title: String,
            body: String,
        }
        let output = Command::new("gh")
            .arg("issue")
            .arg("list")
            .arg("--json")
            .arg("author,number,title,body,createdAt")
            .output()?;
        if !output.status.success() {
            return Err("Failed to get issues".into());
        }
        let stdout = String::from_utf8(output.stdout)?;
        let issues_json = serde_json::from_str::<Vec<IssueJson>>(&stdout)?;
        let mut issues = Vec::new();
        for issue in &issues_json {
            let datetime_str = issue.created_at.clone();
            let created_at: DateTime<Utc> = datetime_str.parse().expect("Invalid datetime format");
            issues.push(Issue {
                title: issue.title.clone(),
                body: issue.body.clone(),
                created_at,
                number: issue.number,
                author: Author {
                    name: Some(issue.author.login.clone()),
                    username: None,
                    email: None,
                },
            });
        }
        Ok(issues)
    }
}
