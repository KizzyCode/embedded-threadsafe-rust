//! A fast, thread-local cell that can be safely shared accross interrupt contexts

use crate::{runtime, LazyCell};
use core::{
    cell::UnsafeCell,
    fmt::{self, Debug, Formatter},
};

/// A fast, thread-local cell that can be safely shared accross interrupt contexts
///
/// # Warning
/// This cell must not be accessed from another thread; doing so will raise a panic.
pub struct InterruptCell<T> {
    /// The wrapped value
    inner: UnsafeCell<T>,
    /// The associated thread ID
    thread_id: usize,
}
impl<T> InterruptCell<T> {
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
        // Ensure that we access this from the correct thread ID
        let thread_id = unsafe { runtime::_runtime_threadid_ZhZIZBv4() };
        assert_eq!(thread_id, self.thread_id, "cannot access local cell from another thread");

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
impl<T> InterruptCell<LazyCell<T>> {
    /// Provides scoped access to the underlying lazy cell
    ///
    /// # Panic
    /// This function will panic if called from another thread or interrupt context
    pub fn lazy_scope<F, FR>(&self, scope: F) -> FR
    where
        F: FnOnce(&mut T) -> FR,
    {
        self.scope(|lazy| lazy.scope_mut(scope))
    }
}
impl<T> Debug for InterruptCell<T>
where
    T: Debug,
{
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        // Return an opaque description if we are in a different thread context
        let thread_id = unsafe { runtime::_runtime_threadid_ZhZIZBv4() };
        if thread_id != self.thread_id {
            return f.debug_tuple("InterruptCell").field(&"<opaque due to different thread>").finish();
        }

        // Debug the value
        self.scope(|value| value.fmt(f))
    }
}
unsafe impl<T> Sync for InterruptCell<T>
where
    T: Send,
{
    // Marker trait, no members to implement
}
