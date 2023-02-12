#[cfg(feature = "half")]
use half::f16;

#[cfg(feature = "half")]
#[inline(always)]
fn f16_to_i16ord(x: f16) -> i16 {
    let x = unsafe { std::mem::transmute::<f16, i16>(x) };
    ((x >> 15) & 0x7FFF) ^ x
}

// TODO: commented this (see the TODO below)
// #[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
#[cfg(feature = "half")]
// #[inline(never)]
pub(crate) fn scalar_argminmax_f16(arr: &[f16]) -> (usize, usize) {
    // f16 is transformed to i16ord
    //   benchmarks  show:
    //     1. this is 7-10x faster than using raw f16
    //     2. this is 3x faster than transforming to f32 or f64
    assert!(!arr.is_empty());
    let mut low_index: usize = 0;
    let mut high_index: usize = 0;
    // It is remarkably faster to iterate over the index and use get_unchecked
    // than using .iter().enumerate() (with a fold).
    let mut low: i16 = f16_to_i16ord(unsafe { *arr.get_unchecked(low_index) });
    let mut high: i16 = f16_to_i16ord(unsafe { *arr.get_unchecked(high_index) });
    for i in 0..arr.len() {
        let v: f16 = unsafe { *arr.get_unchecked(i) };
        if v.is_nan() {
            // Return the index of the first NaN value
            return (i, i);
        }
        let v: i16 = f16_to_i16ord(v);
        if v < low {
            low = v;
            low_index = i;
        } else if v > high {
            high = v;
            high_index = i;
        }
    }
    (low_index, high_index)
}

// TODO: previously we had dedicated non x86_64 code for f16 (see below)

// #[cfg(not(any(target_arch = "x86", target_arch = "x86_64")))]
// #[cfg(feature = "half")]
// // #[inline(never)]
// pub(crate) fn scalar_argminmax_f16(arr: &[f16]) -> (usize, usize) {
//     // f16 is transformed to i16ord
//     //   benchmarks  show:
//     //     1. this is 7-10x faster than using raw f16
//     //     2. this is 3x faster than transforming to f32 or f64
//     assert!(!arr.is_empty());
//     // This is 3% slower on x86_64, but 12% faster on aarch64.
//     let minmax_tuple: (usize, i16, usize, i16) = arr.iter().enumerate().fold(
//         (0, f16_to_i16ord(arr[0]), 0, f16_to_i16ord(arr[0])),
//         |(low_index, low, high_index, high), (i, item)| {
//             if item.is_nan() {
//                 // Return the index of the first NaN value
//                 return (i, i);
//             }
//             let item = f16_to_i16ord(*item);
//             if item < low {
//                 (i, item, high_index, high)
//             } else if item > high {
//                 (low_index, low, i, item)
//             } else {
//                 (low_index, low, high_index, high)
//             }
//         },
//     );
//     (minmax_tuple.0, minmax_tuple.2)
// }

#[cfg(feature = "half")]
#[cfg(test)]
mod tests {
    use super::scalar_argminmax_f16;
    use crate::scalar::generic::scalar_argminmax;

    use half::f16;

    extern crate dev_utils;
    use dev_utils::utils;

    fn get_array_f16(len: usize) -> Vec<f16> {
        let v = utils::get_random_array(len, i16::MIN, i16::MAX);
        v.iter().map(|x| f16::from_f32(*x as f32)).collect()
    }

    #[test]
    fn test_generic_and_specific_impl_return_the_same_results() {
        for _ in 0..100 {
            let data: &[f16] = &get_array_f16(1025);
            let (argmin_index, argmax_index) = scalar_argminmax(data);
            let (argmin_index_f16, argmax_index_f16) = scalar_argminmax_f16(data);
            assert_eq!(argmin_index, argmin_index_f16);
            assert_eq!(argmax_index, argmax_index_f16);
        }
    }

    #[test]
    fn test_generic_and_specific_impl_return_nans() {
        let arr_len: usize = 1025;

        // firts, middle, last element
        let nan_pos: [usize; 3] = [0, arr_len / 2, arr_len - 1];
        for pos in nan_pos.iter() {
            let mut data: Vec<f16> = get_array_f16(arr_len);
            data[*pos] = f16::NAN;
            let (argmin_index, argmax_index) = scalar_argminmax(&data);
            let (argmin_index_f16, argmax_index_f16) = scalar_argminmax_f16(&data);
            assert_eq!(argmin_index, argmin_index_f16);
            assert_eq!(argmax_index, argmax_index_f16);
            assert_eq!(argmin_index, *pos);
            assert_eq!(argmax_index, *pos);
        }

        // All elements are NaN
        let mut data: Vec<f16> = get_array_f16(arr_len);
        for i in 0..arr_len {
            data[i] = f16::NAN;
        }
        let (argmin_index, argmax_index) = scalar_argminmax(&data);
        let (argmin_index_f16, argmax_index_f16) = scalar_argminmax_f16(&data);
        assert_eq!(argmin_index, argmin_index_f16);
        assert_eq!(argmax_index, argmax_index_f16);
        assert_eq!(argmin_index, 0);
        assert_eq!(argmax_index, 0);
    }
}
