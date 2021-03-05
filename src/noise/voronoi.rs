use crate::collections::F64Array;
use rust_wren::prelude::*;
use std::collections::BTreeSet;

#[wren_class]
pub struct Voronoi2D {
    /// Doubly connected edge list.
    dcel: voronoi::DCEL,
}

#[wren_methods]
impl Voronoi2D {
    #[construct]
    fn new(points: &WrenCell<F64Array>, boxsize: f64) -> Self {
        // Unfortunately we have to copy the points to a buffer
        // in order to call voronoi, then internally the library
        // copies the points to another buffer.
        let mut points = points
            .borrow()
            .as_slice()
            .chunks(2)
            .filter_map(|chunk| {
                if let [Some(x), Some(y)] = [chunk.get(0), chunk.get(1)] {
                    Some(voronoi::Point::new(*x, *y))
                } else {
                    None
                }
            })
            .collect::<Vec<_>>();

        Self::shift_conflicts(&mut points);

        Voronoi2D {
            dcel: voronoi::voronoi(points, boxsize),
        }
    }

    #[method(name = makePolygons)]
    pub fn make_polygons(&self) {
        let _polygons = voronoi::make_polygons(&self.dcel);

        todo!()
    }
}

impl Voronoi2D {
    fn shift_conflicts(points: &mut [voronoi::Point]) {
        let mut set = BTreeSet::<voronoi::Point>::new();

        for point in points {
            while set.contains(&point) {
                // Point coordinate conflict.
                *point = voronoi::Point::new(point.x(), point.y() + 1.0);
            }

            set.insert(*point);
        }
    }
}
