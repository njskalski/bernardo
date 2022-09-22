use clap::Parser;

use bernardo;

mod reader_main_widget;

#[derive(Parser, Debug)]
#[clap(about)]
struct Args {
    #[clap(short, long, value_parser)]
    file: String,
}

fn main() {
    let args = Args::parse();
}