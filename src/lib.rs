pub mod cli;

use csv::Writer;
use gdal::errors::GdalError;
use gdal::vector::LayerAccess;
use gdal::{vector::Layer, Dataset, DatasetOptions, DriverManager, GdalOpenFlags};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::error::Error;
use std::fmt::Display;
use std::fs::File;
use std::io::{self, Read, Write};
use std::path::{Path, PathBuf};
use std::str::FromStr;
use std::vec;

lazy_static::lazy_static! {
    static ref DRIVERS: Vec<String> = {
        DriverManager::register_all();
        let count = DriverManager::count();
        let mut list: Vec<String> = vec![];
        for i in 0..count {
            if let Ok(d) = DriverManager::get_driver(i) {
            list.push(d.short_name())
            }
        }
        list
    };
}

lazy_static::lazy_static! {
    static ref DRIVERS_STR: Vec<&'static str> = {
        let v: Vec<&str> = DRIVERS.iter().map(|s| s.as_str()).collect();
        v
    };
}

fn get_dataset_options() -> DatasetOptions<'static> {
    DatasetOptions {
        open_flags: GdalOpenFlags::GDAL_OF_VECTOR,
        allowed_drivers: Some(&DRIVERS_STR),
        open_options: None,
        sibling_files: None,
    }
}

// LayerCount
/// Hosts the layer name and its corresponding feature count.
#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
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

impl FromStr for LayerCount {
    type Err = String;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.split_once(',') {
            Some((layer, count)) => Ok(LayerCount {
                layer: layer.trim().into(),
                count: count.trim().parse().expect("error around here"),
            }),
            None => Err(format!("error while parsing {s}")),
        }
    }
}

/// Wrapper around a `Vec<LayerCount>`. A Dataset is composed of layers. Counting Dataset features imply counting features for each Layer.
#[derive(Serialize, Deserialize, PartialEq, Debug)]
pub struct DatasetCount(Vec<LayerCount>);

// Functions
impl DatasetCount {
    /// Create an empty `DatasetCount` for initialization.
    fn new() -> DatasetCount {
        DatasetCount(Vec::new())
    }

    /// Build a `DatasetCount` from a csv formatted input implementing the Read trait.
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

    /// Serialize a `DatasetCount` into a writer implementing the Write trait.
    pub fn to_csv<W: Write>(&self, output: W) -> Result<csv::Writer<W>, CountError> {
        let mut wtr = Writer::from_writer(output);
        for r in self.0.iter() {
            wtr.serialize(r).map_err(|kind| CountError {
                kind: ErrorKind::Csv(kind),
            })?;
        }
        Ok(wtr)
    }

    /// Compare to `DatasetCount` by joining them and calculating their count differences.
    pub fn outer_join(&self, other: &DatasetCount) -> Vec<CountDifference> {
        let mut differences = Vec::new();
        let mut layer_map = HashMap::new();

        // Insert layers from self
        for layer_count in &self.0 {
            layer_map.insert(&layer_count.layer, (Some(layer_count.count), None));
        }

        // Insert layers from other
        for layer_count in &other.0 {
            layer_map
                .entry(&layer_count.layer)
                .and_modify(|(_, right_count)| *right_count = Some(layer_count.count))
                .or_insert((Some(0), Some(layer_count.count)));
        }

        // Iterate through layer_map to create CountDifference
        for (layer, (left_count, right_count)) in layer_map {
            let difference = match (left_count, right_count) {
                (Some(left), Some(right)) => Some(left as i128 - right as i128),
                (Some(left), None) => Some(left as i128),
                (None, Some(right)) => Some(-(right as i128)),
                (None, None) => None,
            };

            differences.push(CountDifference {
                layer: layer.to_string(), // Convert layer to owned String
                left_count,
                right_count,
                difference,
            });
        }

        // Sort differences by layer name
        differences.sort_by(|a, b| a.layer.cmp(&b.layer));

        differences
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

impl FromStr for DatasetCount {
    type Err = CountError;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut iter = s.lines();
        if iter.next().is_some() {
            Ok(DatasetCount(
                iter.map(|x| LayerCount::from_str(x).unwrap()).collect(),
            ))
        } else {
            panic!("omg2")
        }
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
        let driver = get_dataset_options();

        match p.metadata() {
            // If metadata acessible and is file
            Ok(m) if m.is_file() => match p.extension() {
                // If zip
                Some(v) if v == "zip" => Ok(DatasetCount::from(
                    &Dataset::open_ex(Path::new("/vsizip/").join(&p), driver).map_err(|kind| {
                        CountError {
                            kind: ErrorKind::Gdal(kind),
                        }
                    })?,
                )),
                // If any other extension
                Some(_) => match DatasetCount::try_from(&File::open(&p).unwrap()) {
                    Ok(dc) => Ok(dc),
                    Err(_) => Ok(DatasetCount::from(&Dataset::open_ex(&p, driver).map_err(
                        |kind| CountError {
                            kind: ErrorKind::Gdal(kind),
                        },
                    )?)),
                },
                // If no extension
                None => Ok(DatasetCount::from(&Dataset::open_ex(&p, driver).map_err(
                    |kind| CountError {
                        kind: ErrorKind::Gdal(kind),
                    },
                )?)),
            },
            // If metadata accessible and anything else than file
            Ok(_) => Ok(DatasetCount::from(&Dataset::open_ex(&p, driver).map_err(
                |kind| CountError {
                    kind: ErrorKind::Gdal(kind),
                },
            )?)),
            // If no Path metadata (e.g. database)
            Err(_) => Ok(DatasetCount::from(&Dataset::open_ex(&p, driver).map_err(
                |kind| CountError {
                    kind: ErrorKind::Gdal(kind),
                },
            )?)),
        }
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
            ErrorKind::File(e) => Some(e),
            ErrorKind::ParseInt(e) => Some(e),
        }
    }
}

#[derive(Debug)]
pub enum ErrorKind {
    Csv(csv::Error),
    Gdal(GdalError),
    File(io::Error),
    ParseInt(std::num::ParseIntError),
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub struct CountDifference {
    layer: String,
    left_count: Option<u64>,
    right_count: Option<u64>,
    difference: Option<i128>,
}

pub struct CountDifferenceVec(Vec<CountDifference>);

impl CountDifferenceVec {
    /// Create an empty DatasetCount for initialization.
    fn new() -> CountDifferenceVec {
        CountDifferenceVec(Vec::new())
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
}

impl From<Vec<CountDifference>> for CountDifferenceVec {
    fn from(vlc: Vec<CountDifference>) -> Self {
        CountDifferenceVec(vlc)
    }
}

impl FromIterator<CountDifference> for CountDifferenceVec {
    fn from_iter<T: IntoIterator<Item = CountDifference>>(iter: T) -> Self {
        let mut dc = CountDifferenceVec::new();
        for i in iter {
            dc.0.push(i);
        }
        dc
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn layercount_from_string() {
        let s = "layer1,100\n";
        let wanted = LayerCount {
            layer: "layer1".into(),
            count: 100,
        };
        assert_eq!(wanted, LayerCount::from_str(s).unwrap())
    }

    #[test]
    fn datasetcount_from_string() {
        let s = "layer,count
        layer1,100
        layer2,50";
        let wanted = DatasetCount(vec![
            LayerCount {
                layer: "layer1".into(),
                count: 100,
            },
            LayerCount {
                layer: "layer2".into(),
                count: 50,
            },
        ]);
        assert_eq!(wanted, DatasetCount::from_str(s).unwrap());
    }

    #[test]
    fn datasetcount_difference() {
        let dc1 = DatasetCount::from_str(
            "layer,count
        layer1,100
        layer2,50
        layer3,500",
        )
        .unwrap();

        let dc2 = DatasetCount::from_str(
            "layer,count
        layer1,100
        layer2,0",
        )
        .unwrap();

        let diff = dc1.outer_join(&dc2);

        let wanted: Vec<CountDifference> = vec![
            CountDifference {
                layer: "layer1".into(),
                left_count: Some(100),
                right_count: Some(100),
                difference: Some(0),
            },
            CountDifference {
                layer: "layer2".into(),
                left_count: Some(50),
                right_count: Some(0),
                difference: Some(50),
            },
            CountDifference {
                layer: "layer3".into(),
                left_count: Some(500),
                right_count: None,
                difference: Some(500),
            },
        ];

        assert_eq!(diff, wanted);
    }
}
