//! A lazy singleton that can be safely be shared across interrupt contexts

use crate::{lazy::LazyCell, runtime};
use core::fmt::{self, Debug, Formatter};

/// A lazy singleton that can be safely be shared across interrupt contexts
///
/// # Warning
/// This singleton must not be accessed from another core/thread; doing so will raise a panic. For multicore/thread-safe
/// singletons use [`SharedSingleton`].
pub struct InterruptSingleton<T, const THREAD_ID: usize, I = fn() -> T> {
    /// The singleton value
    inner: LazyCell<T, I>,
}
impl<T, const THREAD_ID: usize, I> InterruptSingleton<T, THREAD_ID, I>
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

        // Create mutable slots to transfer state to/from the closure and create the caller
        let mut scope = Some(scope);
        let mut result: Option<FR> = None;
        let mut call_scope = || {
            // Consume and call the scope
            let scope = scope.take().expect("missing scope function");
            let result_ = unsafe { self.raw(scope) };
            result = Some(result_);
        };

        // Run the implementation in a threadsafe context and return the result
        unsafe { runtime::_runtime_interruptsafe_1l52Ge5e(&mut call_scope) };
        result.expect("implementation scope did not set result value")
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
impl<T, const THREAD_ID: usize, I> Debug for InterruptSingleton<T, THREAD_ID, I>
where
    I: Fn() -> T + Copy,
    T: Debug,
{
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        // Return an opaque description if we are in a different thread context
        let thread_id = unsafe { runtime::_runtime_threadid_ZhZIZBv3() };
        if thread_id != THREAD_ID {
            return f.debug_tuple("InterruptSingleton").field(&"<opaque due to different thread context>").finish();
        }

        // Debug the value
        self.scope(|value| value.fmt(f))
    }
}
unsafe impl<T, const THREADS_MAX: usize, I> Sync for InterruptSingleton<T, THREADS_MAX, I>
where
    T: Send,
    I: Send,
{
    // Marker trait, no members to implement
}
