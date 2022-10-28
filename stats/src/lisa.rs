use geo_weights::weights::{TransformType, Weights};
use nalgebra::DVector;
use rand::seq::index::sample;
use rayon::prelude::*;
use serde::Serialize;

#[derive(Debug, Serialize)]
pub struct LISAResult {
    pub moran_val: Vec<f64>,
    pub quads: Vec<Quad>,
    pub lags: Vec<f64>,
    pub p_vals: Vec<f64>,
    pub sims: Vec<Vec<f64>>,
}

/// Specifies the permutation method to use when performing significance tests 
/// if it's FULL we calculate a random conditional permutation of the observations for every single
/// simulation draw
/// If it's LOOKUP, we generate a set of permutaitons ahead of time corrisoponding to each set of
/// no of neighbors and then this is used as a look up when performing the simulation.
#[derive(Debug, Serialize)]
pub enum PermutationMethod{
    FULL,
    LOOKUP
}

/// Moran Quad definitions
#[derive(Debug, Serialize)]
pub enum Quad {
    HH,
    HL,
    LH,
    LL,
}

/// Generates a nested set of vectors which contain permuted indices for each of the different
/// number of neighbors that we have in the set. These are used to speed up selection of neighbors
/// during the simulation phase of lisa generation
pub fn generate_perturbation_lookups( max_no_neighbors: usize, permutations:usize, no_observations: usize)-> Vec<Vec<Vec<usize>>>{
    (0..max_no_neighbors+1).into_par_iter().map(|no_neighbors|{
        let mut rng = rand::thread_rng();
        (0..permutations).map(|_|{
            sample(&mut rng, no_observations-1, no_neighbors).into_vec()
        }).collect()
    }).collect()
}


/// Computes the LISA stats for the given weights and values. Results are a LisaResult object that
/// contains the 
/// - moran_values: for each observation
/// - lags: the lag value for each observation
/// - quads: the moran quad specification for each observation, 
/// - p_vals: the estimated p_val of each observation 
/// - sims: the simulated moran values for each observation if keep_sims is specified
pub fn lisa(weights: &Weights, values: &[f64], permutations: usize, keep_sims: bool, permutation_method: PermutationMethod) -> LISAResult {

    // Generate a vector from the slice of values we are provided 
    let x = DVector::from_column_slice(values);
    let no_observations = x.len();

    // Standardize the vector by dividing by subtracting off the mean and dividing by the standard
    // deviation. 
    let mean = x.mean();
    let std = x.variance().sqrt();
    let x_z = (&x - DVector::from_element(x.len(), mean)) / std;

    // Get the sparse matrix representation of the weights matrix 
    let w_matrix = weights.as_sparse_matrix(Some(TransformType::Row));

    // Calculate the lags through matrix multiplication
    let lags: DVector<_> = &w_matrix * &x_z;

    // Assign the moran quad to each observation
    let quads: Vec<Quad> = x_z
        .iter()
        .zip(lags.iter())
        .map(|(a, b)| match (a, b) {
            (a, b) if *a >= 0.0 && *b >= 0.0 => Quad::HH,
            (a, b) if *a >= 0.0 && *b < 0.0 => Quad::HL,
            (a, b) if *a < 0.0 && *b >= 0.0 => Quad::LH,
            (a, b) if *a < 0.0 && *b < 0.0 => Quad::LL,
            (_, _) => unreachable!(),
        })
        .collect();

    // Multiply the input values by the lags and normalize to get the moran value 
    let mut results = x_z.component_mul(&lags);
    let norm = (x_z.len() as f64 - 1.0) / x_z.dot(&x_z);
    results *= norm;

    // Next section runs the simulation to determine the significance of each observation
    //
    // First we get the number of neighbors for each observation determined by the no of entries in the 
    // sparse matrix. we also record the max number of neighbors.

    let no_neighbors: Vec<usize> = w_matrix.row_iter().map(|row| row.values().len()).collect();
    let max_neighbors = no_neighbors.iter().max();

    // If the lookup permutations method is specified we generate the lookup table of permutations
    // to be used in the simulation
    let permutation_lookup = match permutation_method{
        PermutationMethod::LOOKUP => Some(generate_perturbation_lookups(*max_neighbors.unwrap(), permutations, no_observations )),
        PermutationMethod::FULL =>None
    } ;

    // Next we iterate over the moran values of our observations, using multiple threads if they are available.  
    let sim_results: Vec<(f64, Vec<f64>)> = results
        .data
        .as_vec()
        .par_iter()
        .zip(no_neighbors)
        .enumerate()
        .map(|(index, (moran, values_to_sample))| {
            // For each observation, because we are randomizing the neighbor observations, we dont
            // need to known which weight corresponds to which neighbor. So we simply collapse them
            // to a vector.
            let collapsed_weights: Vec<f64> = w_matrix.row(index).values().into();
            
            // We get the value at the current observation. 
            let self_value: f64 = *x_z.get(index).unwrap();

            // Then generate a list of observations with that value removed. I suspect there is a
            // way to simply mask the value out rather than having to construct this new array
            // which might be a performance boost in future.
            let mut values_with_self_removed: Vec<f64> = x_z.iter().map(|v| *v).collect();
            values_with_self_removed.remove(index);

            let mut rng = rand::thread_rng();

            // For each permutation we calclate the lags and moran value for this observation.
            let sim_vals: Vec<f64> = (0..permutations)
                .map(|permutation| {
                        
                    let sim_moran = match &permutation_lookup{
                        // if we haven't generated a permutation lookup we sample from the set of
                        // observations directly  
                        None=> sample(&mut rng, no_observations-1, values_to_sample).into_iter()
                               .map(|i| values_with_self_removed[i] )
                               .zip(&collapsed_weights)
                               .map(|(val, weight)| val * weight)
                               .sum(),

                        // if we have generated a permutation lookup we will use it to simply
                        // lookup permuted indices for the correct number of weights
                        Some(lookup)=> lookup[values_to_sample][permutation].iter()
                               .map(|i| values_with_self_removed[*i] )
                               .zip(&collapsed_weights)
                               .map(|(val, weight)| val * weight)
                               .sum()
                    };
                    sim_moran
                })
                .map(|v: f64| v * self_value * norm)
                .collect();

            // Now that we have the moran values for each simulation, we could how many of them are
            // greater or equal to the moran value for the actual dataset 
            let mut larger = sim_vals.iter().filter(|v| *v >= moran).count();

            // Because the distribution has two tails, and can either be in the lower tail or the
            // upper tail, we check to see if most observations are above or bellow the moran value
            // and switch directions if needed
            if permutations - larger < larger {
                larger = permutations - larger;
            }

            // Calculate the sudo p value as defined as the fraction  of observations more extream
            // than the calculated moran value
            let p_val = (larger as f64 + 1.0) / (permutations as f64 + 1.0);

            if keep_sims{
                (p_val, sim_vals)
            }
            else{
                (p_val,vec![])
            }
        })
        .collect();

    LISAResult {
        moran_val: results.data.into(),
        quads,
        lags: lags.data.into(),
        sims: sim_results.iter().map(|r| r.1.clone()).collect(),
        p_vals: sim_results.iter().map(|r| r.0).collect(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use test::Bencher;

    use geo::GeometryCollection;
    use geo_weights::{QueensWeights, WeightBuilder};
    use geojson::{quick_collection, GeoJson};

    #[bench]
    fn real_data_small(b: &mut Bencher) {
        let jsonfile = std::fs::read_to_string(format!(
            "{}/{}",
            std::env::var("CARGO_MANIFEST_DIR").unwrap(),
            "test_data/guerry.geojson"
        ))
        .unwrap();
        let geojson: GeoJson = jsonfile.parse().unwrap();
        let geoms: GeometryCollection<f64> = quick_collection(&geojson).unwrap();
        let weight_builder = QueensWeights::new(10000.0);
        let weights = weight_builder.compute_weights(&geoms.0);

        if let GeoJson::FeatureCollection(fc) = geojson {
            let values: Vec<f64> = fc
                .features
                .iter()
                .map(|f| f.property("Donatns").unwrap().as_f64().unwrap())
                .collect();

            b.iter(|| {
                lisa(&weights, &values, 9999, false, PermutationMethod::LOOKUP);
            })
        } else {
            panic!("Expected data to be a feature collection")
        }
    }

    #[bench]
    fn real_data_large(b: &mut Bencher) {
        let jsonfile = std::fs::read_to_string(format!(
            "{}/{}",
            std::env::var("CARGO_MANIFEST_DIR").unwrap(),
            "test_data/covid.geojson"
        ))
        .unwrap();
        let geojson: GeoJson = jsonfile.parse().unwrap();
        let geoms: GeometryCollection<f64> = quick_collection(&geojson).unwrap();
        let weight_builder = QueensWeights::new(10000.0);
        let weights = weight_builder.compute_weights(&geoms.0);

        if let GeoJson::FeatureCollection(fc) = geojson {
            let values: Vec<f64> = fc
                .features
                .iter()
                .map(|f| f.property("cases").unwrap().as_f64().unwrap())
                .collect();

            b.iter(|| {
                lisa(&weights, &values, 9999, false, PermutationMethod::LOOKUP);
            })
        } else {
            panic!("Expected data to be a feature collection")
        }
    }
}
