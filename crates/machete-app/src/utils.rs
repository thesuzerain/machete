// TODO: This should not be in a utils category. Avoid if possible.
pub trait SelectableOption
where
    Self: std::marker::Sized + PartialEq,
{
    fn as_selectable_str(&self) -> &'static str;
    fn iter_options() -> Vec<Self>;
}
