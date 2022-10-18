use crate::utils::coords_to_tolerance;
use crate::weights::Weights;
use geo::algorithm::coords_iter::CoordsIter;
use geo::GeoFloat;
use geo_types::Geometry;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::hash::Hash;

// T is the type being used to index our geometries
// A is the type of the weight we are computing

#[derive(Serialize, Deserialize, Debug)]
pub struct RookWeights<T, A>
where
    T: Hash + Eq + Clone + std::fmt::Display,
    A: GeoFloat,
{
    weights: Option<HashMap<T, HashMap<T, f64>>>,
    tolerance: A,
}

impl<T, A> RookWeights<T, A>
where
    T: Hash + Eq + Clone + std::fmt::Display,
    A: GeoFloat,
{
    pub fn new(tolerance: A) -> Self {
        Self {
            weights: None,
            tolerance,
        }
    }
}

impl<T, A> Weights<T, A> for RookWeights<T, A>
where
    T: Hash + Eq + Clone + std::fmt::Display,
    A: GeoFloat,
{
    fn compute_weights(&mut self, geoms: &[Geometry<A>], ids: &[T]) {
        let mut coord_hash: HashMap<[isize; 4], Vec<usize>> = HashMap::new();

        for (index, geom) in geoms.iter().enumerate() {
            for (prev_coords, next_coords) in geom.coords_iter().zip(geom.coords_iter().skip(1)) {
                let prev_coords_hashed = coords_to_tolerance(prev_coords, 1000.0);
                let next_coords_hashed = coords_to_tolerance(next_coords, 1000.0);
                let hashed_coords = [
                    prev_coords_hashed.0,
                    prev_coords_hashed.1,
                    next_coords_hashed.0,
                    next_coords_hashed.1,
                ];
                coord_hash
                    .entry(hashed_coords)
                    .and_modify(|v| v.push(index))
                    .or_insert_with(|| vec![index]);
            }
        }

        let mut weights: HashMap<T, HashMap<T, f64>> = HashMap::new();
        for (_coords, values) in coord_hash.iter() {
            for index in values.iter() {
                weights
                    .entry(ids[*index].clone())
                    .and_modify(|entry| {
                        for index2 in values.iter() {
                            if index != index2 {
                                entry.insert(ids[*index2].clone(), 1.0);
                            }
                        }
                    })
                    .or_insert_with(|| HashMap::new());
            }
        }
        self.weights = Some(weights);
    }

    fn weights(&self) -> Option<&HashMap<T, HashMap<T, f64>>> {
        self.weights.as_ref()
    }
}
