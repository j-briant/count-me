use clap::Parser;

#[derive(Parser)]
pub struct Cli {
    #[arg(num_args(1..2))]
    pub path: Vec<String>,
    /*     #[command(subcommand)]
    pub command: Option<Command>, */
}

impl Cli {
    pub fn arg_parse() -> Cli {
        Cli::parse()
    }
}

/* #[derive(Subcommand)]
pub enum Command {
    Count {
        #[arg()]
        dataset: String,
    },
    Compare {
        #[arg(num_args(2))]
        counted: Option<Vec<String>>,
    },
} */
