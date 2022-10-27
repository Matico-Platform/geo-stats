use geo::{Centroid, GeoFloat, Line};
use geo_types::Geometry;
use geojson::{Feature, FeatureCollection};
use nalgebra_sparse::{coo::CooMatrix, csr::CsrMatrix};
use std::collections::{HashMap, HashSet};
use std::fmt;

pub enum TransformType {
    Row,
    Binary,
    DoublyStandardized,
}

pub trait WeightBuilder<A>
where
    A: GeoFloat,
{
    fn compute_weights(&self, geoms: &[Geometry<A>]) -> Weights;
}

// A is the precision of the Geometry
pub struct Weights {
    weights: HashMap<usize, HashMap<usize, f64>>,
    no_elements: usize,
    islands: HashSet<usize>,
}

impl fmt::Display for Weights {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "({:#?})", self.weights())
    }
}

impl Weights {
    pub fn new(
        weights: HashMap<usize, HashMap<usize, f64>>,
        no_elements: usize,
        islands: HashSet<usize>,
    ) -> Weights {
        Self {
            weights,
            no_elements,
            islands,
        }
    }

    pub fn weights(&self) -> &HashMap<usize, HashMap<usize, f64>> {
        &self.weights
    }

    pub fn no_elements(&self) -> usize {
        self.no_elements
    }

    pub fn are_neighbors(&self, origin: usize, dest: usize) -> bool {
        self.weights.get(&origin).unwrap().contains_key(&dest)
    }

    pub fn get_neighbor_ids(&self, origin: usize) -> Option<Vec<usize>> {
        match self.weights.get(&origin) {
            Some(m) => {
                let results: Vec<usize> = m.keys().into_iter().cloned().collect();
                Some(results)
            }
            None => None,
        }
    }

    pub fn as_sparse_matrix(&self, transform: Option<TransformType>) -> CsrMatrix<f64> {
        let mut coo_matrix = CooMatrix::new(self.no_elements, self.no_elements);

        for (key, vals) in self.weights.iter() {
            let norm: f64 = match &transform {
                Some(TransformType::Row) => vals.values().sum(),
                _ => 1.0,
            };
            for (key2, weight) in vals.iter() {
                coo_matrix.push(*key, *key2, *weight / norm);
            }
        }

        let csr = CsrMatrix::from(&coo_matrix);
        csr
    }

    pub fn links_geojson<A: GeoFloat>(&self, geoms: &[Geometry<A>]) -> FeatureCollection {
        let mut features: Vec<Feature> = vec![];

        for (origin, dests) in self.weights.iter() {
            for (dest, _weight) in dests.iter() {
                let origin_centroid = geoms.get(*origin).unwrap().centroid().unwrap();
                let dest_centroid = geoms.get(*dest).unwrap().centroid().unwrap();
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

    // pub fn as_geopoalrs(&self, geoms: &[Geometry<A>], ids: &Vec<T>)->Result<DataFrame, Error>{
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
