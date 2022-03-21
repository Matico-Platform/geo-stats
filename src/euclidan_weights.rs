use std::collections::{HashMap};
use std::hash::Hash;
use num_traits::Float;
use num_traits::FromPrimitive;
use std::iter::Sum;
use crate::weights::Weights;
use geo_types::{Geometry,Point};
use geo::centroid::Centroid;
use geo::euclidean_distance::EuclideanDistance;
use serde::{Serialize, Deserialize};

#[derive(Serialize, Deserialize, Debug)]
pub struct EuclidanWeights <T,A>
where T:Hash + Eq + Clone +  std::fmt::Display,
      A:Float + FromPrimitive + Sum {
    cutoff_dist: Option<A>,
    use_distance_as_weight:bool,
    weights: Option<HashMap<T, HashMap<T,A>>>,
}

impl <T,A> EuclidanWeights <T,A>
    where T: Hash + Eq + Clone + std::fmt::Display,
          A: Float + FromPrimitive + Sum
{
    pub fn new(cutoff_dist: Option<A>, use_distance_as_weight: bool)->Self{
        Self{
            cutoff_dist,
            use_distance_as_weight,
            weights: None
        }
    }
}

impl<T,A> Weights<T,A> for EuclidanWeights<T,A> 
where T:Hash + Eq + Clone + std::fmt::Display,
      A:Float + FromPrimitive + Sum
{

    fn weights(&self)->Option<&HashMap<T, HashMap<T,A>>>{
        self.weights.as_ref()
    } 
    // fn get_neighbor_weights(&self, origin:T)->Option<&HashMap<T,A>>{
    //     match self.weights{
    //         Some(weights)=> weights.get(&origin)
    //         ,
    //         None => None
    //     }
    // }


    fn compute_weights(&mut self, geoms: &[Geometry<A>], ids:Vec<T> ){
        let centroids: Vec<Point<A>> = geoms.iter().map(|geom| 
            match geom{
                Geometry::Point(p)=> *p,
                Geometry::Polygon(p) => p.centroid().expect("Polygon Geometry invalid, could not compute centroid"),
                Geometry::MultiPolygon(p)=>p.centroid().expect("MultiPolygon Geometry invalid, could not compute centroid"),
                _ => panic!("Geometry not supported")
            }
        ).collect();
        let mut weights:HashMap<T,HashMap<T,A>> = HashMap::new();
        let mut weight_count =0;

        for i in 0..centroids.len(){
            for j in 0..centroids.len(){
                if i==j {
                    continue
                }
                let dist= centroids[i].euclidean_distance(&centroids[j]);
                let weight : Option<A> = match (self.cutoff_dist,self.use_distance_as_weight ){
                    (Some(cutoff), true) => {
                        if dist < cutoff { Some(dist) } else {None}  
                    },
                    (Some(cutoff), false)=>{
                        if dist < cutoff {Some(A::one())} else {None}
                    },
                    (None, true) =>{
                        Some(dist)
                    },
                    _=> panic!("Need to specify either a cutoff or use dist as weight")
                };
                if let Some(w) = weight{
                        weight_count+=1;
                        weights.entry(ids[i].clone())
                       .or_insert_with(HashMap::new)
                       .entry(ids[j].clone())
                       .or_insert(w);
                    }
            }
        }
        println!("Generated weights {} ", weight_count);
        for key in weights.keys(){
            println!("Key is {}",key)
        }
        self.weights = Some(weights)
    }
}


#[cfg(test)]
mod test {
    use crate::{EuclidanWeights, Weights};
    use geo_types::{Point, Geometry};
    #[test]
    fn non_weighted_euclid_weight_should_include_points_under_the_threshold_and_not_above() {
        let mut weights : EuclidanWeights<usize,f64> = EuclidanWeights::new(Some(20.0), false);
        let points:Vec<Geometry<f64>> = vec![
            Point::new(1.0,2.0).into(),
            Point::new(100.0,0.0).into(),
            Point::new(2.0,2.0).into()
        ];

        let ids: Vec<usize> = vec![
            0,
            1,
            2
        ];

        weights.compute_weights(&points, ids);
        println!("weights are {:?}", weights);
        let n1 = weights.get_neighbor_ids(0);
        let n2 = weights.get_neighbor_ids(1);
        let n3 = weights.get_neighbor_ids(2);
       
        let neighbors_for_one = n1.unwrap();
        let neighbors_for_two = n2;
        let neighbors_for_three = n3.unwrap();

        println!("n1 is {:?}", neighbors_for_one);
        println!("n3 is {:?}", neighbors_for_three);
        assert!(neighbors_for_one.contains(&2));
        assert_eq!(neighbors_for_two, None);
        assert!(neighbors_for_three.contains(&0));
    }

    #[test]
    fn weighted_euclid_weights_should_compute_correct_weight(){
        let mut weights : EuclidanWeights<usize,f64> = EuclidanWeights::new(Some(20.0), true);
        let points:Vec<Geometry<f64>> = vec![
            Point::new(1.0,2.0).into(),
            Point::new(100.0,0.0).into(),
            Point::new(2.0,2.0).into()
        ];

        let ids: Vec<usize> = vec![
            0,
            1,
            2
        ];
        weights.compute_weights(&points, ids)
    }
}
