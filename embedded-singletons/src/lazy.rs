//! A lazily instantiated cell

use core::cell::UnsafeCell;

/// A lazily instantiated cell
pub struct LazyCell<T, I> {
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
