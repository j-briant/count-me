pub use countme::{cli::Cli, CountDifference, CountDifferenceVec, DatasetCount};
use std::io;
use std::path::PathBuf;

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
                Err(e) => {
                    println!("{e}");
                }
            },
            Err(e) => {
                println!("{e}");
            }
        }
    // If 2 arguments we compare.
    } else if cli.path.len() == 2 {
        let (p1, p2) = (PathBuf::from(&cli.path[0]), PathBuf::from(&cli.path[1]));

        match (DatasetCount::try_from(p1), DatasetCount::try_from(p2)) {
            (Ok(dc1), Ok(dc2)) => {
                match CountDifferenceVec::from(dc1.outer_join(&dc2)).to_csv(io::stdout()) {
                    Ok(_) => (),
                    Err(e) => {
                        println!("{e}");
                    }
                }
            }
            (Ok(_), Err(e)) => println!("{e}"),
            (Err(e), Ok(_)) => println!("{e}"),
            (Err(e), Err(f)) => println!("{e}, {f}"),
        }
    }
}
