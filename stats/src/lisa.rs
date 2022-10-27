use geo_weights::weights::{TransformType, Weights};
use nalgebra::DVector;
use rand::seq::{SliceRandom, IteratorRandom, index::sample};
use rayon::prelude::*;
use serde::Serialize;

#[derive(Debug, Serialize)]
pub struct LISAResult {
    pub moran_val: Vec<f64>,
    pub quads: Vec<Quad>,
    pub p_vals: Vec<f64>,
    pub sims: Vec<Vec<f64>>,
}

#[derive(Debug, Serialize)]
pub enum PermutationMethod{
    FULL,
    LOOKUP
}

#[derive(Debug, Serialize)]
pub enum Quad {
    HH,
    HL,
    LH,
    LL,
}

pub fn generate_perturbation_lookups( max_no_neighbors: usize, permutations:usize, no_observations: usize)-> Vec<Vec<Vec<usize>>>{
    let lookup: Vec<Vec<Vec<usize>>> = (0..max_no_neighbors+1).into_par_iter().map(|no_neighbors|{
        let mut rng = rand::thread_rng();
        let inner_lookup: Vec<Vec<usize>> = (0..permutations).map(|_|{
            let sample = sample(&mut rng, no_observations-1, no_neighbors).into_vec();
            sample
        }).collect();
        inner_lookup
    }).collect();
    println!("Lookup dim 1: {} 2: {} 3: {} observations {}", lookup.len(), lookup[0].len(), lookup[0][0].len(), no_observations);
    lookup
}


pub fn lisa(weights: &Weights, values: &[f64], permutations: usize, keep_sims: bool, permutation_method: PermutationMethod) -> LISAResult {
    let x = DVector::from_column_slice(values);
    let no_observations = x.len();

    let mean = x.mean();
    let std = x.variance().sqrt();
    let x_z = (&x - DVector::from_element(x.len(), mean)) / std;
    let w_matrix = weights.as_sparse_matrix(Some(TransformType::Row));
    let lags: DVector<_> = &w_matrix * &x_z;

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

    let mut results = x_z.component_mul(&lags);
    let norm = (x_z.len() as f64 - 1.0) / x_z.dot(&x_z);
    results *= norm;

    let no_neighbors: Vec<usize> = w_matrix.row_iter().map(|row| row.values().len()).collect();
    let max_neighbors = no_neighbors.iter().max();

    let permutation_lookup = match permutation_method{
        PermutationMethod::LOOKUP => Some(generate_perturbation_lookups(*max_neighbors.unwrap(), permutations, no_observations )),
        PermutationMethod::FULL =>None
    } ;

    
    let sim_results: Vec<(f64, Vec<f64>)> = results
        .data
        .as_vec()
        .par_iter()
        .zip(no_neighbors)
        .enumerate()
        .map(|(index, (moran, values_to_sample))| {
            let collapsed_weights: Vec<f64> = w_matrix.row(index).values().into();
            let self_value: f64 = *x_z.get(index).unwrap();
            let mut values_with_self_removed: Vec<f64> = x_z.iter().map(|v| *v).collect();
            values_with_self_removed.remove(index);
            let mut rng = rand::thread_rng();

            // let collapsed_weights = DVector::from_vec(collapsed_weights);
            let sim_vals: Vec<f64> = (0..permutations)
                .map(|permutation| {

                    let sim_moran = match &permutation_lookup{
                        None=> sample(&mut rng, no_observations-1, values_to_sample).into_iter()
                               .map(|i| values_with_self_removed[i] )
                               .zip(&collapsed_weights)
                               .map(|(val, weight)| val * weight)
                               .sum(),

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
            let mut larger = sim_vals.iter().filter(|v| *v > moran).count();

            if permutations - larger < larger {
                larger = permutations - larger;
            }

            let p_val = (larger as f64 + 1.0) / (permutations as f64 + 1.0);

            (p_val, sim_vals)
        })
        .collect();

    LISAResult {
        moran_val: results.data.into(),
        quads,
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
