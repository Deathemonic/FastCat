#![no_std]
#![warn(clippy::pedantic, clippy::nursery)]
#![deny(clippy::unwrap_used, clippy::expect_used)]

extern crate alloc;

use core::mem::MaybeUninit;
use core::ptr;

pub use fastcat_macro::fconcat;

#[doc(hidden)]
pub use core;

/// Concatenates `slices` into a `[MaybeUninit<u8>; LEN]`.
///
/// Panics (at compile time, only ever called from `const` position) if the
/// combined length of `slices` does not equal `LEN`.
#[doc(hidden)]
#[must_use]
pub const fn concat_bytes<const LEN: usize>(slices: &[&[u8]]) -> [MaybeUninit<u8>; LEN] {
    let mut arr: [MaybeUninit<u8>; LEN] = [MaybeUninit::uninit(); LEN];
    let mut base = 0;
    let mut i = 0;
    while i < slices.len() {
        let slice = slices[i];

        assert!(base + slice.len() <= LEN, "invalid length");

        // SAFETY: just checked base + slice.len() <= LEN, so the destination
        // range is in-bounds. Source and destination don't overlap since
        // `slice` and `arr` are always disjoint allocations.
        unsafe {
            ptr::copy_nonoverlapping(
                slice.as_ptr(),
                arr.as_mut_ptr().add(base).cast::<u8>(),
                slice.len(),
            );
        }

        base += slice.len();
        i += 1;
    }
    assert!(base == LEN, "invalid length");
    arr
}

#[cfg(test)]
mod tests {
    use super::fconcat;

    #[test]
    fn all_dynamic() {
        let a = "hello ";
        let b = "world";
        assert_eq!(fconcat!(a, b), "hello world");
    }

    #[test]
    fn all_const() {
        const A: &str = "foo";
        const B: &str = "bar";
        const RESULT: &str = fconcat!(const { A }, const { B }, "baz");
        assert_eq!(RESULT, "foobarbaz");
    }

    #[test]
    fn mixed() {
        const PREFIX: &str = "pre";
        let dynamic = alloc::string::String::from("dyn");
        let result = fconcat!(const { PREFIX }, "-", dynamic.as_str(), "-suf");
        assert_eq!(result, "pre-dyn-suf");
    }
}
