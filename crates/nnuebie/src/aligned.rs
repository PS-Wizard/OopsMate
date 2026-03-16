//! Fixed-alignment heap buffers used by SIMD-sensitive code.

use std::alloc::{alloc, dealloc, handle_alloc_error, Layout};
use std::ops::{Deref, DerefMut, Index, IndexMut};
use std::ptr::{self, NonNull};
use std::slice;

/// A heap-allocated buffer with 64-byte alignment.
pub struct AlignedBuffer<T: Copy> {
    ptr: NonNull<T>,
    len: usize,
    capacity: usize,
}

unsafe impl<T: Copy + Send> Send for AlignedBuffer<T> {}
unsafe impl<T: Copy + Sync> Sync for AlignedBuffer<T> {}

impl<T: Copy> AlignedBuffer<T> {
    const ALIGNMENT: usize = 64;

    /// Allocates and initializes a buffer of length `len` with `T::default()`.
    pub fn new(len: usize) -> Self
    where
        T: Default,
    {
        Self::with_element(len, T::default())
    }

    /// Allocates and initializes a buffer of length `len` with `elem`.
    pub fn with_element(len: usize, elem: T) -> Self {
        let mut buf = Self::with_capacity(len);
        for i in 0..len {
            unsafe {
                ptr::write(buf.ptr.as_ptr().add(i), elem);
            }
        }
        buf.len = len;
        buf
    }

    /// Allocates capacity for `capacity` elements without initializing logical length.
    pub fn with_capacity(capacity: usize) -> Self {
        if capacity == 0 {
            return Self {
                ptr: NonNull::dangling(),
                len: 0,
                capacity: 0,
            };
        }

        let layout = Layout::from_size_align(
            capacity.checked_mul(std::mem::size_of::<T>()).unwrap(),
            Self::ALIGNMENT,
        )
        .unwrap();

        let ptr = unsafe { alloc(layout) } as *mut T;
        if ptr.is_null() {
            handle_alloc_error(layout);
        }

        Self {
            ptr: unsafe { NonNull::new_unchecked(ptr) },
            len: 0,
            capacity,
        }
    }

    /// Copies an existing vector into a 64-byte-aligned buffer.
    pub fn from_vec(vec: Vec<T>) -> Self {
        let mut buf = Self::with_capacity(vec.len());
        unsafe {
            ptr::copy_nonoverlapping(vec.as_ptr(), buf.ptr.as_ptr(), vec.len());
        }
        buf.len = vec.len();
        buf
    }

    /// Returns the initialized portion as an immutable slice.
    pub fn as_slice(&self) -> &[T] {
        unsafe { slice::from_raw_parts(self.ptr.as_ptr(), self.len) }
    }

    /// Returns the initialized portion as a mutable slice.
    pub fn as_mut_slice(&mut self) -> &mut [T] {
        unsafe { slice::from_raw_parts_mut(self.ptr.as_ptr(), self.len) }
    }
}

impl<T: Copy> Drop for AlignedBuffer<T> {
    fn drop(&mut self) {
        if self.capacity > 0 {
            let layout = Layout::from_size_align(
                self.capacity.checked_mul(std::mem::size_of::<T>()).unwrap(),
                Self::ALIGNMENT,
            )
            .unwrap();
            unsafe {
                dealloc(self.ptr.as_ptr() as *mut u8, layout);
            }
        }
    }
}

impl<T: Copy> Deref for AlignedBuffer<T> {
    type Target = [T];
    fn deref(&self) -> &Self::Target {
        self.as_slice()
    }
}

impl<T: Copy> DerefMut for AlignedBuffer<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.as_mut_slice()
    }
}

impl<T: Copy> Clone for AlignedBuffer<T> {
    fn clone(&self) -> Self {
        let mut new_buf = Self::with_capacity(self.len);
        unsafe {
            ptr::copy_nonoverlapping(self.ptr.as_ptr(), new_buf.ptr.as_ptr(), self.len);
        }
        new_buf.len = self.len;
        new_buf
    }
}

impl<T: Copy> Index<usize> for AlignedBuffer<T> {
    type Output = T;
    fn index(&self, index: usize) -> &Self::Output {
        &self.as_slice()[index]
    }
}

impl<T: Copy> IndexMut<usize> for AlignedBuffer<T> {
    fn index_mut(&mut self, index: usize) -> &mut Self::Output {
        &mut self.as_mut_slice()[index]
    }
}

impl<T: Copy> Index<std::ops::Range<usize>> for AlignedBuffer<T> {
    type Output = [T];
    fn index(&self, range: std::ops::Range<usize>) -> &Self::Output {
        &self.as_slice()[range]
    }
}

impl<T: Copy> IndexMut<std::ops::Range<usize>> for AlignedBuffer<T> {
    fn index_mut(&mut self, range: std::ops::Range<usize>) -> &mut Self::Output {
        &mut self.as_mut_slice()[range]
    }
}

impl<T: Copy> Index<std::ops::RangeTo<usize>> for AlignedBuffer<T> {
    type Output = [T];
    fn index(&self, range: std::ops::RangeTo<usize>) -> &Self::Output {
        &self.as_slice()[range]
    }
}

impl<T: Copy> Index<std::ops::RangeFrom<usize>> for AlignedBuffer<T> {
    type Output = [T];
    fn index(&self, range: std::ops::RangeFrom<usize>) -> &Self::Output {
        &self.as_slice()[range]
    }
}
