//! Collection of utility functions to find path.

use std::{cmp::Ordering, collections::BinaryHeap};

use crate::{Map, Point};

/// An A* pathfinding implementation for a grid base map, where diagonal movements are disabled.
/// Returns an optional vector containing the several points on the map to walk through, including the origin and destination.
///
/// Implements the algorithm and fixes found on the
/// [redblobgames.com](https://www.redblobgames.com/pathfinding/a-star/implementation.html#python-astar).
///
/// Uses a binary heap as described in the [rust-lang](https://doc.rust-lang.org/stable/std/collections/binary_heap/) doc.
///
/// # Arguments
///
/// * `map` - a struct implementing the `Map` trait.
/// * `from` - the origin.
/// * `to` - the destination.
///
/// # Examples
/// ```
/// use torchbearer::{Map, Point};
/// use torchbearer::path::astar_path;
///
/// struct SampleMap {
///     width: i32,
///     height: i32,
///     walkable: Vec<bool>,
/// }
///
/// impl SampleMap {
///     fn new(width: i32, height: i32) -> Self {
///          // (…)
/// #        SampleMap {
/// #            width,
/// #            height,
/// #            walkable: vec![true; (width * height) as usize],
/// #        }
///    }
/// }
///
/// impl Map for SampleMap {
///     fn dimensions(&self) -> (i32, i32) {
///         (self.width, self.height)
///     }
///
///     fn is_transparent(&self, x: i32, y: i32) -> bool {
///         // pathfinding only considers walkability.
///         unreachable!("Not used in pathfinding.")
///     }
///
///     fn is_walkable(&self, x: i32, y: i32) -> bool {
///         self.walkable[(x + y * self.width) as usize]
///     }
/// }
///
/// let sample_map = SampleMap::new(16, 10);
///
/// // (…) You probably want at this point to add some walls to your map.
///
/// let path = astar_path(&sample_map, (1,1), (3,8));
///
/// if let Some(path) = astar_path(&sample_map, (1,1), (3,8)) {
///     // (…)
/// }
/// ```
pub fn astar_path<T: Map>(map: &T, from: Point, to: Point) -> Option<Vec<Point>> {
    let (width, height) = map.dimensions();
    let capacity = rough_capacity(&from, &to);
    let mut frontier = BinaryHeap::with_capacity(capacity);

    let from_index = point_to_index(from, width);
    let to_index = point_to_index(to, width);

    frontier.push(State {
        cost: 0.,
        item: from_index,
    });

    let mut came_from: Vec<Option<usize>> = vec![None; (width * height) as usize];
    let mut costs: Vec<Option<f32>> = vec![None; (width * height) as usize];
    costs[from_index] = Some(0.);

    let mut to_cost = 0.;

    while let Some(State {
        item: current_index,
        cost: current_cost,
    }) = frontier.pop()
    {
        if current_index == to_index {
            to_cost = current_cost;
            break;
        }

        let (x, y) = index_to_point(current_index, width);
        let current = (x, y);
        for &(x, y) in &[(x, y + 1), (x, y - 1), (x - 1, y), (x + 1, y)] {
            if x < 0 || y < 0 || x >= width || y >= height || !map.is_walkable(x, y) {
                continue;
            }
            let next = (x, y);
            let next_index = point_to_index(next, width);

            let cost_so_far = costs[current_index].unwrap();
            let new_cost = cost_so_far + cost(&current, &next);

            if costs[next_index].is_none() || new_cost < costs[next_index].unwrap() {
                let priority = new_cost + heuristic(&next, &to);
                frontier.push(State {
                    cost: priority,
                    item: next_index,
                });
                came_from[next_index] = Some(current_index);
                costs[next_index] = Some(new_cost);
            }
        }
    }

    reconstruct_path(from, to, came_from, to_cost, width)
}

fn cost(from: &Point, to: &Point) -> f32 {
    let basic = 1.;
    let &(x1, y1) = from;
    let &(x2, y2) = to;
    let nudge = if ((x1 + y1) % 2 == 0 && x2 != x1) || ((x1 + y1) % 2 == 1 && y2 != y1) {
        1.
    } else {
        0.
    };
    basic + 0.001 * nudge
}

fn reconstruct_path(
    from: Point,
    to: Point,
    came_from: Vec<Option<usize>>,
    cost: f32,
    width: i32,
) -> Option<Vec<Point>> {
    let mut current = Some(point_to_index(to, width));
    let target_index = point_to_index(from, width);

    let mut path = Vec::with_capacity((cost.floor() + 2.0) as usize);

    while current != Some(target_index) {
        if let Some(position) = current {
            path.push(position);
            current = if let Some(entry) = came_from[position] {
                Some(entry)
            } else {
                return None;
            }
        } else {
            return None;
        }
    }
    path.push(target_index);

    Some(
        path.into_iter()
            .map(|index| index_to_point(index, width))
            .rev()
            .collect(),
    )
}

fn heuristic(a: &Point, b: &Point) -> f32 {
    let (xa, ya) = a;
    let (xb, yb) = b;

    ((xa - xb).abs() + (ya - yb).abs()) as f32
}

fn point_to_index((x, y): Point, width: i32) -> usize {
    (x + y * width) as usize
}

fn index_to_point(index: usize, width: i32) -> Point {
    (index as i32 % width, index as i32 / width)
}

/// Estimate a basic capacity. Chances are there will still be re-allocation, but we at last prevent
/// a few useless one.
fn rough_capacity(a: &Point, b: &Point) -> usize {
    let &(xa, ya) = a;
    let &(xb, yb) = b;
    let distance = (xa - xb).abs().max((ya - yb).abs()) as usize;
    distance * distance
}

struct State<C: PartialOrd, T> {
    cost: C,
    item: T,
}
impl<C: PartialOrd, T> PartialEq for State<C, T> {
    fn eq(&self, other: &Self) -> bool {
        self.cost.eq(&other.cost)
    }
}

impl<C: PartialOrd, T> Eq for State<C, T> {}

// The priority queue depends on `Ord`.
// Explicitly implement the trait so the queue becomes a min-heap
// instead of a max-heap.
impl<C: PartialOrd, T> Ord for State<C, T> {
    fn cmp(&self, other: &State<C, T>) -> Ordering {
        // Notice that the we flip the ordering on costs.
        // In case of a tie we compare positions - this step is necessary
        // to make implementations of `PartialEq` and `Ord` consistent.
        other
            .cost
            .partial_cmp(&self.cost)
            .unwrap_or(Ordering::Equal)
    }
}

// `PartialOrd` needs to be implemented as well.
impl<C: PartialOrd, T> PartialOrd for State<C, T> {
    fn partial_cmp(&self, other: &State<C, T>) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

#[cfg(test)]
mod tests {
    use crate::{bresenham::BresenhamLine, Map, Point};

    use super::astar_path;

    struct SampleMap {
        width: i32,
        height: i32,
        walkable: Vec<bool>,
    }

    impl SampleMap {
        fn new(width: i32, height: i32) -> Self {
            SampleMap {
                width,
                height,
                walkable: vec![true; (width * height) as usize],
            }
        }

        fn build_wall(&mut self, from: Point, to: Point) {
            let bresenham = BresenhamLine::new(from, to);
            for (x, y) in bresenham {
                self.walkable[(x + y * self.width) as usize] = false;
            }
        }
    }

    impl Map for SampleMap {
        fn dimensions(&self) -> (i32, i32) {
            (self.width, self.height)
        }

        fn is_transparent(&self, _x: i32, _y: i32) -> bool {
            todo!("Not needed for pathfinding.");
        }

        fn is_walkable(&self, x: i32, y: i32) -> bool {
            self.walkable[(x + y * self.width) as usize]
        }
    }

    #[test]
    fn astar_find_path() {
        let mut map = SampleMap::new(10, 10);
        map.build_wall((3, 3), (3, 6));
        map.build_wall((0, 3), (3, 3));

        let from = (0, 4);
        let to = (5, 4);

        let path = astar_path(&map, from, to);
        assert!(path.is_some());

        if let Some(path) = path {
            assert_eq!(from, path[0]);
            assert_eq!(to, path[path.len() - 1]);

            assert_eq!(
                path,
                [
                    (0, 4),
                    (0, 5),
                    (1, 5),
                    (1, 6),
                    (2, 6),
                    (2, 7),
                    (3, 7),
                    (4, 7),
                    (5, 7),
                    (5, 6),
                    (5, 5),
                    (5, 4)
                ]
            );
        }
    }

    #[test]
    fn astar_no_path() {
        let mut map = SampleMap::new(10, 10);
        map.build_wall((3, 3), (3, 6));
        map.build_wall((0, 3), (3, 3));
        map.build_wall((0, 6), (3, 6));

        let from = (0, 4);
        let to = (5, 4);

        let path = astar_path(&map, from, to);
        assert!(path.is_none());
    }
}
