use crate::{Weights};


// T is the type being used to index our geometries
// A is the type of the weight we are computing 
pub struct QueensWeights <T,A>
where T:Hash + Eq + Clone +  std::fmt::Display,
      A:Float + FromPrimitive + Sum {
    weights: Option<HashMap<T, HashMap<T,A>>>,
}


impl<T,A> Weights<T,A> for EuclidanWeights<T,A>
    where T: Hash + Eq + Clone + std::fmt::Display,
          A: Float + FromPrimitive + Sum
{
    pub get
}

