use geo::{Geometry, GeometryCollection};
use geojson::{quick_collection, GeoJson};
use std::env;
use std::fs;

pub fn tracts() -> Vec<Geometry<f64>> {
    let test_data_dir = format!(
        "{}/test_data/ny_tracts.geojson",
        env::var("CARGO_MANIFEST_DIR").unwrap()
    );
    let geojson_str = fs::read_to_string(test_data_dir).expect("Failed to load the geojson file");
    let geojson: GeoJson = geojson_str.parse().unwrap();
    let geoms: GeometryCollection<f64> = quick_collection(&geojson).unwrap();
    geoms.0
}
