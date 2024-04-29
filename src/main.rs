use count_me::{cli::Cli, data};
use csv::Writer;
use serde::{Deserialize, Serialize};
use std::io::{self, Read, Write};

use gdal::{
    errors::GdalError,
    vector::{Layer, LayerAccess},
    Dataset, GdalOpenFlags,
};

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

fn main() -> Result<(), GdalError> {
    let drivers = data::drivers().unwrap();
    let d: Vec<&str> = drivers.iter().map(|s| &**s).collect();

    let opt = gdal::DatasetOptions {
        open_flags: GdalOpenFlags::GDAL_OF_VECTOR,
        allowed_drivers: Some(&d[..]),
        open_options: None,
        sibling_files: None,
    };

    // Initialize CLI
    let cli = Cli::arg_parse();

    // If 1 argument we count.
    if cli.path.len() == 1 {
        let mut p = String::from(&cli.path[0]);
        // If zipfile prepend /vsizip/ to work with virtual file system.
        if p.ends_with(".zip") {
            p.insert_str(0, "/vsizip/");
        }
        let data = Dataset::open_ex(&p, opt)?;
        let dc: DatasetCount = DatasetCount::from(&data);
        dc.to_csv(io::stdout()).unwrap();
        Ok(())
    // If 2 arguments we compare.
    } else if cli.path.len() == 2 {
        todo!();
    // If anything else we talk sh*t.
    } else {
        Err(GdalError::BadArgument("Missing input".into()))
    }
}
