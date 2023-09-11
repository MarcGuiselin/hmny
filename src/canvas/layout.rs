#[derive(Clone)]
pub enum Layout {
    FlexBasic(FlexBasic),
}

impl Default for Layout {
    fn default() -> Self {
        Self::FlexBasic(FlexBasic::default())
    }
}

#[derive(Clone, Default)]
pub struct FlexBasic {
    pub direction: Direction,
    pub gap: f32,
}

#[derive(Clone, Default)]
pub enum Direction {
    #[default]
    Horizontal,
    Vertical,
}
