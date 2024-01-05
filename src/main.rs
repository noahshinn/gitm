use clap::Parser;
use gitm::bm25::BM25Ranker;
use gitm::git::Client;
use gitm::rankers::Ranker;
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
    let ranker = BM25Ranker::<String>::new().k1(1.2).b(0.75).build();
    let ranked_results = ranker.rank(
        String::from("some search about germany"),
        vec![
            "Germany was founded in 1871",
            "Apples fall downwards",
            "What happened to Alan Turing?",
            "Google is searching for answers",
            "When was the first computer invented?",
        ]
        .iter()
        .map(|s| s.to_string())
        .collect(),
        None,
    )?;
    for result in ranked_results {
        println!("{}", result.doc);
    }
    Ok(())
}
