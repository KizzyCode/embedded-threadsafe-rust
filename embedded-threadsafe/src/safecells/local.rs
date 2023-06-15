//! A fast, thread-local cell

use crate::runtime;
use core::{
    cell::UnsafeCell,
    fmt::{self, Debug, Formatter},
};

/// A fast, thread-local cell
///
/// # Warning
/// This cell must not be accessed from another thread or an interrupt context; doing so will raise a panic.
pub struct LocalCell<T> {
    /// The wrapped value
    inner: UnsafeCell<T>,
    /// The associated thread ID
    thread_id: usize,
}
impl<T> LocalCell<T> {
    /// Creates a new thread-local cell
    pub const fn new_with_threadid(value: T, thread_id: usize) -> Self {
        Self { inner: UnsafeCell::new(value), thread_id }
    }

    /// Creates a new thread-local cell
    pub fn new(value: T) -> Self {
        // Get the thread ID and init self
        let thread_id = unsafe { runtime::_runtime_threadid_ZhZIZBv4() };
        Self::new_with_threadid(value, thread_id)
    }

    /// Provides scoped access to the underlying value
    ///
    /// # Panic
    /// This function will panic if called from another thread or interrupt context
    pub fn scope<F, FR>(&self, scope: F) -> FR
    where
        F: FnOnce(&mut T) -> FR,
    {
        // Ensure that we are not in an interrupt handler
        let is_interrupted = unsafe { runtime::_runtime_isinterrupted_v5tnnoC7() };
        assert!(!is_interrupted, "cannot access local cell from an interrupt handler");

        // Ensure that we access this from the correct thread
        let thread_id = unsafe { runtime::_runtime_threadid_ZhZIZBv4() };
        assert_eq!(thread_id, self.thread_id, "cannot access local cell from another thread");

        // Provide access to the value
        unsafe { self.raw(scope) }
    }

    /// Provides an unsafe raw scoped access to the underlying value
    ///
    /// # Safety
    /// This function provides unchecked, mutable access to the underlying value, so incorrect use of this function may
    /// lead to race conditions or undefined behavior.
    pub unsafe fn raw<F, FR>(&self, scope: F) -> FR
    where
        F: FnOnce(&mut T) -> FR,
    {
        // Provide access to the inner value
        let inner_ptr = self.inner.get();
        let value = inner_ptr.as_mut().expect("unexpected NULL pointer inside cell");
        scope(value)
    }
}
impl<T> Debug for LocalCell<T>
where
    T: Debug,
{
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        // Return an opaque description if we are in a different thread
        let thread_id = unsafe { runtime::_runtime_threadid_ZhZIZBv4() };
        if thread_id != self.thread_id {
            return f.debug_tuple("LocalCell").field(&"<opaque due to different thread>").finish();
        }

        // Return an opaque description if we are in an interrupt context
        let is_interrupted = unsafe { runtime::_runtime_isinterrupted_v5tnnoC7() };
        if is_interrupted {
            return f.debug_tuple("LocalCell").field(&"<opaque due to interrupt context>").finish();
        }

        // Debug the value
        self.scope(|value| value.fmt(f))
    }
}
unsafe impl<T> Sync for LocalCell<T>
where
    T: Send,
{
    // Marker trait, no members to implement
}
