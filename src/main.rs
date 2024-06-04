use clap::Parser;
mod odos;


#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    #[arg(long)]
    rpc: String,

    #[arg(long)]
    private_key: String,

    #[arg(long, default_value_t = String::from("0x0000000000000000000000000000000000000000"))]
    token_in: String,

    #[arg(long, default_value_t = String::from("0x0000000000000000000000000000000000000000"))]
    token_out: String,

    #[arg(long)]
    amount: u128,
}


#[tokio::main]
async fn main() {
    let args: Args = Args::parse();

    let token_in = args.token_in;
    let token_out = args.token_out;

    if &token_in == &token_out {
        panic!("token in cannot be same as token out")
    };

    let mut path: Vec<String> = Vec::new();
    path.push(token_in);
    path.push(token_out);

    let result = odos::odos_onchain::swap(
        args.rpc, 
        args.private_key, 
        args.amount,
        path
    ).await;

    println!("Result: {:?}", result)
}
