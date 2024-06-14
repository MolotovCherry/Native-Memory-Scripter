use std::{fmt::Debug, ops::Deref};
use std::{ops::DerefMut, ptr::NonNull};

#[derive(Copy, Clone)]
pub struct RawSendable<T>(pub NonNull<T>);

unsafe impl<T> Send for RawSendable<T> {}
unsafe impl<T> Sync for RawSendable<T> {}

impl<T> RawSendable<T> {
    pub fn new(ptr: *mut T) -> Self {
        Self(NonNull::new(ptr).unwrap())
    }
}

impl<T> Debug for RawSendable<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.0.fmt(f)
    }
}

impl<T> Deref for RawSendable<T> {
    type Target = NonNull<T>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<T> DerefMut for RawSendable<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

#[derive(Copy, Clone)]
pub struct Sendable<T>(pub T);

unsafe impl<T> Send for Sendable<T> {}
unsafe impl<T> Sync for Sendable<T> {}

impl<T> Deref for Sendable<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<T> DerefMut for Sendable<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}
