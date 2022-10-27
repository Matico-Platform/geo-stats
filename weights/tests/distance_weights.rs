use geo_types::{Geometry, Point};
use geo_weights::{DistanceWeights, WeightBuilder};

#[test]
fn non_weighted_euclid_weight_should_include_points_under_the_threshold_and_not_above() {
    let weight_builder: DistanceWeights<f64> = DistanceWeights::new(Some(20.0), false);
    let points: Vec<Geometry<f64>> = vec![
        Point::new(1.0, 2.0).into(),
        Point::new(100.0, 0.0).into(),
        Point::new(2.0, 2.0).into(),
    ];

    let weights = weight_builder.compute_weights(&points);
    println!("weights are {}", weights);
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
fn weighted_euclid_weights_should_compute_correct_weight() {
    let weight_builder: DistanceWeights<f64> = DistanceWeights::new(Some(20.0), true);
    let points: Vec<Geometry<f64>> = vec![
        Point::new(1.0, 2.0).into(),
        Point::new(100.0, 0.0).into(),
        Point::new(2.0, 2.0).into(),
    ];

    let weights = weight_builder.compute_weights(&points);
}
