use crate::weights::Weights;
use crate::{utils::coords_to_tolerance, WeightBuilder};
use geo::algorithm::coords_iter::CoordsIter;
use geo::GeoFloat;
use geo_types::Geometry;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Serialize, Deserialize, Debug)]
pub struct QueensWeights<A>
where
    A: GeoFloat,
{
    tolerance: A,
}

impl<A> QueensWeights<A>
where
    A: GeoFloat,
{
    pub fn new(tolerance: A) -> Self {
        Self { tolerance }
    }
}

impl<A> WeightBuilder<A> for QueensWeights<A>
where
    A: GeoFloat,
{
    fn compute_weights(&self, geoms: &[Geometry<A>]) -> Weights {
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
                    .or_insert_with(HashMap::new);
            }
        }

        Weights::new(weights, geoms.len())
    }
}
