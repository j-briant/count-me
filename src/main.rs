use countme::CountError;
pub use countme::{CountDifference, CountDifferenceVec, DatasetCount};
use std::env;
use std::error::Error;
use std::io;
use std::path::PathBuf;

const HELP_MESSAGE: &str = "Count features in GDAL compatible vector data
Usage: countme [SRC]...

Arguments:
    [SRC]...  Path to 1 or 2 data sources or counts (see GDAL drivers documentation)

Options:
    -h, --help  Print help (see more with '--help')";

fn print_error(e: CountError) {
    println!("Error: {e}");
    let mut outer: &dyn Error = &e;
    while let Some(source) = outer.source() {
        println!("Cause: {source}");
        outer = source;
    }
}

fn main() {
    // Get cli arguments
    let args: Vec<String> = env::args().collect();

    // If 1 argument we count.
    if args.len() == 2 {
        if args[1] == "-h" || args[1] == "--help" {
            println!("{HELP_MESSAGE}");
        } else {
            let p = String::from(&args[1]);
            let path = PathBuf::from(&p);

            match DatasetCount::try_from(path) {
                Ok(dc) => match dc.to_csv(io::stdout()) {
                    Ok(_) => (),
                    Err(e) => print_error(e),
                },
                Err(e) => print_error(e),
            }
        }
    // If 2 arguments we compare.
    } else if args.len() == 3 {
        let (p1, p2) = (PathBuf::from(&args[1]), PathBuf::from(&args[2]));

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
    } else {
        println!("Error: 1 or 2 arguments must be given, see help below:\n{HELP_MESSAGE}");
    }
}
