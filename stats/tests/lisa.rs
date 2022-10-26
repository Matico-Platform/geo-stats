#![feature(test)]

extern crate test;

use geo_stats::lisa::lisa;
use geo_weights::{QueensWeights, Weights, WeightBuilder};
use geo::{ polygon, Geometry, GeometryCollection};
use geojson::{quick_collection, GeoJson, FeatureCollection};
use std::collections::{HashSet,HashMap};
use std::fs::File;
use std::io::Write;
use test::{Bencher,black_box};


#[macro_use]
extern crate approx;

#[test]
fn lisa_should_produce_correct_values(){
    let mut values: Vec<f64>= vec![2.24, 3.1, 4.55, -5.15, -4.39, 0.46, 5.54, 9.02, -2.09, -3.06];

    let mut dict : HashMap<usize, HashMap<usize,f64>>  =HashMap::new();
   
    dict.insert(0, HashMap::from([(1,1.0), (3,1.0)]));
    dict.insert(1, HashMap::from([(0,1.0), (4,1.0)]));
    dict.insert(2, HashMap::from([(3,1.0), (6,1.0)]));
    dict.insert(3, HashMap::from([(0,1.0), (2,1.0), (4, 1.0), (7,1.0)]));
    dict.insert(4, HashMap::from([(1,1.0), (3,1.0), (5,1.0), (8,1.0) ]));
    dict.insert(5, HashMap::from([(4,1.0), (9,1.0)]));
    dict.insert(6, HashMap::from([(2,1.0), (7,1.0)]));
    dict.insert(7, HashMap::from([(3,1.0), (6,1.0),(8,1.0)]));
    dict.insert(8, HashMap::from([(4,1.0), (7,1.0), (9,1.0)]));
    dict.insert(9, HashMap::from([(5,1.0), (8,1.0)]));

    let weights = Weights::new(dict,10,HashSet::new());
    
    let moran = lisa(&weights,&values, 9999);
    let expected : Vec<f64>= vec![
        -0.11409277104439629,
        -0.19940542567754874,
        -0.13351408484935268,
        -0.5177038320446772,
        0.480950090494397,
        0.12208113114030265,
        0.18876001422853794,
        -0.5814430454766123,
        0.07101382808078055,
        0.3431430079934854,
    ];

    relative_eq!(moran.moran_val.iter().sum::<f64>(), expected.iter().sum());

    let j = serde_json::to_string(&moran).unwrap();
    let mut file = File::create("results.json").unwrap();


}

#[test]
fn real_data(){
    let jsonfile = std::fs::read_to_string( format!("{}/{}",std::env::var("CARGO_MANIFEST_DIR").unwrap(),"test_data/guerry.geojson" )).unwrap();
    let geojson: GeoJson = jsonfile.parse().unwrap();
    let geoms: GeometryCollection<f64> = quick_collection(&geojson).unwrap();
    let weight_builder = QueensWeights::new(10000.0);
    let weights = weight_builder.compute_weights(&geoms.0);
    
    if let GeoJson::FeatureCollection(fc) = geojson{
        let values: Vec<f64> = fc.features.iter().map(|f| f.property("Donatns").unwrap().as_f64().unwrap()).collect();


        let lisa_results = lisa(&weights,&values,9999);
        println!("{:#?}, {:#?}", lisa_results.moran_val, lisa_results.p_vals);
        let j = serde_json::to_string(&lisa_results).unwrap();
        let mut file = File::create("results_real.json").unwrap();
        write!(file,"{}",j);
    }
    else{
        panic!("Expected data to be a feature collection")
    }

}

#[bench]
fn real_data_bench(b: &mut Bencher){
    let jsonfile = std::fs::read_to_string( format!("{}/{}",std::env::var("CARGO_MANIFEST_DIR").unwrap(),"test_data/guerry.geojson" )).unwrap();
    let geojson: GeoJson = jsonfile.parse().unwrap();
    let geoms: GeometryCollection<f64> = quick_collection(&geojson).unwrap();
    let weight_builder = QueensWeights::new(10000.0);
    let weights = weight_builder.compute_weights(&geoms.0);
    
    if let GeoJson::FeatureCollection(fc) = geojson{
        let values: Vec<f64> = fc.features.iter().map(|f| f.property("Donatns").unwrap().as_f64().unwrap()).collect();
            
        b.iter(||{
            black_box(lisa(&weights,&values,9999));
        })

    }
    else{
        panic!("Expected data to be a feature collection")
    }

}
