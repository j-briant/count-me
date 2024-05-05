use clap::Parser;

#[derive(Parser)]
pub struct Cli {
    #[arg(num_args(1..2))]
    pub path: Vec<String>,
}

impl Cli {
    pub fn arg_parse() -> Cli {
        Cli::parse()
    }
}
