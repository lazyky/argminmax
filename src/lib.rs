mod simd;
mod task;
mod utils;
mod scalar_generic;
mod scalar_f16;

pub use scalar_generic::*;
pub use simd::{simd_f32, simd_f64, simd_i16, simd_i32, simd_i64};

use ndarray::ArrayView1;

pub trait ArgMinMax {
    // TODO: future work implement these other functions
    // fn min(self) -> Self::Item;
    // fn max(self) -> Self::Item;
    // fn minmax(self) -> (T, T);

    // fn argmin(self) -> usize;
    // fn argmax(self) -> usize;
    fn argminmax(self) -> (usize, usize);
}

macro_rules! impl_argminmax {
    ($t:ty, $scalar_func:ident, $simd_mod:ident, $simd_func:ident) => {
        impl ArgMinMax for ArrayView1<'_, $t> {
            fn argminmax(self) -> (usize, usize) {
                // TODO: what to do with cfg target_feature?
                #[cfg(not(target_feature = "sse"))]
                return $scalar_func(self);
                #[cfg(target_feature = "sse")]
                return $simd_mod::$simd_func(self);
            }
        }
    };
}

// Implement ArgMinMax for the rust primitive types
impl_argminmax!(f32, scalar_argminmax, simd_f32, argminmax_f32);
impl_argminmax!(f64, scalar_argminmax, simd_f64, argminmax_f64);
impl_argminmax!(i16, scalar_argminmax, simd_i16, argminmax_i16);
impl_argminmax!(i32, scalar_argminmax, simd_i32, argminmax_i32);
impl_argminmax!(i64, scalar_argminmax, simd_i64, argminmax_i64);
// Implement ArgMinMax for other data types
#[cfg(feature = "half")]
use half::f16;
#[cfg(feature = "half")]
pub use scalar_f16::scalar_argminmax_f16;
#[cfg(feature = "half")]
pub use simd::simd_f16;
#[cfg(feature = "half")]
impl_argminmax!(f16, scalar_argminmax_f16, simd_f16, argminmax_f16);
