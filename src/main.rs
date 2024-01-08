use clap::Parser;
use gitm::llm::ChatModel;
use gitm::llm::ChatModelKey::Gpt4;
use gitm::search_agent::SearchConfigBuilder;
use gitm::utils::{does_command_exist, does_valid_git_dir_exist};
use std::env;

#[derive(Parser, Debug)]
#[clap(
    name = "gitm",
    about = "A command line tool for searching through GitHub issues, commit messages, and code patches."
)]
struct Args {
    query: String,

    #[arg(long, default_value = "", help = "OpenAI API key")]
    api_key: String,

    #[arg(
        long,
        default_value = "false",
        help = "If set, only GitHub issues will be searched"
    )]
    issues_only: bool,

    #[arg(
        long,
        default_value = "false",
        help = "If set, GitHub issues will be searched"
    )]
    issues_too: bool,

    #[arg(
        long,
        default_value = "false",
        help = "If set, code patches will be included in the search"
    )]
    include_code_patches: bool,

    #[arg(
        long,
        default_value = "false",
        help = "If set, the natural language classifications on the query will be disabled"
    )]
    disable_classifications: bool,

    #[arg(
        long,
        default_value = "false",
        help = "If set, the search will be performed on all (issues or commits); by default, the last 2 months of data are searched."
    )]
    all: bool,
}

fn get_args() -> Result<Args, Box<dyn std::error::Error>> {
    let args = Args::parse();
    if args.api_key == "" {
        match env::var("OPENAI_API_KEY") {
            Ok(api_key) => {
                return Ok(Args { api_key, ..args });
            }
            Err(_) => {
                if args.api_key == "" {
                    return Err("No API key provided. Set OPENAI_API_KEY as an env var or pass it with the --api-key flag".into());
                }
            }
        }
    } else if args.issues_only && args.issues_too {
        return Err("Cannot specify both --issues-only and --issues-too".into());
    }
    Ok(args)
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = get_args();
    if let Err(e) = args {
        println!("\n{}", e);
        return Ok(());
    }
    let args = args.unwrap();
    if !does_command_exist("git")? {
        println!("Git is not installed");
        return Ok(());
    } else if !does_command_exist("gh")? {
        println!("Github CLI is not installed");
        return Ok(());
    } else if !does_valid_git_dir_exist()? {
        println!("Not a valid git directory");
        return Ok(());
    }

    let model = ChatModel::new(args.api_key, Gpt4);
    let search_agent = gitm::search_agent::SearchAgent::new(model);
    let search_config = SearchConfigBuilder::new(args.query)
        .max_num_results(10)
        .include_commits(!args.issues_only)
        .include_issues(args.issues_too)
        .include_code_patches(args.include_code_patches)
        .disable_classifications(args.disable_classifications)
        .build();
    let (commits, issues) = search_agent.search(search_config).await.unwrap();
    for commit in commits {
        println!("{}", commit.mock_git_log_fmt());
    }
    for issue in issues {
        println!("{}", issue.mock_gh_issue_fmt());
    }
    Ok(())
}
