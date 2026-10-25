//! Original hand-pixelled silhouette variants. Palette symbols are logical pixels.
pub const SEATED: &[&str] = &[
    "       oo             oo",
    "      occo           occo",
    "      obcco         occbo",
    "     oabcccooooooooocccbao",
    "    oabcccddddddddddccbbao",
    "   oabcccddffffffffddcbbao",
    "  oabcccfffffffffffffccbbao",
    "  obcccffffffffffffffccbbbo",
    " obcccffffffffffffffffccbbbo",
    " obccffffffffffffffffffcbbbo",
    " obccffffffffffffffffffcbbbo",
    " obccfffppffffffffppfffcbbbo",
    " obcccffffffffffffffffccbbbo",
    " oabccccffffffffffffccccbbao",
    "  oabccccccccccccccccccbbbao",
    "  oabbbcccccccccccccccbbbao",
    "   oabbbcccccccccccbbbbbao",
    "    oabbbbbccccccbbbbbao",
    "     oabbbbbbbbbbbbbao",
    "      oooooooooooooo",
];
pub const CURLED: &[&str] = &[
    "          oooo       oo",
    "      ooooccccoooooocco",
    "    oobcccddddddddcccbo",
    "   obcccddffffffffddcbbo",
    "  obcccfffffffffffffccbbo",
    " obcccffffffffffffffccbbbo",
    " obccffffffffffffffffcbbbo",
    "obcccfffppfffffffppffccbbbo",
    "obccccfffffffffffffccccbbbo",
    "oabcccccfffffffffccccccbbbo",
    " oabccccccccccccccccccbbbo",
    " oabbbcccccccccccccccbbbao",
    "  oabbbbbccccccccccbbbbbo",
    "   oabbbbbbbbbbbbbbbbbo",
    "    ooooooooooooooooo",
];
pub const STRETCHED: &[&str] = &[
    "        oo           oo",
    "       occo         occo",
    "       obcco       occbo",
    "      oabcccooooooocccbao",
    "     oabcccddddddddccbbao",
    "    oabcccddffffffddcbbao",
    "    obcccfffffffffffccbbbo",
    "   oabccfffffffffffffcbbbao",
    "   obccffffffffffffffcbbbo",
    "   obccffffffffffffffcbbbo",
    "   obccffffffffffffffcbbbo",
    "   obccffffffffffffffcbbbo",
    "   obccffffffffffffffcbbbo",
    "   obccfffppffffffppfcbbbo",
    "   oabccffffffffffffcbbbao",
    "    obccfffffffffffccbbbo",
    "    oabccccffffffccccbao",
    "     oabccccccccccccbbao",
    "     oabccccccccccccbbao",
    "      oabccccccccccbbao",
    "      oabbbccccccbbbao",
    "       oabbccccccbbbao",
    "       oabbbccccbbbao",
    "        oabbbbbbbbao",
    "         oabbbbbbao",
    "          ooooooo",
];

/// Canonical logical-pixel model. Every pose shares one canvas and ground origin.
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
#[derive(Clone, Copy)]
pub struct Model {
    pub art: &'static [&'static str],
    pub offset: Point,
    pub eyes: [Point; 2],
    pub mouth: Point,
    pub shoulders: [Point; 2],
    pub hips: [Point; 2],
}
pub fn model(shape: crate::motion::Shape, standing: &'static [&'static str]) -> Model {
    use crate::motion::Shape::*;
    let (art, offset, eye_y, mouth_y) = match shape {
        Stand => (standing, Point::new(3, 3), 14, 21),
        Crouch => (LOWERING, Point::new(3, 5), 15, 22),
        Seat => (SEATED, Point::new(3, 9), 16, 23),
        Curl => (CURLED, Point::new(4, 14), 20, 25),
        Stretch => (STRETCHED, Point::new(3, 2), 12, 19),
        Run => (RUNNING, Point::new(3, 3), 14, 21),
    };
    Model {
        art,
        offset,
        eyes: [
            Point::new(if shape == Run { 13 } else { 12 }, eye_y),
            Point::new(if shape == Run { 23 } else { 22 }, eye_y),
        ],
        mouth: Point::new(if shape == Run { 19 } else { 18 }, mouth_y),
        shoulders: [Point::new(8, eye_y + 5), Point::new(28, eye_y + 5)],
        hips: if shape == Stretch {
            [Point::new(14, 25), Point::new(21, 25)]
        } else {
            [Point::new(13, 27), Point::new(23, 27)]
        },
    }
}
/// Lowering pose: rounded torso, shortened lower body, never resampled.
pub const LOWERING: &[&str] = &[
    "       oo              oo",
    "      occo            occo",
    "      occco          occco",
    "      obcccoooooooooccccbo",
    "     oabccccddddddcccccbbo",
    "    oabcccddddddddddcccbbao",
    "   oabcccddddddddddddccbbao",
    "   obcccfffffffffffffccbbbo",
    "  oabccfffffffffffffffcbbbao",
    "  obccffffffffffffffffcbbbo",
    "  obccffffffffffffffffcbbbo",
    "  obccffffffffffffffffcbbbo",
    "  obccffffffffffffffffcbbbo",
    "  obccffffffffffffffffcbbbo",
    "  obccfffppffffffffppfcbbbo",
    "  oabccffffffffffffffcbbbao",
    "   obccfffffffffffffccbbbo",
    "   oabccccffffffffccccbao",
    "    oabcccccccccccccbbao",
    "     oabbbcccccccccbbbao",
    "      oabbbccccccbbbbao",
    "       oabbbccccbbbbao",
    "        oabbbbbbbbao",
    "         ooooooooo",
];
/// Forward head mass is authored into the outline; running never shears the face.
pub const RUNNING: &[&str] = &[
    "        oo              oo",
    "       occo            occo",
    "       occco          occco",
    "       obcccoooooooooccccbo",
    "      oabccccddddddcccccbbo",
    "     oabcccddddddddddcccbbao",
    "    oabcccddddddddddddccbbao",
    "    obcccfffffffffffffccbbbo",
    "   oabccfffffffffffffffcbbbao",
    "   obccffffffffffffffffcbbbo",
    "   obccffffffffffffffffcbbbo",
    "   obccffffffffffffffffcbbbo",
    "   obccffffffffffffffffcbbbo",
    "   obccffffffffffffffffcbbbo",
    "   obccffffffffffffffffcbbbo",
    "   obccffffffffffffffffcbbbo",
    "   obccfffppffffffffppfcbbbo",
    "  oabccffffffffffffffcbbbao",
    "   obccfffffffffffffccbbbo",
    "   oabccccffffffffccccbao",
    "    oabcccccccccccccbbao",
    "     oabbbcccccccccbbbao",
    "      oabbbccccccbbbbao",
    "       oabbbccccbbbbao",
    "        oabbbbbbbbao",
    "         ooooooooo",
];
