use std::{mem::MaybeUninit, pin::Pin};

pub(crate) struct Page<T: Sized, const CAP_PER_PAGE: usize> {
    pub(crate) start: *mut MaybeUninit<T>,
    pub(crate) end: *mut MaybeUninit<T>,
    elems: [MaybeUninit<T>; CAP_PER_PAGE],
}

impl<T, const CAP_PER_PAGE: usize> Page<T, CAP_PER_PAGE>
where
    T: Sized,
{
    pub(crate) fn new() -> Box<Self> {
        let mut res = Box::new(Self {
            start: std::ptr::null_mut(),
            end: std::ptr::null_mut(),
            elems: unsafe { MaybeUninit::uninit().assume_init() },
        });
        res.reset_for_back_insertion();
        res
    }

    pub(crate) fn reset_for_front_insertion(&mut self) {
        let last_ptr = self.last_ptr() as *mut _;
        self.start = last_ptr;
        self.end = last_ptr;
    }

    pub(crate) fn reset_for_back_insertion(&mut self) {
        let first_ptr = self.first_ptr() as *mut _;
        self.start = first_ptr;
        self.end = first_ptr;
    }

    pub(crate) fn reserve_front(&mut self) -> Option<&mut MaybeUninit<T>> {
        if self.start > self.first_ptr() as *mut _ {
            unsafe {
                let res = self.start.wrapping_offset(-1);
                self.start = res;
                Some(&mut *res)
            }
        } else {
            None
        }
    }

    pub(crate) fn reserve_back(&mut self) -> Option<&mut MaybeUninit<T>> {
        if self.end < self.last_ptr() as *mut _ {
            unsafe {
                let res = self.end;
                self.end = res.wrapping_add(1);
                Some(&mut *res)
            }
        } else {
            None
        }
    }

    fn first_ptr(&self) -> *const MaybeUninit<T> {
        &self.elems[0]
    }

    fn last_ptr(&self) -> *const MaybeUninit<T> {
        self.first_ptr().wrapping_add(CAP_PER_PAGE)
    }

    pub(crate) fn len(&self) -> usize {
        assert!(self.start <= self.end);
        let res = unsafe { self.end.offset_from(self.start) };
        res as usize
    }

    pub(crate) fn get(&self, idx: usize) -> Pin<&T> {
        unsafe {
            let res = self.start.wrapping_add(idx);
            assert!(res < self.end);
            let res: &MaybeUninit<T> = &*(res as *const _);
            Pin::new_unchecked(res.assume_init_ref())
        }
    }

    pub(crate) fn get_mut(&mut self, idx: usize) -> Pin<&mut T> {
        unsafe {
            let res = self.start.wrapping_add(idx);
            assert!(res < self.end);
            let res: &mut MaybeUninit<T> = &mut *res;
            Pin::new_unchecked(res.assume_init_mut())
        }
    }

    pub(crate) fn front(&self) -> Pin<&T> {
        assert!(self.end > self.start);
        unsafe {
            let res: &MaybeUninit<T> = &*(self.start as *const _);
            Pin::new_unchecked(res.assume_init_ref())
        }
    }

    pub(crate) fn back(&self) -> Pin<&T> {
        assert!(self.end > self.start);
        unsafe {
            let res = self.end.wrapping_offset(-1);
            let res: &MaybeUninit<T> = &*(res as *const _);
            Pin::new_unchecked(res.assume_init_ref())
        }
    }

    pub(crate) fn front_mut(&mut self) -> Pin<&mut T> {
        assert!(self.end > self.start);
        unsafe {
            let res = &mut *self.start;
            Pin::new_unchecked(res.assume_init_mut())
        }
    }

    pub(crate) fn back_mut(&mut self) -> Pin<&mut T> {
        assert!(self.end > self.start);
        unsafe {
            let res = self.end.wrapping_offset(-1);
            let res = &mut *res;
            Pin::new_unchecked(res.assume_init_mut())
        }
    }

    pub(crate) fn pop_front(&mut self) {
        assert!(self.end > self.start);
        unsafe {
            let res = &mut *self.start;
            self.start = self.start.wrapping_add(1);
            res.assume_init_drop();
        }
    }

    pub(crate) fn pop_back(&mut self) {
        assert!(self.end > self.start);
        unsafe {
            self.end = self.end.wrapping_offset(-1);
            let res = &mut *self.end;
            res.assume_init_drop();
        }
    }

    pub(crate) fn front_capacity(&self) -> usize {
        let first_ptr = self.first_ptr() as *mut _;
        assert!(first_ptr <= self.start);
        unsafe { self.start.offset_from(first_ptr) as usize }
    }

    pub(crate) fn back_capacity(&self) -> usize {
        let last_ptr = self.last_ptr() as *mut _;
        assert!(self.end <= last_ptr);
        unsafe { last_ptr.offset_from(self.end) as usize }
    }

    pub(crate) fn drop_all(&mut self) {
        assert!(self.start <= self.end);
        let mut ptr = self.start;
        while ptr < self.end {
            unsafe {
                let obj: &mut MaybeUninit<T> = &mut *ptr;
                obj.assume_init_drop();
            }
            ptr = ptr.wrapping_add(1);
        }
    }
}
