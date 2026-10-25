//! Complete original floating-character frames. Dots are transparent; no feet/legs.
//! Each embedded 36×36 text frame includes arms, silhouette and shading.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Point {
    pub x: i32,
    pub y: i32,
}
impl Point {
    pub const fn new(x: i32, y: i32) -> Self {
        Self { x, y }
    }
}
pub const WIDTH: usize = 36;
pub const HEIGHT: usize = 36;
pub const ORIGIN: Point = Point::new(18, 31);
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(usize)]
pub enum Shape {
    Relax,
    Droop,
    Peek,
    Rise,
    Reach,
    Hold,
    Stretch,
    Sleep,
}
impl Shape {
    pub const ALL: [Self; 8] = [
        Self::Relax,
        Self::Droop,
        Self::Peek,
        Self::Rise,
        Self::Reach,
        Self::Hold,
        Self::Stretch,
        Self::Sleep,
    ];
}
#[derive(Clone, Copy)]
pub struct Model {
    pub pixels: &'static str,
    pub foreground: Option<&'static str>,
    pub origin: Point,
    pub eyes: [Point; 2],
    pub mouth: Point,
    pub arms: [Point; 2],
    pub orb: Point,
}
pub fn model(shape: Shape) -> Model {
    use Shape::*;
    let (pixels, arms) = match shape {
        Relax => (
            include_str!("sprites/relax.sprite"),
            [Point::new(6, 19), Point::new(29, 19)],
        ),
        Droop => (
            include_str!("sprites/droop.sprite"),
            [Point::new(6, 22), Point::new(29, 22)],
        ),
        Peek => (
            include_str!("sprites/peek.sprite"),
            [Point::new(6, 19), Point::new(29, 15)],
        ),
        Rise => (
            include_str!("sprites/rise.sprite"),
            [Point::new(5, 12), Point::new(29, 12)],
        ),
        Reach => (
            include_str!("sprites/reach.sprite"),
            [Point::new(6, 19), Point::new(29, 18)],
        ),
        Hold => (
            include_str!("sprites/hold.sprite"),
            [Point::new(9, 21), Point::new(27, 22)],
        ),
        Stretch => (
            include_str!("sprites/stretch.sprite"),
            [Point::new(5, 11), Point::new(29, 11)],
        ),
        Sleep => (
            include_str!("sprites/sleep.sprite"),
            [Point::new(8, 21), Point::new(27, 21)],
        ),
    };
    Model {
        pixels,
        foreground: if shape == Hold {
            Some(include_str!("sprites/hold-front.sprite"))
        } else {
            None
        },
        origin: ORIGIN,
        eyes: [Point::new(12, 14), Point::new(22, 14)],
        mouth: Point::new(18, 21),
        arms,
        orb: Point::new(27, 20),
    }
}
