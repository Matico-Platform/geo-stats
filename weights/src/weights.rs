use geo::{Centroid, GeoFloat, Line};
use geo_types::Geometry;
use geojson::{Feature, FeatureCollection};
use nalgebra_sparse::{coo::CooMatrix, csr::CsrMatrix};
use std::collections::{HashMap, HashSet};
use std::fmt;
use std::iter::IntoIterator;

pub enum TransformType {
    Row,
    Binary,
    DoublyStandardized,
}

pub trait WeightBuilder<A>
where
    A: GeoFloat,
{
    fn compute_weights<T>(&self, geoms: &T) -> Weights
    where
        for<'a> &'a T: IntoIterator<Item = &'a Geometry<A>>;
}

/// Structure holding and providing methods to access and query a weights matrix. These are either
/// loaded from external representations or constructed from WeightBuilders.
#[derive(Debug)]
pub struct Weights {
    weights: HashMap<usize, HashMap<usize, f64>>,
    no_elements: usize,
}

impl fmt::Display for Weights {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "({:#?})", self.weights())
    }
}

impl Weights {
    /// Create a new weights object from a hashmap indicating origin and destination ids and
    /// weights.
    ///
    /// # Arguments
    ///
    /// * `weights` - A mapping of {origin => dest =>weight}f64
    /// * `no_elements` - The number of elements in the original geometry set (because we want to be
    /// sure of the full length given this is a sparse representation)
    ///
    pub fn new(weights: HashMap<usize, HashMap<usize, f64>>, no_elements: usize) -> Weights {
        Self {
            weights,
            no_elements,
        }
    }

    /// Create a new weights object from a series of lists representing the origin, destinations
    /// and weights of the matrix
    ///
    /// # Arguments
    ///
    /// * `origins` - A  list of the origin ids
    /// * `origins` - A  list of the destination ids
    /// * `weights` - A  list of the weights
    /// * `no_elements` - The number of elements in the original geometry set (because we want to be
    /// sure of the full length given this is a sparse representation)
    ///
    pub fn from_list_rep<T, W>(origins: &T, dests: &T, weights: &W, no_elements: usize) -> Weights
    where
        for<'a> &'a T: std::iter::IntoIterator<Item = &'a usize>,
        for<'a> &'a W: std::iter::IntoIterator<Item = &'a f64>,
    {
        let mut weights_lookup: HashMap<usize, HashMap<usize, f64>> = HashMap::new();

        for ((origin, dest), weight) in origins
            .into_iter()
            .zip(dests.into_iter())
            .zip(weights.into_iter())
        {
            let entry = weights_lookup.entry(*origin).or_insert(HashMap::new());
            entry.insert(*dest, *weight);

            let entry = weights_lookup.entry(*dest).or_insert(HashMap::new());
            entry.insert(*origin, *weight);
        }
        Self {
            weights: weights_lookup,
            no_elements,
        }
    }

    /// Return a reference to the hash map representation of the weights
    pub fn weights(&self) -> &HashMap<usize, HashMap<usize, f64>> {
        &self.weights
    }

    /// Return the total number of elements in the original geometry set
    pub fn no_elements(&self) -> usize {
        self.no_elements
    }

    /// Returns true if the origin and destination are neighbors
    ///
    /// # Arguments
    ///
    /// * `origin` - the id of the origin geometry
    /// * `destination` - the id of the destination geometry
    ///
    pub fn are_neighbors(&self, origin: usize, dest: usize) -> bool {
        self.weights.get(&origin).unwrap().contains_key(&dest)
    }

    /// Returns the ids of a given geometries neighbors
    ///
    /// # Arguments
    ///
    /// * `origin` - the id of the origin geometry
    ///
    pub fn get_neighbor_ids(&self, origin: usize) -> Option<HashSet<usize>> {
        match self.weights.get(&origin) {
            Some(m) => {
                let results: HashSet<usize> = m.keys().into_iter().cloned().collect();
                Some(results)
            }
            None => None,
        }
    }

    /// Returns the weights matrix as a nalgebra sparse matrix
    ///
    /// # Arguments
    ///
    /// * `transfrom` - what transform, if any to apply to the weights matrix as we  transform.
    /// Only TransformType::Row for row normalized is currently implemented.
    ///
    pub fn as_sparse_matrix(&self, transform: Option<TransformType>) -> CsrMatrix<f64> {
        let mut coo_matrix = CooMatrix::new(self.no_elements, self.no_elements);

        for (key, vals) in self.weights.iter() {
            let norm: f64 = match &transform {
                Some(TransformType::Row) => vals.values().sum(),
                _ => 1.0,
            };
            for (key2, weight) in vals.iter() {
                coo_matrix.push(*key, *key2, *weight / norm);
            }
        }

        CsrMatrix::from(&coo_matrix)
    }

    /// Returns the weights matrix in a list format
    ///
    /// Output format is a tuple of origin ids, dest ids, weight values
    ///
    pub fn to_list(&self) -> (Vec<usize>, Vec<usize>, Vec<f64>) {
        let mut origin_list: Vec<usize> = vec![];
        let mut dest_list: Vec<usize> = vec![];
        let mut weight_list: Vec<f64> = vec![];

        for (origin, dests) in self.weights.iter() {
            for (dest, weight) in dests.iter() {
                origin_list.push(*origin);
                dest_list.push(*dest);
                weight_list.push(*weight);
            }
        }
        (origin_list, dest_list, weight_list)
    }

    /// Returns the weights matrix in a list format with geometries
    ///
    /// Output format is a tuple of origin ids, dest ids, weight values, geometry linking origin
    /// and destination
    ///
    /// # Arguments
    ///
    /// * `geoms` - the list of geometries originally used to generate the weights matrix.
    pub fn to_list_with_geom<A: GeoFloat>(
        &self,
        geoms: &[Geometry<A>],
    ) -> Result<(Vec<usize>, Vec<usize>, Vec<f64>, Vec<Geometry<A>>), String> {
        let mut origin_list: Vec<usize> = vec![];
        let mut dest_list: Vec<usize> = vec![];
        let mut weight_list: Vec<f64> = vec![];
        let mut geoms: Vec<Geometry<A>> = vec![];
        let no_geoms = geoms.len();

        for (origin, dests) in self.weights.iter() {
            for (dest, weight) in dests.iter() {
                origin_list.push(*origin);
                dest_list.push(*dest);
                weight_list.push(*weight);
                let origin_centroid = geoms
                    .get(*origin)
                    .ok_or_else(|| format!("Failed to get origin {} {}", origin, no_geoms))?
                    .centroid()
                    .unwrap();
                let dest_centroid = geoms
                    .get(*dest)
                    .ok_or_else(|| format!("Failed to get origin {} {}", dest, no_geoms))?
                    .centroid()
                    .unwrap();
                let line: geo::Geometry<A> =
                    geo::Geometry::Line(Line::new(origin_centroid, dest_centroid));
                geoms.push(line);
            }
        }
        Ok((origin_list, dest_list, weight_list, geoms))
    }

    /// Returns the weights matrix in a GeoJson format with lines between the origin and
    /// destinations
    ///
    /// # Arguments
    ///
    /// * `geoms` - the list of geometries originally used to generate the weights matrix.
    pub fn links_geojson<A: GeoFloat>(&self, geoms: &[Geometry<A>]) -> FeatureCollection {
        let mut features: Vec<Feature> = vec![];

        for (origin, dests) in self.weights.iter() {
            for (dest, _weight) in dests.iter() {
                let origin_centroid = geoms.get(*origin).unwrap().centroid().unwrap();
                let dest_centroid = geoms.get(*dest).unwrap().centroid().unwrap();
                let line: geojson::Geometry =
                    geojson::Value::from(&Line::new(origin_centroid, dest_centroid)).into();

                let mut feature = Feature {
                    geometry: Some(line),
                    ..Default::default()
                };
                feature.set_property("origin", format!("{}", origin));
                feature.set_property("dest", format!("{}", dest));
                features.push(feature);
            }
        }
        FeatureCollection {
            features,
            bbox: None,
            foreign_members: None,
        }
    }

    // pub fn as_geopoalrs(&self, geoms: &[Geometry<A>], ids: &Vec<T>)->Result<DataFrame, Error>{
    //    use polars::io::{SerWriter, SerReader};
    //    use polars::prelude::NamedFromOwned;
    //    use geopolars::geoseries::GeoSeries;

    //    let mut origin_ids : Vec<i32> = Vec::with_capacity(geoms.len());
    //    let mut dests_ids: Vec<i32> = Vec::with_capacity(geoms.len());
    //    let mut weights: Vec<f32> = Vec::with_capacity(geoms.len());
    //    let mut lines: Vec<Line> = Vec::with_capacity(geoms.len());

    //    for (origin, dests) in self.weights().unwrap().iter(){
    //         for (dest, _weight) in dests.iter(){
    //             origin_ids.push(origin);
    //             dest_ids.push(dest);
    //             weights.push(weight);
    //             let origin_index =  ids.iter().position(|a| a == origin).unwrap();
    //             let dest_index =  ids.iter().position(|a| a == dest).unwrap();
    //             let origin_centroid = geoms.get(origin_index).unwrap().centroid().unwrap();
    //             let dest_centroid = geoms.get(dest_index).unwrap().centroid().unwrap();
    //             let line  = Line::new(origin_centroid, dest_centroid);
    //             geoms.push(line);
    //         }
    //    }
    //    let geom_col = Series::from_geom_vec(&lines);
    //    let result = DataFrame:::new([
    //         Series::from_vec("origin_id", origin_ids),
    //         Series::from_vec("dest_id", dests_ids),
    //         Series::from_vec("weight", weights),
    //         Series::from_vec("geom", geoms),
    //    ]);
    //    result
    // }
}
