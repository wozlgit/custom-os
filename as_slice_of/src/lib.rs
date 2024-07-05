#![no_std]
#![feature(generic_const_exprs)]

use core::marker::PhantomData;
use core::mem::size_of;
use core::ptr::slice_from_raw_parts;

pub const fn slice_element_count<T, R>() -> usize {
    size_of::<T>() / size_of::<R>()
}

struct DummyAsSliceOf<T, R> {
    dt: PhantomData<T>,
    dr: PhantomData<R>
}

trait IsValidSliceTarget<T, R> {
    const RESULT: ();
}

impl<T, R> IsValidSliceTarget<T, R> for DummyAsSliceOf<T, R> {
    const RESULT: () = assert!(size_of::<T>() <= isize::MAX as usize);
}


/// # Safety requirements
/// This function will `panic!` at **compile-time** if `mem::size_of::<T>()` is larger than isize::MAX.
/// Behaviour is undefined if any of the following requirements are violated:
/// * The sum of `mem::size_of::<T>()` and the address of `val` must not "wrap around" the address space
/// * `val` must be aligned on a `mem::size_of::<R>()` -size boundary
/// * All elements of the returned slice must be properly initialized values of type T
/// * The memory referenced by the returned slice must not be mutated except inside an
/// `UnsafeCell` for the duration of `val`'s lifetime
pub fn as_slice_of<T, R>(val: &T) -> &[R; slice_element_count::<T, R>()] {
    let _ = <DummyAsSliceOf<T, R> as IsValidSliceTarget<T, R>>::RESULT;

    let sp = slice_from_raw_parts(val as *const T, slice_element_count::<T, R>())
        as *const [R; slice_element_count::<T, R>()];
    unsafe { &*sp }
}
