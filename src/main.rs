pub use countme::{cli::Cli, data, DatasetCount};
use std::error::Error;
use std::io;
use std::path::PathBuf;

fn main() -> Result<(), Box<dyn Error>> {
    // Initialize CLI
    let cli = Cli::arg_parse();

    // If 1 argument we count.
    if cli.path.len() == 1 {
        let p = String::from(&cli.path[0]);
        let path = PathBuf::from(&p);

        let dc = DatasetCount::try_from(path)?;
        dc.to_csv(io::stdout())?;

        Ok(())
    // If 2 arguments we compare.
    } else if cli.path.len() == 2 {
        let (p1, p2) = (PathBuf::from(&cli.path[0]), PathBuf::from(&cli.path[1]));
        let data1 = DatasetCount::try_from(p1)?;
        let data2 = DatasetCount::try_from(p2)?;

        data1.compare(data2)?;
        Ok(())

    // If anything else we talk sh*t.
    } else {
        Ok(())
    }
}
