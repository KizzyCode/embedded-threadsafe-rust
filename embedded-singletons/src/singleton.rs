//! Implements shared and thread-/core-local singletons

use crate::runtime;
use core::{
    cell::UnsafeCell,
    fmt::{self, Debug, Formatter},
};

/// A lazily instantiated cell
struct LazyCell<T, I> {
    /// A tuple containing the initializer and the value
    inner: UnsafeCell<(Option<I>, Option<T>)>,
}
impl<T, I> LazyCell<T, I> {
    /// Creates a new lazy singleton cell with the given initializer
    pub const fn new(init: I) -> Self {
        let value = (Some(init), None);
        Self { inner: UnsafeCell::new(value) }
    }

    /// Provides scoped access to the underlying value, initializes it if necessary
    #[inline]
    pub unsafe fn scope<F, FR>(&self, scope: F) -> FR
    where
        I: FnOnce() -> T,
        F: FnOnce(&mut T) -> FR,
    {
        // Get the inner state
        let inner_ptr = self.inner.get();
        let (init, value) = inner_ptr.as_mut().expect("unexpected NULL pointer inside cell");

        // Initialize the value if necessary
        if let Some(init) = init.take() {
            let value_ = init();
            *value = Some(value_);
        }

        // Take the initialized value
        let Some(value) = value.as_mut() else {
            unreachable!("initialized cell has not value");
        };

        // Call the scope
        scope(value)
    }
}

/// A globally shared, lazy singleton
pub struct SharedSingleton<T, I = fn() -> T> {
    /// The singleton value
    inner: LazyCell<T, I>,
}
impl<T, I> SharedSingleton<T, I>
where
    I: FnOnce() -> T,
{
    /// Creates a new lazy singleton with the given initializer
    pub const fn new(init: I) -> Self {
        Self { inner: LazyCell::new(init) }
    }

    /// Provides scoped access to the underlying value
    pub fn scope<F, FR>(&self, scope: F) -> FR
    where
        F: FnOnce(&mut T) -> FR,
    {
        // Create mutable slots to transfer state to/from the closure
        let mut scope = Some(scope);
        let mut result: Option<FR> = None;

        // Create the caller
        let mut call_scope = || {
            // Consume and call the scope
            let scope = scope.take().expect("missing scope function");
            let result_ = unsafe { self.raw(scope) };
            result = Some(result_);
        };

        // Run the implementation in a threadsafe context and return the result
        unsafe { runtime::_runtime_threadsafe_e0LtH0x3(&mut call_scope) };
        result.expect("implementation scope did not set result value")
    }

    /// Provides an unsafe raw scoped access to the underlying value
    ///
    /// # Safety
    /// This function does not perform any kind of synchronization or safety check or whatsoever - it is up to the caller
    /// to avoid race conditions.
    pub unsafe fn raw<F, FR>(&self, scope: F) -> FR
    where
        F: FnOnce(&mut T) -> FR,
    {
        self.inner.scope(scope)
    }
}
impl<T, I> Debug for SharedSingleton<T, I>
where
    I: FnOnce() -> T,
    T: Debug,
{
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        self.scope(|value| value.fmt(f))
    }
}
unsafe impl<T, I> Sync for SharedSingleton<T, I>
where
    T: Send,
    I: Send,
{
    // Marker trait, no members to implement
}

/// A fast, thread local lazy singleton
///
/// # Warning
/// This singleton must not be accessed from another thread or interrupt context; doing so will raise a panic. For
/// thread- and interrupt-safe singletons, use [`SharedSingleton`].
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
