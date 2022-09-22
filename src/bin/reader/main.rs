use clap::Parser;

#[derive(Parser, Debug)]
#[clap(about)]
struct Args {
    #[clap(short, long, value_parser)]
    file: String,
}

fn main() {
    let args = Args::parse();
}