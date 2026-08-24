pub mod capability;
pub mod journey;
pub mod page;
pub mod refs;
pub mod service;

pub trait Entity: Copy + Sized + 'static {
    const ALL: &'static [Self];
    fn as_str(&self) -> &'static str;
}
