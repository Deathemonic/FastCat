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

    #[test]
    fn sep_all_const() {
        const RESULT: &str = fconcat!("-"; "a", "b", "c");
        assert_eq!(RESULT, "a-b-c");
    }

    #[test]
    fn sep_all_const_named() {
        const A: &str = "foo";
        const B: &str = "bar";
        const RESULT: &str = fconcat!("-"; const { A }, const { B }, "baz");
        assert_eq!(RESULT, "foo-bar-baz");
    }

    #[test]
    fn sep_lit_dyn_lit() {
        let dynamic = alloc::string::String::from("dyn");
        let result = fconcat!("-"; "a", dynamic.as_str(), "c");
        assert_eq!(result, "a-dyn-c");
    }

    #[test]
    fn sep_dyn_dyn_gap() {
        let x = alloc::string::String::from("X");
        let y = alloc::string::String::from("Y");
        let result = fconcat!("-"; "a", x.as_str(), y.as_str(), "c");
        assert_eq!(result, "a-X-Y-c");
    }

    #[test]
    fn sep_all_dynamic() {
        let a = alloc::string::String::from("a");
        let b = alloc::string::String::from("b");
        let c = alloc::string::String::from("c");
        let result = fconcat!("-"; a.as_str(), b.as_str(), c.as_str());
        assert_eq!(result, "a-b-c");
    }

    #[test]
    fn sep_dynamic_separator() {
        let sep = alloc::string::String::from(":");
        let x = alloc::string::String::from("X");
        let y = alloc::string::String::from("Y");
        let result = fconcat!(sep.as_str(); x.as_str(), y.as_str());
        assert_eq!(result, "X:Y");
    }

    #[test]
    fn sep_dynamic_separator_reused_across_gaps() {
        let sep = alloc::string::String::from("|");
        let a = alloc::string::String::from("a");
        let b = alloc::string::String::from("b");
        let c = alloc::string::String::from("c");
        let result = fconcat!(sep.as_str(); a.as_str(), b.as_str(), c.as_str());
        assert_eq!(result, "a|b|c");
    }

    #[test]
    fn sep_single_item() {
        let result = fconcat!("-"; "solo");
        assert_eq!(result, "solo");
    }

    #[test]
    fn sep_two_literals_only() {
        const RESULT: &str = fconcat!("-"; "only");
        assert_eq!(RESULT, "only");
    }

    #[test]
    fn sep_three_literals_chain_fuse() {
        const RESULT: &str = fconcat!("-"; "a", "b", "c", "d");
        assert_eq!(RESULT, "a-b-c-d");
    }

    #[test]
    fn sep_named_const_then_two_literals() {
        const A: &str = "hello ";
        const RESULT: &str = fconcat!("-"; const { A }, "literal ", "world");
        assert_eq!(RESULT, "hello -literal -world");
    }

    #[test]
    fn sep_two_named_consts_then_two_literals() {
        const A: &str = "hello ";
        const B: &str = "const ";
        const RESULT: &str = fconcat!("-"; const { A }, const { B }, "literal ", "world");
        assert_eq!(RESULT, "hello -const -literal -world");
    }

    #[test]
    fn sep_const_block_wrapping_a_literal() {
        const RESULT: &str = fconcat!("-"; "literal ", const { "world" });
        assert_eq!(RESULT, "literal -world");
    }

    #[test]
    fn sep_exact_all_const_repro() {
        const A: &str = "hello ";
        const B: &str = "const ";
        const RESULT: &str = fconcat!("-"; const { A }, const { B }, "literal ", const { "world" });
        assert_eq!(RESULT, "hello -const -literal -world");
    }

    #[test]
    fn sep_four_literals_middle_named_const() {
        const MID: &str = "MID";
        const RESULT: &str = fconcat!("-"; "a", "b", const { MID }, "c", "d");
        assert_eq!(RESULT, "a-b-MID-c-d");
    }

    #[test]
    fn sep_long_literal_run_after_dynamic() {
        let d = alloc::string::String::from("DYN");
        let result = fconcat!("-"; d.as_str(), "a", "b", "c", "d");
        assert_eq!(result, "DYN-a-b-c-d");
    }
}
