use clap::Parser;

#[derive(Parser)]
/// Count features in GDAL compatible vector data.
///
/// If 1 argument is passed, count features and print a csv-formatted output.
/// If 2 arguments are passed, count features and compute the by-layer difference, print a csv-formatted output.
pub struct Cli {
    /// Path to data sources (see GDAL drivers documentation)
    #[arg(num_args(1..2))]
    pub src: Vec<String>,
}

impl Cli {
    pub fn arg_parse() -> Cli {
        Cli::parse()
    }
}
