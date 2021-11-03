use std::any::{type_name, TypeId};
use std::mem;

// Code adopted from https://github.com/sagebind/castaway.
fn type_eq<T: 'static, U: 'static>() -> bool {
    // Reduce the chance of `TypeId` collisions causing a problem by also
    // verifying the layouts match and the type names match. Since `T` and `U`
    // are known at compile time the compiler should optimize away these extra
    // checks anyway.
    mem::size_of::<T>() == mem::size_of::<U>()
        && mem::align_of::<T>() == mem::align_of::<U>()
        && TypeId::of::<T>() == TypeId::of::<U>()
        && type_name::<T>() == type_name::<U>()
}

#[inline]
unsafe fn transmute_unchecked<T, U>(value: T) -> U {
    mem::transmute_copy(&mem::ManuallyDrop::new(value))
}

#[inline]
pub fn assert_transmute<T: 'static, U: 'static>(value: T) -> U {
    assert!(type_eq::<T, U>(), "type mismatch");
    unsafe { transmute_unchecked(value) }
}
