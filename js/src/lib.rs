mod utils;

use geo_weights::Weights;
use geojson::GeoJson;
use wasm_bindgen::prelude::*;
// pub use wasm_bindgen_rayon::init_thread_pool;


// When the `wee_alloc` feature is enabled, use `wee_alloc` as the global
// allocator.
#[cfg(feature = "wee_alloc")]
#[global_allocator]
static ALLOC: wee_alloc::WeeAlloc = wee_alloc::WeeAlloc::INIT;

#[wasm_bindgen]
extern {
    fn alert(s: &str);
}

#[wasm_bindgen]
pub fn greet() {
    alert("Hello, js!");
}

#[wasm_bindgen]
pub fn calc_weights_from_geojson(geoJson: &JSValue){
    let geojson:GeoJson = geoJson.into_serde().unwrap(); 
}

#[wasm_bindgen]
pub fn calc_lisa(values: [f64], weights: Weights){

}
