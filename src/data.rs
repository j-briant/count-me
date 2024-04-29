use gdal::{errors::GdalError, DriverManager};

pub fn drivers() -> Result<Vec<String>, GdalError> {
    DriverManager::register_all();
    let count = DriverManager::count();
    let mut list: Vec<String> = vec![];
    for i in 0..count {
        list.push(DriverManager::get_driver(i)?.short_name())
    }
    Ok(list)
}
