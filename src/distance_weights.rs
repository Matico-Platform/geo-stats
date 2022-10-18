use crate::weights::Weights;
use geo::centroid::Centroid;
use geo::euclidean_distance::EuclideanDistance;
use geo::GeoFloat;
use geo_types::{Geometry, Point};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::hash::Hash;

#[derive(Serialize, Deserialize, Debug)]
pub struct DistanceWeights<T, A>
where
    T: Hash + Eq + Clone + std::fmt::Display,
    A: GeoFloat,
{
    cutoff_dist: Option<A>,
    use_distance_as_weight: bool,
    weights: Option<HashMap<T, HashMap<T, f64>>>,
}

impl<T, A> DistanceWeights<T, A>
where
    T: Hash + Eq + Clone + std::fmt::Display,
    A: GeoFloat,
{
    pub fn new(cutoff_dist: Option<A>, use_distance_as_weight: bool) -> Self {
        Self {
            cutoff_dist,
            use_distance_as_weight,
            weights: None,
        }
    }
}

impl<T, A> Weights<T, A> for DistanceWeights<T, A>
where
    T: Hash + Eq + Clone + std::fmt::Display,
    A: GeoFloat,
{
    fn weights(&self) -> Option<&HashMap<T, HashMap<T, f64>>> {
        self.weights.as_ref()
    }

    fn compute_weights(&mut self, geoms: &[Geometry<A>], ids: &[T]) {
        let centroids: Vec<Point<A>> = geoms
            .iter()
            .map(|geom| match geom {
                Geometry::Point(p) => *p,
                Geometry::Polygon(p) => p
                    .centroid()
                    .expect("Polygon Geometry invalid, could not compute centroid"),
                Geometry::MultiPolygon(p) => p
                    .centroid()
                    .expect("MultiPolygon Geometry invalid, could not compute centroid"),
                _ => panic!("Geometry not supported"),
            })
            .collect();
        let mut weights: HashMap<T, HashMap<T, f64>> = HashMap::new();

        for i in 0..centroids.len() {
            for j in 0..centroids.len() {
                if i == j {
                    continue;
                }
                let dist = centroids[i].euclidean_distance(&centroids[j]);
                let weight: Option<A> = match (self.cutoff_dist, self.use_distance_as_weight) {
                    (Some(cutoff), true) => {
                        if dist < cutoff {
                            Some(dist)
                        } else {
                            None
                        }
                    }
                    (Some(cutoff), false) => {
                        if dist < cutoff {
                            Some(A::one())
                        } else {
                            None
                        }
                    }
                    (None, true) => Some(dist),
                    _ => panic!("Need to specify either a cutoff or use dist as weight"),
                };
                if let Some(w) = weight {
                    weights
                        .entry(ids[i].clone())
                        .or_insert_with(HashMap::new)
                        .entry(ids[j].clone())
                        .or_insert_with(|| w.to_f64().unwrap());
                }
            }
        }

        self.weights = Some(weights)
    }
}
