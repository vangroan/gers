use std::fmt::{self, Debug, Display};

use num_traits::Num;

// TODO: For bin packing we need to rotate the
//       bins. This rectangle solution may be
//       insufficient, and we may have to change
//       to UV offsets.
//       Examples:
//         Offset { offset: [0, 0], size: [1, 1] }
//         Offset { offset: [0, 1], size: [1, -1] } // rotated

/// General purpose 2D rectangle.
///
/// Contains a position and size.
#[derive(Debug, Clone, Copy)]
pub struct Rect<N: Debug + Copy> {
    pub pos: [N; 2],
    pub size: [N; 2],
}

impl<N> Display for Rect<N>
where
    N: Display + Debug + Copy,
{
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "[{}, {}, {}, {}]",
            self.pos[0], self.pos[1], self.size[0], self.size[1]
        )
    }
}

impl<N> Rect<N>
where
    N: PartialOrd + Debug + Copy + Num,
{
    pub fn new(pos: [N; 2], size: [N; 2]) -> Self {
        Self { pos, size }
    }

    /// Calculate the bounding box of the rectangle, as the minimum and maximum corners.
    pub fn as_bounds(&self) -> [N; 4] {
        [
            self.pos[0],
            self.pos[1],
            self.pos[0].add(self.size[0]),
            self.pos[1].add(self.size[1]),
        ]
    }

    /// Checks whether `other` can fit inside this rectangle.
    pub fn can_fit(&self, other: &Rect<N>) -> bool {
        let [ax1, ay1, ax2, ay2] = self.as_bounds();
        let [bx1, by1, bx2, by2] = other.as_bounds();

        // top-left corner
        ax1 <= bx1 && ay1 <= by1
        // bottom-right corner
        && ax2 >= bx2 && ay2 >= by2

        // other.pos[0] >= self.pos[0]
        //     && other.pos[1] >= self.pos[1]
        //     && other.size[0] <= self.size[0]
        //     && other.size[1] <= self.size[1]
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_rect_fit() {
        let cases: &[(Rect<f32>, Rect<f32>, bool)] = &[
            (
                Rect::new([0.0, 0.0], [10.0, 10.0]),
                Rect::new([2.0, 2.0], [5.0, 5.0]),
                true,
            ),
            (
                Rect::new([0.0, 0.0], [10.0, 10.0]),
                Rect::new([2.0, 2.0], [10.0, 10.0]),
                false,
            ),
            (
                Rect::new([0.0, 0.0], [10.0, 10.0]),
                Rect::new([-15.0, -15.0], [10.0, 10.0]),
                false,
            ),
        ];

        for (outer, inner, expect) in cases {
            println!("case: can {} fit inside {}", inner, outer);
            assert_eq!(outer.can_fit(inner), *expect);
        }
    }
}
