use geo::GeometryCollection;
use geo_stats::lisa::{lisa, PermutationMethod};
use geo_weights::{QueensWeights, WeightBuilder};
use geojson::GeoJson;

fn main() {
    let jsonfile = std::fs::read_to_string(format!(
        "{}/{}",
        std::env::var("CARGO_MANIFEST_DIR").unwrap(),
        "test_data/covid.geojson"
    ))
    .unwrap();
    let geojson: GeoJson = jsonfile.parse().unwrap();
    let geoms: GeometryCollection<f64> = geojson::quick_collection(&geojson).unwrap();
    let weight_builder = QueensWeights::new(10000.0);
    let weights = weight_builder.compute_weights(&geoms.0);

    if let GeoJson::FeatureCollection(fc) = geojson {
        let values: Vec<f64> = fc
            .features
            .iter()
            .map(|f| f.property("cases").unwrap().as_f64().unwrap())
            .collect();

        lisa(&weights, &values, 9999, false, PermutationMethod::LOOKUP);
    } else {
        panic!("Expected data to be a feature collection")
    }
}
