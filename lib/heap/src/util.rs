/// Align `addr` downwards to the nearest multiple of `align`.
///
/// The returned usize is always <= `addr.`
///
/// # Panics
///
/// Panics if `align` is not a power of 2.
#[allow(dead_code)]
pub fn align_down(addr: usize, align: usize) -> usize {
    if !align.is_power_of_two() {
        panic!("align is not a power of two");
    }
    
    let align_minus_one = align.checked_sub(1).expect("align overflow");
    addr & !align_minus_one
}

/// Align `addr` upwards to the nearest multiple of `align`.
///
/// The returned `usize` is always >= `addr.`
///
/// # Panics
///
/// Panics if `align` is not a power of 2
/// or aligning up overflows the address.
pub fn align_up(addr: usize, align: usize) -> usize {
    if !align.is_power_of_two() {
        panic!("align is not a power of two");
    }
    
    let align_minus_one = align.checked_sub(1).expect("align overflow");
    addr.checked_add(align_minus_one)
        .expect("align overflow")
        & !align_minus_one
}