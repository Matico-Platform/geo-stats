use geo::{Coordinate, GeoFloat};

pub fn coords_to_tolerance<T>(coords: Coordinate<T>, tolerance: f64) -> (isize, isize)
where
    T: GeoFloat,
{
    (
        (coords.x.to_f64().unwrap() * tolerance).floor() as isize,
        (coords.y.to_f64().unwrap() * tolerance).floor() as isize,
    )
}

#[cfg(test)]
mod test {
    use geo::Coordinate;

    use super::coords_to_tolerance;

    #[test]
    fn coords_to_tolerance_should_work() {
        let transformed = coords_to_tolerance(Coordinate::<f64> { x: 100.0, y: 200.0 }, 10000.0);
        assert_eq!(
            transformed,
            (1000000, 2000000),
            "Should get correct transformed object for hash"
        );
    }
    #[test]
    fn coords_to_tolerance_should_work_when_fractional_negative() {
        let transformed = coords_to_tolerance(
            Coordinate::<f64> {
                x: 74.34234,
                y: -20.23423,
            },
            10000.0,
        );
        assert_eq!(
            transformed,
            (743423, -202343),
            "Should get correct transformed object for hash"
        );
    }
}
