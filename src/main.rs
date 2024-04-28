use clap::{Parser, Subcommand};
use csv::{Error, Writer};
use serde::{Deserialize, Serialize};
use std::{
    fs::File,
    io::{self, Read, Write},
    path::Path,
};

use gdal::{vector::Layer, vector::LayerAccess, Dataset};

#[derive(Debug, Serialize, Deserialize)]
struct LayerCount {
    layer: String,
    count: u64,
}

impl From<&Layer<'_>> for LayerCount {
    fn from(l: &Layer) -> Self {
        LayerCount {
            layer: l.name(),
            count: l.feature_count(),
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
struct DatasetCount(Vec<LayerCount>);

impl DatasetCount {
    fn new() -> DatasetCount {
        DatasetCount(Vec::new())
    }

    fn from_csv<R: Read>(input: R) -> Result<Self, csv::Error> {
        let mut rdr = csv::Reader::from_reader(input);
        let mut dc: Vec<LayerCount> = vec![];
        for r in rdr.deserialize() {
            dc.push(r?);
        }
        Ok(DatasetCount(dc))
    }

    fn to_csv<W: Write>(&self, output: W) -> Result<(), csv::Error> {
        let mut wtr = Writer::from_writer(output);
        for r in self.0.iter() {
            wtr.serialize(r)?;
        }
        Ok(())
    }

    fn compare(&self, dc: DatasetCount) {
        todo!();
    }
}

impl From<&Dataset> for DatasetCount {
    fn from(d: &Dataset) -> Self {
        d.layers().collect()
    }
}

impl From<Vec<LayerCount>> for DatasetCount {
    fn from(vlc: Vec<LayerCount>) -> Self {
        DatasetCount(vlc)
    }
}

impl FromIterator<LayerCount> for DatasetCount {
    fn from_iter<T: IntoIterator<Item = LayerCount>>(iter: T) -> Self {
        let mut dc = DatasetCount::new();
        for i in iter {
            dc.0.push(i);
        }
        dc
    }
}

impl<'a> FromIterator<Layer<'a>> for DatasetCount {
    fn from_iter<T: IntoIterator<Item = Layer<'a>>>(iter: T) -> Self {
        let mut dc = DatasetCount::new();
        for i in iter {
            dc.0.push(LayerCount::from(&i));
        }
        dc
    }
}

#[derive(Parser)]
struct Cli {
    #[command(subcommand)]
    command: Option<Command>,
}

#[derive(Subcommand)]
enum Command {
    Count {
        #[arg()]
        dataset: String,
    },
    Compare {
        #[arg(num_args(2))]
        counted: Option<Vec<String>>,
    },
}

fn main() {
    /*
    let cli = Cli::parse();

    match &cli.command {
        Some(Command::Count { dataset }) => {
            let data = Dataset::open(dataset).unwrap();
            let datacount2: DatasetCount = DatasetCount::from(&data);
            let _ = datacount2.to_csv(io::stdout());
        }
        Some(Command::Compare { counted }) => {
            todo!();
        }
        None => {}
    }
    */

    let mut file = File::open("foo.txt").unwrap();
    let mut content = String::new();
    file.read_to_string(&mut content).unwrap();
    let dc = DatasetCount::from_csv(content.as_bytes());

    println!("{content}");
    println!("{:?}", dc.unwrap());
}
