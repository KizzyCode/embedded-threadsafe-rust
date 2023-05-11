//! A fast, thread local lazy singleton

use crate::{lazy::LazyCell, runtime};
use core::fmt::{self, Debug, Formatter};

/// A fast, thread local lazy singleton
///
/// # Warning
/// This singleton must not be accessed from another core/thread or an interrupt context; doing so will raise a panic. For
/// interrupt-safe singletons use [`InterruptSingleton`], and for multicore/thread-safe singletons use [`SharedSingleton`].
pub struct LocalSingleton<T, const THREAD_ID: usize, I = fn() -> T> {
    /// The singleton value
    inner: LazyCell<T, I>,
}
impl<T, const THREAD_ID: usize, I> LocalSingleton<T, THREAD_ID, I>
where
    I: FnOnce() -> T,
{
    /// Creates a new thread local singleton
    pub const fn new(init: I) -> Self {
        Self { inner: LazyCell::new(init) }
    }

    /// Provides scoped access to the underlying value
    ///
    /// # Panic
    /// This function will panic if called from another thread or interrupt context
    pub fn scope<F, FR>(&self, scope: F) -> FR
    where
        F: FnOnce(&mut T) -> FR,
    {
        // Ensure that we access this from the correct thread ID
        let thread_id = unsafe { runtime::_runtime_threadid_ZhZIZBv3() };
        assert_eq!(thread_id, THREAD_ID, "cannot access local singleton from different thread context");

        // Ensure that we are not in an interrupt handler
        let is_interrupted = unsafe { runtime::_runtime_isinterrupted_v5tnnoC7() };
        assert!(!is_interrupted, "cannot access local singleton from an interrupt handler");

        // Provide access to the value
        unsafe { self.raw(scope) }
    }

    /// Provides an unsafe raw scoped access to the underlying value
    ///
    /// # Safety
    /// This function can also be called from other thread or interrupt contexts and does not perform any kind of
    /// synchronization or safety check or whatsoever - it is up to the caller to avoid race conditions.
    pub unsafe fn raw<F, FR>(&self, scope: F) -> FR
    where
        F: FnOnce(&mut T) -> FR,
    {
        self.inner.scope(scope)
    }
}
impl<T, const THREAD_ID: usize, I> Debug for LocalSingleton<T, THREAD_ID, I>
where
    I: Fn() -> T + Copy,
    T: Debug,
{
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        // Return an opaque description if we are in a different thread context
        let thread_id = unsafe { runtime::_runtime_threadid_ZhZIZBv3() };
        if thread_id != THREAD_ID {
            return f.debug_tuple("LocalSingleton").field(&"<opaque due to different thread context>").finish();
        }

        // Return an opaque description if we are in an interrupt context
        let is_interrupted = unsafe { runtime::_runtime_isinterrupted_v5tnnoC7() };
        if is_interrupted {
            return f.debug_tuple("LocalSingleton").field(&"<opaque due to interrupt context>").finish();
        }

        // Debug the value
        self.scope(|value| value.fmt(f))
    }
}
unsafe impl<T, const THREADS_MAX: usize, I> Sync for LocalSingleton<T, THREADS_MAX, I>
where
    T: Send,
    I: Send,
{
    // Marker trait, no members to implement
}
