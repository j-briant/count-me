use countme::CountError;
pub use countme::{cli::Cli, CountDifference, CountDifferenceVec, DatasetCount};
use std::error::Error;
use std::io;
use std::path::PathBuf;

fn print_error(e: CountError) {
    println!("Error: {e}");
    let mut outer: &dyn Error = &e;
    while let Some(source) = outer.source() {
        println!("Cause: {source}");
        outer = source;
    }
}

fn main() {
    // Initialize CLI
    let cli = Cli::arg_parse();

    // If 1 argument we count.
    if cli.path.len() == 1 {
        let p = String::from(&cli.path[0]);
        let path = PathBuf::from(&p);

        match DatasetCount::try_from(path) {
            Ok(dc) => match dc.to_csv(io::stdout()) {
                Ok(_) => (),
                Err(e) => print_error(e),
            },
            Err(e) => print_error(e),
        }
    // If 2 arguments we compare.
    } else if cli.path.len() == 2 {
        let (p1, p2) = (PathBuf::from(&cli.path[0]), PathBuf::from(&cli.path[1]));

        match (DatasetCount::try_from(p1), DatasetCount::try_from(p2)) {
            (Ok(dc1), Ok(dc2)) => {
                match CountDifferenceVec::from(dc1.outer_join(&dc2)).to_csv(io::stdout()) {
                    Ok(_) => (),
                    Err(e) => print_error(e),
                }
            }
            (Ok(_), Err(e)) => print_error(e),
            (Err(e), Ok(_)) => print_error(e),
            (Err(e), Err(f)) => {
                print_error(e);
                print_error(f);
            }
        }
    }
}
