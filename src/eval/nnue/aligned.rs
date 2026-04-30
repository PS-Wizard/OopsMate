use std::alloc::{Layout, alloc, dealloc, handle_alloc_error};
use std::fmt::{self, Debug};
use std::mem::align_of;
use std::ops::{Deref, DerefMut};
use std::ptr::{self, NonNull};
use std::slice;

pub(crate) const CACHELINE_BYTES: usize = 64;

#[repr(C, align(64))]
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct CacheAligned<T>(pub(crate) T);

impl<T> CacheAligned<T> {
    #[inline(always)]
    pub(crate) const fn new(value: T) -> Self {
        Self(value)
    }
}

impl<T> Deref for CacheAligned<T> {
    type Target = T;

    #[inline(always)]
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<T> DerefMut for CacheAligned<T> {
    #[inline(always)]
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

pub(crate) struct AlignedSlice<T: Copy, const ALIGN: usize = CACHELINE_BYTES> {
    ptr: NonNull<T>,
    len: usize,
}

unsafe impl<T: Copy + Send, const ALIGN: usize> Send for AlignedSlice<T, ALIGN> {}
unsafe impl<T: Copy + Sync, const ALIGN: usize> Sync for AlignedSlice<T, ALIGN> {}

impl<T: Copy, const ALIGN: usize> AlignedSlice<T, ALIGN> {
    #[inline]
    pub(crate) fn from_vec(vec: Vec<T>) -> Self {
        let len = vec.len();
        if len == 0 {
            return Self {
                ptr: NonNull::dangling(),
                len: 0,
            };
        }

        let layout = layout_for::<T, ALIGN>(len);
        let ptr = unsafe { alloc(layout) } as *mut T;
        if ptr.is_null() {
            handle_alloc_error(layout);
        }

        unsafe {
            // SAFETY: `ptr` points to `len` properly aligned `T` slots, and `vec`
            // holds exactly `len` initialized `Copy` elements to copy from.
            ptr::copy_nonoverlapping(vec.as_ptr(), ptr, len);
        }

        Self {
            ptr: unsafe { NonNull::new_unchecked(ptr) },
            len,
        }
    }

    #[inline(always)]
    pub(crate) fn as_ptr(&self) -> *const T {
        self.ptr.as_ptr()
    }

    #[inline(always)]
    pub(crate) fn as_slice(&self) -> &[T] {
        unsafe {
            // SAFETY: `ptr` either dangles for len 0 or points to `len` initialized `T`s.
            slice::from_raw_parts(self.ptr.as_ptr(), self.len)
        }
    }

    #[inline(always)]
    pub(crate) fn as_mut_slice(&mut self) -> &mut [T] {
        unsafe {
            // SAFETY: same invariant as `as_slice`, with unique access through `&mut self`.
            slice::from_raw_parts_mut(self.ptr.as_ptr(), self.len)
        }
    }
}

impl<T: Copy, const ALIGN: usize> Drop for AlignedSlice<T, ALIGN> {
    fn drop(&mut self) {
        if self.len == 0 {
            return;
        }

        let layout = layout_for::<T, ALIGN>(self.len);
        unsafe {
            // SAFETY: `ptr` was allocated with this exact layout in `from_vec`.
            dealloc(self.ptr.as_ptr().cast::<u8>(), layout);
        }
    }
}

impl<T: Copy, const ALIGN: usize> Deref for AlignedSlice<T, ALIGN> {
    type Target = [T];

    #[inline(always)]
    fn deref(&self) -> &Self::Target {
        self.as_slice()
    }
}

impl<T: Copy, const ALIGN: usize> DerefMut for AlignedSlice<T, ALIGN> {
    #[inline(always)]
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.as_mut_slice()
    }
}

impl<T: Copy + Debug, const ALIGN: usize> Debug for AlignedSlice<T, ALIGN> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.as_slice().fmt(f)
    }
}

#[inline]
fn layout_for<T, const ALIGN: usize>(len: usize) -> Layout {
    debug_assert!(ALIGN.is_power_of_two());
    let array = Layout::array::<T>(len).expect("aligned slice layout overflow");
    let alignment = ALIGN.max(align_of::<T>());
    array
        .align_to(alignment)
        .expect("aligned slice invalid alignment")
        .pad_to_align()
}

#[cfg(test)]
mod tests {
    use super::{AlignedSlice, CACHELINE_BYTES};

    #[test]
    fn aligned_slice_preserves_contents_and_alignment() {
        let values = AlignedSlice::<i16>::from_vec(vec![1, -2, 3, -4, 5]);

        assert_eq!(&*values, &[1, -2, 3, -4, 5]);
        assert_eq!(values.as_ptr() as usize % CACHELINE_BYTES, 0);
    }
}
