use torchbearer::{fov::field_of_view, Map, Point};

pub fn field_of_view_no_walls<T: Map>(
    map: &T,
    from: Point,
    radius: i32,
) -> std::vec::Vec<(i32, i32)> {
    field_of_view(map, from, radius)
        .into_iter()
        .filter(|&(x, y)| map.is_transparent((x, y)))
        .collect()
}
