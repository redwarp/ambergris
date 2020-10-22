use std::{
    cmp::Ordering,
    collections::{BinaryHeap, HashMap},
};

use crate::{Map, Position};

/// https://www.redblobgames.com/pathfinding/a-star/implementation.html#python-astar
/// Checking binary heap here: https://doc.rust-lang.org/stable/std/collections/binary_heap/
pub fn astar_path<T: Map>(map: &T, from: Position, to: Position) -> Option<Vec<Position>> {
    let mut frontier = BinaryHeap::new();

    frontier.push(State {
        position: from,
        cost: 0.,
    });

    let mut came_from: HashMap<Position, Option<Position>> = HashMap::new();
    let mut cost_so_far: HashMap<Position, f64> = HashMap::new();
    came_from.insert(from, None);
    cost_so_far.insert(from, 0.);

    while let Some(State {
        position: current,
        cost: _,
    }) = frontier.pop()
    {
        if current == to {
            break;
        }

        for next in neighboors(map, current) {
            let new_cost = cost_so_far[&current] + 1.; // or weighted cost;

            if !cost_so_far.contains_key(&next) || new_cost < cost_so_far[&next] {
                cost_so_far.insert(next, new_cost);
                let priority = new_cost + heuristic(next, to);
                frontier.push(State {
                    position: next,
                    cost: priority,
                });
                came_from.insert(next, Some(current));
            }
        }
    }

    reconstruct_path(from, to, came_from)
}

fn neighboors<T: Map>(map: &T, position: Position) -> Vec<Position> {
    let (width, height) = map.dimensions();
    let (x, y) = position;
    // This is a hack for nicer paths, as described here:
    // https://www.redblobgames.com/pathfinding/a-star/implementation.html#troubleshooting-ugly-path
    let neighboors = if (x + y) % 2 == 0 {
        vec![(x, y + 1), (x, y - 1), (x - 1, y), (x + 1, y)]
    } else {
        vec![(x + 1, y), (x - 1, y), (x, y - 1), (x, y + 1)]
    };

    neighboors
        .into_iter()
        .filter(|(x, y)| is_bounded(*x, *y, width, height))
        .filter(|(x, y)| !map.is_opaque(*x, *y))
        .collect()
}

fn reconstruct_path(
    start: Position,
    target: Position,
    mut came_from: HashMap<Position, Option<Position>>,
) -> Option<Vec<Position>> {
    let mut current = Some(target);
    let mut path = vec![];

    while current != Some(start) {
        if let Some(position) = current {
            path.push(position);

            current = came_from.remove(&position).unwrap_or(None);
        } else {
            return None;
        }
    }

    Some(path.into_iter().rev().collect())
}

fn is_bounded(x: i32, y: i32, width: i32, height: i32) -> bool {
    x >= 0 && y >= 0 && x < width && y < height
}

fn heuristic(a: Position, b: Position) -> f64 {
    let (xa, ya) = a;
    let (xb, yb) = b;

    ((xa - xb).abs() + (ya - yb).abs()) as f64
}

#[derive(Copy, Clone, PartialEq)]
struct State {
    position: Position,
    cost: f64,
}

impl Eq for State {}

// The priority queue depends on `Ord`.
// Explicitly implement the trait so the queue becomes a min-heap
// instead of a max-heap.
impl Ord for State {
    fn cmp(&self, other: &State) -> Ordering {
        // Notice that the we flip the ordering on costs.
        // In case of a tie we compare positions - this step is necessary
        // to make implementations of `PartialEq` and `Ord` consistent.
        other
            .cost
            .partial_cmp(&self.cost)
            .unwrap_or(Ordering::Equal)
            .then_with(|| self.position.cmp(&other.position))
    }
}

// `PartialOrd` needs to be implemented as well.
impl PartialOrd for State {
    fn partial_cmp(&self, other: &State) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

#[cfg(test)]
mod tests {
    use crate::{bresenham::Bresenham, Map, Position};

    use super::astar_path;

    struct TestMap {
        width: i32,
        height: i32,
        tiles: Vec<bool>,
    }

    impl TestMap {
        fn new(width: i32, height: i32) -> Self {
            TestMap {
                width,
                height,
                tiles: vec![false; (width * height) as usize],
            }
        }

        fn build_wall(&mut self, from: Position, to: Position) {
            let bresenham = Bresenham::new(from, to);
            for (x, y) in bresenham {
                self.tiles[(x + y * self.width) as usize] = true;
            }
        }
    }

    impl Map for TestMap {
        fn dimensions(&self) -> (i32, i32) {
            (self.width, self.height)
        }

        fn is_opaque(&self, x: i32, y: i32) -> bool {
            self.tiles[(x + y * self.width) as usize]
        }
    }

    #[test]
    fn astar() {
        let mut map = TestMap::new(10, 10);
        map.build_wall((3, 3), (3, 6));
        map.build_wall((0, 3), (3, 3));

        let path = astar_path(&map, (0, 4), (5, 4));
        println!("{:?}", path);
    }
}
