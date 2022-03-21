use std::collections::HashMap;
use geo_types::Geometry;
use std::hash::Hash;
use num_traits::float::Float;
use num_traits::FromPrimitive;
use std::cmp::Eq;
use std::iter::Sum;


// T is the id type which has to support Hash
// A is the precision of the Geoemtry
pub trait Weights<T,A> 
where T: Hash + Eq + Clone +std::fmt::Display,
      A: Float + FromPrimitive + Sum{

    fn compute_weights(& mut self, geoms:&[Geometry<A>], ids:Vec<T>);
    fn weights(&self)->Option<&HashMap<T,HashMap<T,A>>>;
    fn are_neighbors(&self, origin: T, dest: T)->bool{
        match &self.weights(){
            Some(map)=> map.get(&origin).unwrap().contains_key(&dest),
            None => false
        }
    }
    fn get_neighbor_ids(&self, origin:T) -> Option<Vec<T>>{
        match &self.weights(){
            Some(map)=> match map.get(&origin){
                Some(m) => {
                    let results:Vec<T> = m.keys().into_iter().cloned().collect();
                    Some(results)
                },
                None => None
           },
            
            None =>  None 
        }
    }

}


