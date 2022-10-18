use geo::{Centroid, GeoFloat, Line};
use geo_types::Geometry;
use geojson::{Feature, FeatureCollection};
use std::cmp::Eq;
use std::collections::HashMap;
use std::hash::Hash;

// T is the id type which has to support Hash
// A is the precision of the Geometry
pub trait Weights<T, A>
where
    T: Hash + Eq + Clone + std::fmt::Display,
    A: GeoFloat,
{
    fn compute_weights(&mut self, geoms: &[Geometry<A>], ids: &[T]);
    fn weights(&self) -> Option<&HashMap<T, HashMap<T, f64>>>;
    fn are_neighbors(&self, origin: T, dest: T) -> bool {
        match &self.weights() {
            Some(map) => map.get(&origin).unwrap().contains_key(&dest),
            None => false,
        }
    }
    fn get_neighbor_ids(&self, origin: T) -> Option<Vec<T>> {
        match &self.weights() {
            Some(map) => match map.get(&origin) {
                Some(m) => {
                    let results: Vec<T> = m.keys().into_iter().cloned().collect();
                    Some(results)
                }
                None => None,
            },

            None => None,
        }
    }

    fn links_geojson(&self, geoms: &[Geometry<A>], ids: &[T]) -> FeatureCollection {
        let mut features: Vec<Feature> = vec![];

        for (origin, dests) in self.weights().unwrap().iter() {
            for (dest, _weight) in dests.iter() {
                let origin_index = ids.iter().position(|a| a == origin).unwrap();
                let dest_index = ids.iter().position(|a| a == dest).unwrap();
                let origin_centroid = geoms.get(origin_index).unwrap().centroid().unwrap();
                let dest_centroid = geoms.get(dest_index).unwrap().centroid().unwrap();
                let line: geojson::Geometry =
                    geojson::Value::from(&Line::new(origin_centroid, dest_centroid)).into();

                let mut feature = Feature {
                    geometry: Some(line),
                    ..Default::default()
                };
                feature.set_property("origin", format!("{}", origin));
                feature.set_property("dest", format!("{}", dest));
                features.push(feature);
            }
        }
        FeatureCollection {
            features,
            bbox: None,
            foreign_members: None,
        }
    }

    // fn as_geopoalrs(&self, geoms: &[Geometry<A>], ids: &Vec<T>)->Result<DataFrame, Error>{
    //    use polars::io::{SerWriter, SerReader};
    //    use polars::prelude::NamedFromOwned;
    //    use geopolars::geoseries::GeoSeries;

    //    let mut origin_ids : Vec<i32> = Vec::with_capacity(geoms.len());
    //    let mut dests_ids: Vec<i32> = Vec::with_capacity(geoms.len());
    //    let mut weights: Vec<f32> = Vec::with_capacity(geoms.len());
    //    let mut lines: Vec<Line> = Vec::with_capacity(geoms.len());

    //    for (origin, dests) in self.weights().unwrap().iter(){
    //         for (dest, _weight) in dests.iter(){
    //             origin_ids.push(origin);
    //             dest_ids.push(dest);
    //             weights.push(weight);
    //             let origin_index =  ids.iter().position(|a| a == origin).unwrap();
    //             let dest_index =  ids.iter().position(|a| a == dest).unwrap();
    //             let origin_centroid = geoms.get(origin_index).unwrap().centroid().unwrap();
    //             let dest_centroid = geoms.get(dest_index).unwrap().centroid().unwrap();
    //             let line  = Line::new(origin_centroid, dest_centroid);
    //             geoms.push(line);
    //         }
    //    }
    //    let geom_col = Series::from_geom_vec(&lines);
    //    let result = DataFrame:::new([
    //         Series::from_vec("origin_id", origin_ids),
    //         Series::from_vec("dest_id", dests_ids),
    //         Series::from_vec("weight", weights),
    //         Series::from_vec("geom", geoms),
    //    ]);
    //    result
    // }
}
