mod test_data;

use geo_types::{polygon, Geometry};
use geo_weights::{QueensWeights, WeightBuilder};
use test_data::tracts;

#[test]
fn queens_real_world_test() {
    let tracts = tracts();
    let weight_builder = QueensWeights::new(100000.0);

    let weights = weight_builder.compute_weights(&tracts);

    let _weights_for_1 = weights.get_neighbor_ids(1).unwrap();
    let _weights_for_2 = weights.get_neighbor_ids(2).unwrap();
    let _weights_for_3 = weights.get_neighbor_ids(3).unwrap();
    let _weights_for_10 = weights.get_neighbor_ids(10).unwrap();
}

#[test]
fn queens_we_should_get_the_correct_weights() {
    let weight_builder = QueensWeights::new(10000.0);
    tracts();
    let points: Vec<Geometry<f64>> = vec![
        polygon![
            (x: 1.0, y:1.0),
            (x: 2.0, y:1.0),
            (x:2.0, y:2.0),
            (x:1.0, y:2.0)
        ]
        .into(),
        polygon![
            (x: 0.0, y:0.0),
            (x: 1.0, y:0.0),
            (x:1.0, y:1.0),
            (x:0.0, y:1.0)
        ]
        .into(),
        polygon![
            (x: 10.0, y:10.0),
            (x: 20.0, y:10.0),
            (x:20.0, y:20.0),
            (x:10.0, y:20.0)
        ]
        .into(),
        polygon![
            (x: 0.0, y:1.0),
            (x: 1.0, y:1.0),
            (x:1.0, y:2.0),
            (x:0.0, y:2.0)
        ]
        .into(),
    ];


    let weights = weight_builder.compute_weights(&points);
    let n1 = weights.get_neighbor_ids(0).unwrap();
    let n2 = weights.get_neighbor_ids(1).unwrap();
    let n3 = weights.get_neighbor_ids(2).unwrap();
    let n4 = weights.get_neighbor_ids(3).unwrap();

    assert!(n1.contains(&1));
    assert!(n1.contains(&3));
    assert!(n2.contains(&3));
    assert!(n3.is_empty());
    assert!(n4.contains(&1));
    assert!(n4.contains(&0));
}
