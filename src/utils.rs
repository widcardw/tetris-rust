#[derive(Debug, PartialEq, Copy, Clone)]
pub enum Color {
    Red,
    Orange,
    Yellow,
    Green,
    Blue,
    Purple,
    Brown,
}

pub fn color_to_str(color: &Color) -> Option<String> {
    match *color {
        Color::Red => Some("🟥".to_owned()),
        Color::Orange => Some("🟧".to_owned()),
        Color::Yellow => Some("🟨".to_owned()),
        Color::Green => Some("🟩".to_owned()),
        Color::Blue => Some("🟦".to_owned()),
        Color::Purple => Some("🟪".to_owned()),
        Color::Brown => Some("🟫".to_owned()),
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Point {
    pub x: i32,
    pub y: i32,
}
