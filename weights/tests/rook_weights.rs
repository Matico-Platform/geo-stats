use geo_types::{polygon, Geometry};
use geo_weights::{RookWeights, WeightBuilder};
#[test]
fn we_should_get_the_correct_weights() {
    let weight_builder = RookWeights::new(10000.0);
    let points: Vec<Geometry<f64>> = vec![
        polygon![
            (x: 1.0, y:1.0),
            (x: 2.0, y:1.0),
            (x: 2.0, y:2.0),
            (x: 1.0, y:2.0)
        ]
        .into(),
        polygon![
            (x: 0.0, y:0.0),
            (x: 1.0, y:0.0),
            (x: 1.0, y:1.0),
            (x: 0.0, y:1.0)
        ]
        .into(),
        polygon![
            (x: 10.0, y:10.0),
            (x: 20.0, y:10.0),
            (x: 20.0, y:20.0),
            (x: 10.0, y:20.0)
        ]
        .into(),
        polygon![
            (x: 0.0, y:1.0),
            (x: 1.0, y:1.0),
            (x: 1.0, y:2.0),
            (x :0.0, y:2.0)
        ]
        .into(),
    ];

    let weights = weight_builder.compute_weights(&points);
    let n1 = weights.get_neighbor_ids(0).unwrap();
    let n2 = weights.get_neighbor_ids(1).unwrap();
    let n3 = weights.get_neighbor_ids(2).unwrap();
    let n4 = weights.get_neighbor_ids(3).unwrap();

    assert!(n1.is_empty());
    assert!(n2.is_empty());
    assert!(n3.is_empty());
    assert!(n4.is_empty());
}
