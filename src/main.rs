use clap::Parser;
use gitm::bm25::BM25Retriever;
use gitm::git::Client;
use gitm::llm::ChatModel;
use gitm::llm::ChatModelKey::Gpt4;
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
    let args = get_args().unwrap();
    if !does_valid_git_dir_exist()? {
        println!("Not a valid git directory");
        return Ok(());
    }
    let client = Client::new();
    let retriever = BM25Retriever::new();
    let model = ChatModel::new(args.api_key, Gpt4);
    let search_agent = gitm::search_agent::SearchAgent::new(client, &retriever, model);
    let results = search_agent.search(args.query, 10).await;
    for result in results {
        println!("{}", result);
    }
    Ok(())
}
