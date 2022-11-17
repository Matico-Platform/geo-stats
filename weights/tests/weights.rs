use geo_weights::Weights;
use std::collections::HashSet;
#[test]
fn we_should_correctly_construct_a_matrix_from_list_representation() {

    let origins: Vec<usize> = vec![
        1,
        1,
        3,
        4,
        5
    ];

    let dests: Vec<usize> = vec![
        2,
        3,
        4,
        2,
        2
    ];

    let weights: Vec<f64> = vec![
        1.0,
        2.0,
        -1.0,
        2.0,
        1.0
    ];

    let weights = Weights::from_list_rep(&origins, &dests, &weights, 5);

    let n0 = weights.get_neighbor_ids(0);
    let n1 = weights.get_neighbor_ids(1);
    let n2 = weights.get_neighbor_ids(2);
    let n3 = weights.get_neighbor_ids(3);
    let n4 = weights.get_neighbor_ids(4);
    let n5 = weights.get_neighbor_ids(5);

    assert!(n0.is_none());
    assert_eq!(n1, Some(HashSet::from([2,3])));
    assert_eq!(n2, Some(HashSet::from([5,1,4])));
    assert_eq!(n3, Some(HashSet::from([1,4])));
    assert_eq!(n3, Some(HashSet::from([1,4])));
    assert_eq!(n3, Some(HashSet::from([1,4])));
    assert_eq!(n4, Some(HashSet::from([3,2])));
    assert_eq!(n5, Some(HashSet::from([2])));
}
