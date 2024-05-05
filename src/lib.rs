pub mod cli;
pub mod data;

use csv::Writer;
use gdal::errors::GdalError;
use gdal::vector::LayerAccess;
use gdal::{vector::Layer, Dataset, DatasetOptions, GdalOpenFlags};
use polars::frame::DataFrame;
use polars::prelude::*;
use serde::{Deserialize, Serialize};
use std::error::Error;
use std::fmt::Display;
use std::fs::File;
use std::io::{self, Read, Write};
use std::path::{Path, PathBuf};

// LayerCount
/// Hosts the layer name and its corresponding feature count.
#[derive(Debug, Serialize, Deserialize)]
struct LayerCount {
    layer: String,
    count: u64,
}

// LayerCount traits

impl Display for LayerCount {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{{ layer: {}, count: {} }}", self.layer, self.count)
    }
}

impl From<&Layer<'_>> for LayerCount {
    fn from(l: &Layer) -> Self {
        LayerCount {
            layer: l.name(),
            count: l.feature_count(),
        }
    }
}

/// Wrapper around a Vec<LayerCount>. A Dataset is composed of layers. Counting Dataset features imply counting features for each Layer.
#[derive(Serialize, Deserialize)]
pub struct DatasetCount(Vec<LayerCount>);

// Functions

impl DatasetCount {
    /// Create an empty DatasetCount for initialization.
    fn new() -> DatasetCount {
        DatasetCount(Vec::new())
    }

    /// Build a DatasetCount from a csv formatted input implementing the Read trait.
    pub fn from_csv<R: Read>(input: R) -> Result<Self, CountError> {
        let mut rdr = csv::Reader::from_reader(input);
        let mut dc: Vec<LayerCount> = vec![];
        for r in rdr.deserialize() {
            dc.push(r.map_err(|kind| CountError {
                kind: ErrorKind::Csv(kind),
            })?);
        }
        Ok(DatasetCount(dc))
    }

    /// Serialize a DatasetCount into a writer implementing the Write trait.
    pub fn to_csv<W: Write>(&self, output: W) -> Result<csv::Writer<W>, CountError> {
        let mut wtr = Writer::from_writer(output);
        for r in self.0.iter() {
            wtr.serialize(r).map_err(|kind| CountError {
                kind: ErrorKind::Csv(kind),
            })?;
        }
        Ok(wtr)
    }

    /// Compare to DatasetCount by joining them and calculating their count differences.
    pub fn compare(self, other: DatasetCount) -> Result<(), CountError> {
        let (df1, df2): (DataFrame, DataFrame) = (self.try_into()?, other.try_into()?);
        let df = &df1
            .outer_join(&df2, ["layer"], ["layer"])
            .map_err(|kind| CountError {
                kind: ErrorKind::Polars(kind),
            })?;

        let mut result = df
            .clone()
            .lazy()
            .select([
                all(),
                (col("count") - col("count_right")).alias("difference"),
            ])
            .collect()
            .map_err(|kind| CountError {
                kind: ErrorKind::Polars(kind),
            })?;

        CsvWriter::new(io::stdout())
            .include_header(true)
            .with_separator(b',')
            .finish(&mut result)
            .map_err(|kind| CountError {
                kind: ErrorKind::Polars(kind),
            })?;

        Ok(())
    }
}

// Traits implementations

impl Display for DatasetCount {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.0.iter().try_for_each(|x| write!(f, "{x}"))
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

impl TryFrom<&File> for DatasetCount {
    type Error = CountError;
    fn try_from(f: &File) -> Result<Self, Self::Error> {
        match DatasetCount::from_csv(f) {
            Ok(dc) => Ok(dc),
            Err(e) => Err(e),
        }
    }
}

impl TryFrom<PathBuf> for DatasetCount {
    type Error = CountError;
    fn try_from(p: PathBuf) -> Result<Self, Self::Error> {
        let drivers = data::drivers().map_err(|kind| CountError {
            kind: ErrorKind::Gdal(kind),
        })?;
        let d: Vec<&str> = drivers.iter().map(|s| &**s).collect();

        let opt = DatasetOptions {
            open_flags: GdalOpenFlags::GDAL_OF_VECTOR,
            allowed_drivers: Some(&d[..]),
            open_options: None,
            sibling_files: None,
        };

        match p.metadata() {
            // If metadata acessible and is file
            Ok(m) if m.is_file() => match p.extension() {
                // If zip
                Some(v) if v == "zip" => Ok(DatasetCount::from(
                    &Dataset::open_ex(Path::new("/vsizip/").join(&p), opt).map_err(|kind| {
                        CountError {
                            kind: ErrorKind::Gdal(kind),
                        }
                    })?,
                )),
                // If any other extension
                Some(_) => match DatasetCount::try_from(&File::open(&p).unwrap()) {
                    Ok(dc) => Ok(dc),
                    Err(_) => Ok(DatasetCount::from(&Dataset::open_ex(&p, opt).map_err(
                        |kind| CountError {
                            kind: ErrorKind::Gdal(kind),
                        },
                    )?)),
                },
                // If no extension
                None => Ok(DatasetCount::from(&Dataset::open_ex(&p, opt).map_err(
                    |kind| CountError {
                        kind: ErrorKind::Gdal(kind),
                    },
                )?)),
            },
            // If metadata accessible and anything else than file
            Ok(_) => Ok(DatasetCount::from(&Dataset::open_ex(&p, opt).map_err(
                |kind| CountError {
                    kind: ErrorKind::Gdal(kind),
                },
            )?)),
            // If no Path metadata (e.g. database)
            Err(_) => Ok(DatasetCount::from(&Dataset::open_ex(&p, opt).map_err(
                |kind| CountError {
                    kind: ErrorKind::Gdal(kind),
                },
            )?)),
        }
    }
}

// Errors

#[derive(Debug)]
pub struct CountError {
    pub kind: ErrorKind,
}

impl Display for CountError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "error while couting stuff")
    }
}

impl Error for CountError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match &self.kind {
            ErrorKind::Csv(e) => Some(e),
            ErrorKind::Gdal(e) => Some(e),
            ErrorKind::Polars(e) => Some(e),
            ErrorKind::File(e) => Some(e),
        }
    }
}

#[derive(Debug)]
pub enum ErrorKind {
    Csv(csv::Error),
    Gdal(GdalError),
    Polars(PolarsError),
    File(io::Error),
}

// Macro for parsing DatasetCount into a Polars DataFrame (from https://stackoverflow.com/questions/73167416/creating-polars-dataframe-from-vecstruct?rq=3)
macro_rules! struct_to_dataframe {
    ($input:expr, [$($field:ident),+]) => {
        {
            let len = $input.len().to_owned();

            // Extract the field values into separate vectors
            $(let mut $field = Vec::with_capacity(len);)*

            for e in $input.into_iter() {
                $($field.push(e.$field);)*
            }
            df! {
                $(stringify!($field) => $field,)*
            }
        }
    };
}

impl TryInto<DataFrame> for DatasetCount {
    type Error = CountError;
    fn try_into(self) -> Result<DataFrame, Self::Error> {
        struct_to_dataframe!(self.0, [layer, count]).map_err(|kind| CountError {
            kind: ErrorKind::Polars(kind),
        })
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
