use crate::utils::coords_to_tolerance;
use crate::weights::Weights;
use geo::algorithm::coords_iter::CoordsIter;
use geo::GeoFloat;
use geo_types::Geometry;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::hash::Hash;

#[derive(Serialize, Deserialize, Debug)]
pub struct QueensWeights<T, A>
where
    T: Hash + Eq + Clone + std::fmt::Display,
    A: GeoFloat,
{
    weights: Option<HashMap<T, HashMap<T, f64>>>,
    tolerance: A,
}

impl<T, A> QueensWeights<T, A>
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

impl<T, A> Weights<T, A> for QueensWeights<T, A>
where
    T: Hash + Eq + Clone + std::fmt::Display,
    A: GeoFloat,
{
    fn compute_weights(&mut self, geoms: &[Geometry<A>], ids: &[T]) {
        let mut coord_hash: HashMap<(isize, isize), Vec<usize>> = HashMap::new();
        for (index, geom) in geoms.iter().enumerate() {
            for coords in geom.coords_iter() {
                let hashed_coords = coords_to_tolerance(coords, 1000.0);
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
