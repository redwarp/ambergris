pub mod bresenham;
pub mod fov;
pub mod path;

pub type Point = (i32, i32);

pub trait Map {
    fn dimensions(&self) -> (i32, i32);
    fn is_opaque(&self, x: i32, y: i32) -> bool;
}
