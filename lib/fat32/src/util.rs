use alloc::vec::Vec;
use core::mem::{align_of, forget, size_of};
use core::slice::{from_raw_parts, from_raw_parts_mut};

pub trait VecExt {
    /// Casts a `Vec<T>` into a `Vec<U>`.
    ///
    /// # Safety
    ///
    /// The caller must ensure the following safety properties:
    ///
    ///   * The vector `self` contains valid elements of type `U`. In
    ///     particular, note that `drop` will never be called for `T`s in `self`
    ///     and instead will be called for the `U`'s in `self`.
    ///   * The size and alignment of `T` and `U` are identical.
    ///
    /// # Panics
    ///
    /// Panics if the size or alignment of `T` and `U` differ.
    unsafe fn cast<U>(self) -> Vec<U>;
}

#[allow(dead_code)]
pub trait SliceExt {
    /// Casts an `&[T]` into an `&[U]`.
    ///
    /// # Safety
    ///
    /// The caller must ensure the following safety properties:
    ///
    ///   * The slice `self` contains valid elements of type `U`.
    ///   * The size of `T` and `U` are identical.
    ///   * The alignment of `T` is an integer multiple of the alignment of `U`.
    ///
    /// # Panics
    ///
    /// Panics if the size of `T` and `U` differ or if the alignment of `T` is
    /// not an integer multiple of `U`.
    unsafe fn cast<'a, U>(&'a self) -> &'a [U];

    /// Casts an `&mut [T]` into an `&mut [U]`.
    ///
    /// # Safety
    ///
    /// The caller must ensure the following safety properties:
    ///
    ///   * The slice `self` contains valid elements of type `U`.
    ///   * The size of `T` and `U` are identical.
    ///   * The alignment of `T` is an integer multiple of the alignment of `U`.
    ///
    /// # Panics
    ///
    /// Panics if the size of `T` and `U` differ or if the alignment of `T` is
    /// not an integer multiple of `U`.
    unsafe fn cast_mut<'a, U>(&'a mut self) -> &'a mut [U];
}

fn calc_new_len_cap<T, U>(vec: &Vec<T>) -> (usize, usize) {
    if size_of::<T>() > size_of::<U>() {
        assert!(size_of::<T>() % size_of::<U>() == 0);
        let factor = size_of::<T>() / size_of::<U>();
        (vec.len() * factor, vec.capacity() * factor)
    } else if size_of::<U>() > size_of::<T>() {
        assert!(size_of::<U>() % size_of::<T>() == 0);
        let factor = size_of::<U>() / size_of::<T>();
        (vec.len() / factor, vec.capacity() / factor)
    } else {
        (vec.len(), vec.capacity())
    }
}

impl<T> VecExt for Vec<T> {
    unsafe fn cast<U>(mut self) -> Vec<U> {
        assert!(align_of::<T>() == align_of::<U>());

        let (new_len, new_cap) = calc_new_len_cap::<T, U>(&self);
        let new_ptr = self.as_mut_ptr() as *mut U;
        forget(self);

        Vec::from_raw_parts(new_ptr, new_len, new_cap)
    }
}

fn calc_new_len<T, U>(slice: &[T]) -> usize {
    if size_of::<T>() > size_of::<U>() {
        assert!(size_of::<T>() % size_of::<U>() == 0);
        let factor = size_of::<T>() / size_of::<U>();
        slice.len() * factor
    } else if size_of::<U>() > size_of::<T>() {
        assert!(size_of::<U>() % size_of::<T>() == 0);
        let factor = size_of::<U>() / size_of::<T>();
        slice.len() / factor
    } else {
        slice.len()
    }
}

impl<T> SliceExt for [T] {
    unsafe fn cast<'a, U>(&'a self) -> &'a [U] {
        assert!(align_of::<T>() % align_of::<U>() == 0);

        let new_len = calc_new_len::<T, U>(self);
        let new_ptr = self.as_ptr() as *const U;
        from_raw_parts(new_ptr, new_len)
    }

    unsafe fn cast_mut<'a, U>(&'a mut self) -> &'a mut [U] {
        assert!(align_of::<T>() % align_of::<U>() == 0);

        let new_len = calc_new_len::<T, U>(self);
        let new_ptr = self.as_mut_ptr() as *mut U;
        from_raw_parts_mut(new_ptr, new_len)
    }
}


use shim::io::Cursor;
pub struct FATCursor<T>(Cursor<T>);

use core::ops::{Deref, DerefMut};
impl<T> Deref for FATCursor<T> {
    type Target = Cursor<T>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<T> DerefMut for FATCursor<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl<T> FATCursor<T> {
    pub fn new(inner: T) -> FATCursor<T> {
        FATCursor {
            0: Cursor::new(inner)
        }
    }
}


use shim::io::Write;
use shim::io;

impl Write for FATCursor<Vec<u8>> {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        self.get_mut().write(buf)
    }

    fn flush(&mut self) -> io::Result<()> {
        self.get_mut().flush()
    }
}

use alloc::boxed::Box;
impl Write for FATCursor<Box<[u8]>> {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        let position = self.position() as usize;
        let buffer = self.get_mut().as_mut();

        if position >= buffer.len() {
            return Err(io::Error::new(io::ErrorKind::WriteZero, "cursor out of bounds"));
        }

        let remaining_space = &mut buffer[position..];
        let bytes_to_write = buf.len().min(remaining_space.len());
        remaining_space[..bytes_to_write].copy_from_slice(&buf[..bytes_to_write]);
        self.set_position((position + bytes_to_write) as u64);

        Ok(bytes_to_write)
    }

    fn flush(&mut self) -> io::Result<()> {
        Ok(())
    }
}

