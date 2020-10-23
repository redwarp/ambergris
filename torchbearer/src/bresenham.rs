use core::iter::Iterator;

use crate::Point;

/// Iterator-based Bresenham's line drawing algorithm.
///
/// Fork from https://github.com/mbr/bresenham-rs so that the iterator includes
/// `start` and `end`.
///
/// [Bresenham's line drawing algorithm](https://en.wikipedia.org/wiki/Bresenham%27s_line_algorithm)
/// is a fast algorithm to draw a line between two points. This implements the fast
/// integer variant, using an iterator-based appraoch for flexibility. It
/// calculates coordinates without knowing anything about drawing methods or
/// surfaces.
///
/// Example:
///
/// ```rust
/// use torchbearer::bresenham::LineBresenham;
///
/// for (x, y) in LineBresenham::new((0, 1), (6, 4)) {
///     println!("{}, {}", x, y);
/// }
/// ```
///
/// Will print:
///
/// ```text
/// (0, 1)
/// (1, 1)
/// (2, 2)
/// (3, 2)
/// (4, 3)
/// (5, 3)
/// (6, 4)
/// ```
pub struct LineBresenham {
    x: i32,
    y: i32,
    dx: i32,
    dy: i32,
    x1: i32,
    y1: i32,
    diff: i32,
    octant: Octant,
}

struct Octant(u8);

impl Octant {
    /// adapted from http://codereview.stackexchange.com/a/95551
    #[inline]
    fn from_points(start: Point, end: Point) -> Octant {
        let mut dx = end.0 - start.0;
        let mut dy = end.1 - start.1;

        let mut octant = 0;

        if dy < 0 {
            dx = -dx;
            dy = -dy;
            octant += 4;
        }

        if dx < 0 {
            let tmp = dx;
            dx = dy;
            dy = -tmp;
            octant += 2
        }

        if dx < dy {
            octant += 1
        }

        Octant(octant)
    }

    #[inline]
    fn to_octant0(&self, p: Point) -> Point {
        match self.0 {
            0 => (p.0, p.1),
            1 => (p.1, p.0),
            2 => (p.1, -p.0),
            3 => (-p.0, p.1),
            4 => (-p.0, -p.1),
            5 => (-p.1, -p.0),
            6 => (-p.1, p.0),
            7 => (p.0, -p.1),
            _ => unreachable!(),
        }
    }

    #[inline]
    fn from_octant0(&self, p: Point) -> Point {
        match self.0 {
            0 => (p.0, p.1),
            1 => (p.1, p.0),
            2 => (-p.1, p.0),
            3 => (-p.0, p.1),
            4 => (-p.0, -p.1),
            5 => (-p.1, -p.0),
            6 => (p.1, -p.0),
            7 => (p.0, -p.1),
            _ => unreachable!(),
        }
    }
}

impl LineBresenham {
    /// Creates a new iterator.Yields intermediate points between `start`
    /// and `end`, inclusive.
    #[inline]
    pub fn new(start: Point, end: Point) -> LineBresenham {
        let octant = Octant::from_points(start, end);

        let start = octant.to_octant0(start);
        let end = octant.to_octant0(end);

        let dx = end.0 - start.0;
        let dy = end.1 - start.1;

        LineBresenham {
            x: start.0,
            y: start.1,
            dx,
            dy,
            x1: end.0,
            y1: end.1,
            diff: dy - dx,
            octant,
        }
    }
}

impl ExactSizeIterator for LineBresenham {}

impl Iterator for LineBresenham {
    type Item = Point;

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        if self.x == self.x1 {
            self.x += 1;
            let p = (self.x1, self.y1);
            return Some(self.octant.from_octant0(p));
        }

        if self.x > self.x1 {
            return None;
        }

        let p = (self.x, self.y);

        if self.diff >= 0 {
            self.y += 1;
            self.diff -= self.dx;
        }

        self.diff += self.dy;

        // loop inc
        self.x += 1;

        Some(self.octant.from_octant0(p))
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        let len = (self.dx + 1) as usize;
        (len, Some(len))
    }
}

#[cfg(test)]
mod tests {
    use super::LineBresenham;
    use std::vec::Vec;

    #[test]
    fn test_wp_example() {
        let bi = LineBresenham::new((0, 1), (6, 4));
        let len = bi.len();
        let res: Vec<_> = bi.collect();

        assert_eq!(
            res,
            [(0, 1), (1, 1), (2, 2), (3, 2), (4, 3), (5, 3), (6, 4)]
        );
        assert_eq!(len, 7);
    }

    #[test]
    fn test_inverse_wp() {
        let bi = LineBresenham::new((6, 4), (0, 1));
        let len = bi.len();
        let res: Vec<_> = bi.collect();

        assert_eq!(
            res,
            [(6, 4), (5, 4), (4, 3), (3, 3), (2, 2), (1, 2), (0, 1)]
        );
        assert_eq!(len, 7);
    }

    #[test]
    fn test_straight_hline() {
        let bi = LineBresenham::new((2, 3), (5, 3));
        let len = bi.len();
        let res: Vec<_> = bi.collect();

        assert_eq!(res, [(2, 3), (3, 3), (4, 3), (5, 3)]);
        assert_eq!(len, 4);
    }

    #[test]
    fn test_straight_vline() {
        let bi = LineBresenham::new((2, 3), (2, 6));
        let len = bi.len();
        let res: Vec<_> = bi.collect();

        assert_eq!(res, [(2, 3), (2, 4), (2, 5), (2, 6)]);
        assert_eq!(len, 4);
    }
}
