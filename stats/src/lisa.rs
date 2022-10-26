//use std::async_iter::from_iter;


use geo::GeoFloat;
use geo_weights::weights::{Weights,TransformType, self};
use nalgebra::DVector;
use rand::seq::IteratorRandom;
use rand::thread_rng;
use rand::seq::SliceRandom;
use serde::Serialize;

#[derive(Debug,Serialize)]
pub struct LISAResult{
    pub moran_val: Vec<f64>,
    pub quads: Vec<Quad>,
    pub p_vals: Vec<f64>,
    pub sims: Vec<Vec<f64>>
}

#[derive(Debug,Serialize)]
pub enum Quad{
    HH,
    HL,
    LH,
    LL
}

pub fn lisa (weights: &Weights,values: &[f64], permutations: usize)-> LISAResult 
{
    let x = DVector::from_column_slice(values);

    let mean = x.mean();
    let std = x.variance().sqrt();
    let x_z = (&x - DVector::from_element(x.len(), mean) ) / std;
    let w_matrix = weights.as_sparse_matrix(Some(TransformType::Row));
    
    let lags  : DVector<_> = &w_matrix * &x_z; 

    let quads : Vec<Quad> = x_z.iter().zip(lags.iter()).map(|(a,b)|
        match (a,b){
            (a,b) if *a >0.0 && *b>0.0 => Quad::HH,
            (a,b) if *a >0.0 && *b<0.0 => Quad::HL,
            (a,b) if *a <0.0 && *b>0.0 => Quad::LH,
            (a,b) if *a <0.0 && *b<0.0 => Quad::LL,
            (_,_) => unreachable!()

        }
    ).collect();

   
    let mut results = x_z.component_mul(&lags); 
    let norm =  (x_z.len() as f64 -1.0) / x_z.dot(&x_z);
    results *= norm;

    let mut rng = rand::thread_rng(); 

    let mut sig_counts = DVector::from_element(x.len(), 0.0); 

    let mut sims: Vec<Vec<f64>> = Vec::with_capacity(values.len());
    for _i in 0..values.len(){
        sims.push(Vec::with_capacity(permutations))
    }

    let no_neighbors: Vec<usize> = w_matrix.row_iter().map(|row| row.values().len()).collect();
    let mut sims :Vec<Vec<f64>> = Vec::with_capacity(values.len());

    let mut p_vals :Vec<f64> = Vec::with_capacity(values.len());
    results.iter().zip(no_neighbors).enumerate().for_each(|(index,(moran,values_to_sample))|{

        let collapsed_weights : Vec<f64> = w_matrix.row(index).values().into(); 
        let self_value: f64= *x_z.get(index).unwrap();
        let mut values_with_self_removed: Vec<f64>= x_z.iter().map(|v| *v).collect();
        values_with_self_removed.remove(index);

        // let collapsed_weights = DVector::from_vec(collapsed_weights);
        let sim_vals: Vec<f64> = (0..permutations).map(|_| 
            values_with_self_removed.choose_multiple(&mut rng,  values_to_sample)
                           .zip(&collapsed_weights)
                           .map(|(val,weight)| val*weight)
                           .sum()
                           ).map(|v: f64| v*self_value*norm).collect();
        // println!("sim vals {:#?}", sim_vals);
        // println!("moran {:#?}", moran);
        let mut larger = sim_vals.iter().filter(|v| *v > moran).count();
        if permutations - larger < larger{
            larger = permutations - larger  
        }
        sims.push(sim_vals);

        p_vals.push( (larger as f64 +1.0 )/ (permutations as f64+1.0));

    });
         

    LISAResult{
        moran_val : results.data.into(),
        quads,
        sims,
        p_vals
    }
}



