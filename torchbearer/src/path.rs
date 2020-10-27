//! Collection of utility functions to find path.

use std::{
    cmp::Ordering,
    collections::{BinaryHeap, HashMap},
};

use crate::{Map, Point};

/// An astar pathfinding implementation for a grid base map, where diagonal movements are disabled.
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

    frontier.push(State {
        cost: 0.,
        item: from,
    });

    let mut origin_and_cost_so_far: HashMap<Point, (Option<Point>, f32)> =
        HashMap::with_capacity(capacity);
    origin_and_cost_so_far.insert(from, (None, 0.));

    let mut to_cost = 0.;

    while let Some(State {
        item: current,
        cost: current_cost,
    }) = frontier.pop()
    {
        if current == to {
            to_cost = current_cost;
            break;
        }

        for next in neighboors(map, &current, width, height).into_iter() {
            let cost_so_far = origin_and_cost_so_far[&current].1;
            // let (_came_from, cost_so_far) = origin_and_cost_so_far[&current];
            let new_cost = cost_so_far + cost(&current, &next);

            if !origin_and_cost_so_far.contains_key(&next)
                || new_cost < origin_and_cost_so_far[&next].1
            {
                let priority = new_cost + heuristic(&next, &to);
                frontier.push(State {
                    cost: priority,
                    item: next,
                });
                origin_and_cost_so_far.insert(next, (Some(current), new_cost));
            }
        }
    }

    reconstruct_path(from, to, origin_and_cost_so_far, to_cost)
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

fn neighboors<T: Map>(map: &T, position: &Point, width: i32, height: i32) -> Vec<Point> {
    let &(x, y) = position;
    // This is a hack for nicer paths, as described here:
    // https://www.redblobgames.com/pathfinding/a-star/implementation.html#troubleshooting-ugly-path
    let candidate_neighboors = if (x + y) % 2 == 0 {
        [(x, y + 1), (x, y - 1), (x - 1, y), (x + 1, y)]
    } else {
        [(x + 1, y), (x - 1, y), (x, y - 1), (x, y + 1)]
    };

    let mut neighboors = Vec::with_capacity(4);
    for &(x, y) in candidate_neighboors.iter() {
        if is_bounded(x, y, width, height) && map.is_walkable(x, y) {
            neighboors.push((x, y));
        }
    }
    neighboors
}

fn reconstruct_path(
    from: Point,
    to: Point,
    mut came_from: HashMap<Point, (Option<Point>, f32)>,
    cost: f32,
) -> Option<Vec<Point>> {
    let mut current = Some(to);

    let mut path = Vec::with_capacity((cost.floor() + 2.0) as usize);

    while current != Some(from) {
        if let Some(position) = current {
            path.push(position);
            current = if let Some(entry) = came_from.remove(&position) {
                entry.0
            } else {
                return None;
            }
        } else {
            return None;
        }
    }
    path.push(from);

    Some(path.into_iter().rev().collect())
}

fn is_bounded(x: i32, y: i32, width: i32, height: i32) -> bool {
    x >= 0 && y >= 0 && x < width && y < height
}

fn heuristic(a: &Point, b: &Point) -> f32 {
    let (xa, ya) = a;
    let (xb, yb) = b;

    ((xa - xb).abs() + (ya - yb).abs()) as f32
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
