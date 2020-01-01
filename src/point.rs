pub trait Point: Sync {
    fn latitude(&self) -> f32;
    fn longitude(&self) -> f32;
    fn altitude(&self) -> i16;
}
