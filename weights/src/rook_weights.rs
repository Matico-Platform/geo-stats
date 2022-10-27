use crate::weights::Weights;
use crate::{utils::coords_to_tolerance, WeightBuilder};
use geo::algorithm::coords_iter::CoordsIter;
use geo::GeoFloat;
use geo_types::Geometry;
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};

// T is the type being used to index our geometries
// A is the type of the weight we are computing

#[derive(Serialize, Deserialize, Debug)]
pub struct RookWeights<A>
where
    A: GeoFloat,
{
    tolerance: A,
}

impl<A> RookWeights<A>
where
    A: GeoFloat,
{
    pub fn new(tolerance: A) -> Self {
        Self { tolerance }
    }
}

impl<A> WeightBuilder<A> for RookWeights<A>
where
    A: GeoFloat,
{
    fn compute_weights(&self, geoms: &[Geometry<A>]) -> Weights {
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

        let mut weights: HashMap<usize, HashMap<usize, f64>> = HashMap::new();
        for (_coords, values) in coord_hash.iter() {
            for index in values.iter() {
                weights
                    .entry(*index)
                    .and_modify(|entry| {
                        for index2 in values.iter() {
                            if index != index2 {
                                entry.insert(*index2, 1.0);
                            }
                        }
                    })
                    .or_insert_with(|| HashMap::new());
            }
        }

        Weights::new(weights, geoms.len(), HashSet::new())
    }
}
