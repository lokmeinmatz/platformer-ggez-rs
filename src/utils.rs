use std::ops::{Neg, Sub};
use std::rc::{Rc, Weak};
use std::cell::RefCell;

/// This function maps all items inplace, returning a vec of the new type
/// must be same size!
/// if f panics, it results in undefined behaviour because it is unknown what was stored
pub fn map_in_place<F, T>(mut source: Vec<F>, f: fn(F) -> T) -> Vec<T> {
    assert_eq!(
        std::mem::size_of::<F>(),
        std::mem::size_of::<T>(),
        "wrong sizes"
    );
    assert_eq!(
        std::mem::align_of::<F>(),
        std::mem::align_of::<T>(),
        "wrong alignment"
    );

    let len = source.len();
    let cap = source.capacity();
    let read = source.as_mut_ptr();
    let write = read as *mut T;
    std::mem::forget(source);

    for i in 0..len {
        unsafe {
            let old = read.offset(i as isize).read();
            let new = f(old);
            write.offset(i as isize).write(new);
        }
    }
    unsafe { Vec::from_raw_parts(write, len, cap) }
}

pub fn mostly_eq<T>(a: T, b: T, max_err: T) -> bool
where
    T: Sub<Output = T> + std::cmp::PartialOrd + Neg<Output = T>,
{
    let zero = a - b;
    zero < max_err && zero > -max_err
}



pub type Shared<T> = Rc<RefCell<T>>;
pub type SharedWeak<T> = Weak<RefCell<T>>;

pub fn shared<T>(inner: T) -> Shared<T> {
    Rc::new(RefCell::new(inner))
}
