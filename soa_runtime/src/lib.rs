//! Traits and helpers used by the `#[derive(SoA)]` proc-macro.

pub trait SoaModel {
    type Soa;
    type View<'a>
    where
        Self: 'a;
    type ViewMut<'a>
    where
        Self: 'a;

    fn push_into(soa: &mut Self::Soa, v: Self);
    fn view(soa: &Self::Soa, i: usize) -> Self::View<'_>;
    fn view_mut(soa: &mut Self::Soa, i: usize) -> Self::ViewMut<'_>;
}

/// Simple cache-line padding wrapper to reduce false sharing between adjacent items.
#[repr(align(64))]
pub struct CachePadded<T>(pub T);
