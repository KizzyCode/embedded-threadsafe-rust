//! A cell that can be safely be shared across thread boundaries and interrupt contexts

use crate::runtime;
use core::{
    cell::UnsafeCell,
    fmt::{self, Debug, Formatter},
};

/// A cell that can be safely be shared across thread boundaries and interrupt contexts
pub struct SharedCell<T> {
    /// The wrapped value
    inner: UnsafeCell<T>,
}
impl<T> SharedCell<T> {
    /// Creates a new cell
    pub const fn new(value: T) -> Self {
        Self { inner: UnsafeCell::new(value) }
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
impl<T> Debug for SharedCell<T>
where
    T: Debug,
{
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        self.scope(|value| value.fmt(f))
    }
}
unsafe impl<T> Sync for SharedCell<T>
where
    T: Send,
{
    // Marker trait, no members to implement
}
