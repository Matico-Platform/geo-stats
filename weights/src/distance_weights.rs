use crate::{weights::Weights, WeightBuilder};
use geo::centroid::Centroid;
use geo::euclidean_distance::EuclideanDistance;
use geo::GeoFloat;
use geo_types::{Geometry, Point};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Serialize, Deserialize, Debug)]
pub struct DistanceWeights<A>
where
    A: GeoFloat,
{
    cutoff_dist: Option<A>,
    use_distance_as_weight: bool,
}

impl<A> DistanceWeights<A>
where
    A: GeoFloat,
{
    pub fn new(cutoff_dist: Option<A>, use_distance_as_weight: bool) -> Self {
        Self {
            cutoff_dist,
            use_distance_as_weight,
        }
    }
}

impl<A> WeightBuilder<A> for DistanceWeights<A>
where
    A: GeoFloat,
{
    fn compute_weights<T>(&self, geoms: &T) -> Weights
    where for<'a> &'a T: IntoIterator<Item=&'a Geometry<A>>{
        let centroids: Vec<Point<A>> = geoms
            .into_iter()
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
        let mut weights: HashMap<usize, HashMap<usize, f64>> = HashMap::new();

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
                        .entry(i)
                        .or_insert_with(HashMap::new)
                        .entry(j)
                        .or_insert_with(|| w.to_f64().unwrap());
                }
            }
        }

        Weights::new(weights, geoms.into_iter().count())
    }
}
