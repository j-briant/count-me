use std::collections::HashMap;
use std::fs;

use gdal::{vector::LayerAccess, Dataset, DatasetOptions, GdalOpenFlags};

#[derive(Debug)]
struct LayerCount(HashMap<String, u64>);

impl From<&Dataset> for LayerCount {
    fn from(d: &Dataset) -> Self {
        let mut h = HashMap::new();
        for layer in d.layers() {
            h.insert(layer.name(), layer.feature_count());
        }
        LayerCount(h)
    }
}

fn main() {
    let paths = fs::read_dir("./tests/").unwrap();
    let mut data: Vec<Dataset> = vec![];
    for p in paths {
        if let Ok(path) = p {
            if let Ok(d) = Dataset::open(path.path()) {
                data.push(d);
            }
        }
    }

    let mut lc: Vec<LayerCount> = vec![];

    for d in data.iter() {
        lc.push(d.into())
    }
    println!("{:?}", lc);

    let test = Dataset::open("./tests/").unwrap();
    let other_lc = LayerCount::from(&test);
    println!("{:?}", other_lc);

    let db_test = Dataset::open_ex("PG:dbname=osm_suisse", DatasetOptions{open_flags: GdalOpenFlags::GDAL_OF_ALL}).unwrap();
    let other_lc2 = LayerCount::from(&db_test);
    println!("{:?}", other_lc2);
}
