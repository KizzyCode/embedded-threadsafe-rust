//! A lazy singleton that can be safely be shared across multicore or thread boundaries and interrupt contexts

use crate::{lazy::LazyCell, runtime};
use core::fmt::{self, Debug, Formatter};

/// A lazy singleton that can be safely be shared across multicore or thread boundaries and interrupt contexts
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
