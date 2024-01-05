use clap::Parser;
use gitm::git::Client;
use gitm::utils::does_valid_git_dir_exist;

#[derive(Parser, Debug)]
struct Args {
    #[arg(short, long)]
    query: String,

    #[arg(long)]
    api_key: String,
}

fn get_args() -> Result<Args, Box<dyn std::error::Error>> {
    let args = Args::parse();
    Ok(args)
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // let args = get_args().unwrap();
    let client = Client::new();
    if !does_valid_git_dir_exist()? {
        println!("Not a valid git directory");
        return Ok(());
    }
    let response = client.get_all_authors();
    match response {
        Ok(commits) => {
            for commit in commits {
                println!("{:?}", commit);
            }
        }
        Err(e) => {
            println!("{}", e);
        }
    }
    Ok(())
}
