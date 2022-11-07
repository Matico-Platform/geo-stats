mod utils;

use std::{collections::HashMap, fmt::Display};

use geo::GeometryCollection;
use geo_weights::{Weights, QueensWeights,WeightBuilder};
use geo_stats::lisa::lisa;
use geojson::{GeoJson, quick_collection};
use wasm_bindgen::prelude::*;
use std::ops::Deref;
use std::fmt;
use serde::ser::Serialize;


// pub use wasm_bindgen_rayon::init_thread_pool;


// When the `wee_alloc` feature is enabled, use `wee_alloc` as the global
// allocator.
#[cfg(feature = "wee_alloc")]
#[global_allocator]
static ALLOC: wee_alloc::WeeAlloc = wee_alloc::WeeAlloc::INIT;

#[wasm_bindgen]
pub struct WeightProxy(Weights);

impl fmt::Display for WeightProxy{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "({:#?})", self.0)
    }
}

impl Deref for WeightProxy {
    type Target = Weights;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}


#[wasm_bindgen]
extern "C" {
    // Use `js_namespace` here to bind `console.log(..)` instead of just
    // `log(..)`
    #[wasm_bindgen(js_namespace = console)]
    fn log(s: &str);

    // The `console.log` is quite polymorphic, so we can bind it with multiple
    // signatures. Note that we need to use `js_name` to ensure we always call
    // `log` in JS.
    #[wasm_bindgen(js_namespace = console, js_name = log)]
    fn log_u32(a: u32);

    // Multiple arguments too!
    #[wasm_bindgen(js_namespace = console, js_name = log)]
    fn log_many(a: &str, b: &str);
}


#[wasm_bindgen]
impl WeightProxy{
   
    #[wasm_bindgen]
    pub fn weights(&self) -> Result<JsValue,JsValue>  {
        let serializer = serde_wasm_bindgen::Serializer::new().serialize_maps_as_objects(true);
        Ok(self.0.weights().serialize(&serializer)?)
    }

    #[wasm_bindgen]
    pub fn no_elements(&self) -> Result<JsValue,JsValue>{
        Ok(serde_wasm_bindgen::to_value(&self.0.no_elements())?)
    }

    #[wasm_bindgen]
    pub fn are_neighbors(&self, origin: usize, dest: usize) -> Result<JsValue,JsValue>{
        Ok(serde_wasm_bindgen::to_value(&self.0.are_neighbors(origin, dest))?)
    }

    #[wasm_bindgen]
    pub fn get_neighbor_ids(&self, origin: usize) -> Result<JsValue,JsValue>{
        let ids = self.0.get_neighbor_ids(origin);
        Ok(serde_wasm_bindgen::to_value(&ids)?)
    }

    #[wasm_bindgen]
    pub fn links_geojson(&self, geoms: JsValue) -> Result<JsValue,JsValue>{
        let geoms: GeoJson = serde_wasm_bindgen::from_value(geoms).unwrap();
        let geoms: GeometryCollection = quick_collection(&geoms).unwrap();
        let fc = self.0.links_geojson(&geoms.0);
        let serializer = serde_wasm_bindgen::Serializer::new().serialize_maps_as_objects(true);
        Ok(fc.serialize(&serializer)?)
    }
}


#[wasm_bindgen]
pub fn calc_weights_from_geojson(geo_json: JsValue)->Result<WeightProxy, JsError>{
    let geo_json:GeoJson = serde_wasm_bindgen::from_value(geo_json).unwrap(); 
    let geom_collection : GeometryCollection = quick_collection(&geo_json)
                                                .map_err(|_| JsError::new("Failed to parse geometry collection"))?;
    let weights = QueensWeights::new(10000.0)
                    .compute_weights(&geom_collection.0);
    Ok(WeightProxy(weights))
}

#[wasm_bindgen]
pub fn calc_lisa(weights: &WeightProxy, values: JsValue)->Result<JsValue,JsValue>{
    let values : Vec<f64> = serde_wasm_bindgen::from_value(values)?;
    let result = lisa(&*weights,&values,9999,false,geo_stats::lisa::PermutationMethod::LOOKUP)?;

    Ok(serde_wasm_bindgen::to_value(&result)?)
}
// #[wasm_bindgen]
// pub fn calc_lisa(values: [f64], weights: Weights){

// }
