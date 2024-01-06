use clap::Parser;
use gitm::bm25::BM25Retriever;
use gitm::git;
use gitm::github;
use gitm::llm::ChatModel;
use gitm::llm::ChatModelKey::Gpt4;
use gitm::utils::{does_command_exist, does_valid_git_dir_exist};

#[derive(Parser, Debug)]
struct Args {
    #[arg(short, long)]
    query: String,

    #[arg(long)]
    api_key: String,

    #[arg(long, default_value = "false")]
    issues_only: bool,

    #[arg(long, default_value = "false")]
    issues_too: bool,
}

fn get_args() -> Result<Args, Box<dyn std::error::Error>> {
    let args = Args::parse();
    if args.issues_only && args.issues_too {
        return Err("Cannot specify both --issues-only and --issues-too".into());
    }
    Ok(args)
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = get_args().unwrap();
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
    let mode = if args.issues_only {
        gitm::search_agent::SearchMode::Issues
    } else if args.issues_too {
        gitm::search_agent::SearchMode::CommitsAndIssues
    } else {
        gitm::search_agent::SearchMode::Commits
    };
    let results = search_agent.search(args.query, 10, mode).await.unwrap();
    for result in results.0 {
        println!("commit: {}", result.patch_set);
    }
    for result in results.1 {
        println!("issue: {}", result.title);
    }
    Ok(())
}
