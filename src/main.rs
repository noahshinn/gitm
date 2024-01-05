use clap::Parser;
use gitm::bm25::BM25Retriever;
use gitm::retrievers::Retriever;
use gitm::store::Store;
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
    if !does_valid_git_dir_exist()? {
        println!("Not a valid git directory");
        return Ok(());
    }
    // let client = Client::new();
    let corpus = vec![
        "Germany was founded in 1871",
        "Apples fall downwards",
        "What happened to Alan Turing?",
        "Google is searching for answers",
        "When was the first computer invented?",
    ];
    let store = Store::from(corpus);
    let retriever = BM25Retriever::new();
    let retrieved_results =
        retriever.retrieve(String::from("some search about germany"), store, 2)?;
    for result in retrieved_results {
        println!("{}", result);
    }
    Ok(())
}
