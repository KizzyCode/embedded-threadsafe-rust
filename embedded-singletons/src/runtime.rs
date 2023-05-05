//! Defines requires runtime-specific function stubs

extern "Rust" {
    /// Ensures that `code` is run exclusively, without being subject to race conditions or interrupts
    pub(crate) fn _runtime_threadsafe_e0LtH0x3(code: &mut dyn FnMut());
    /// Gets the __unique__ and __persistent__ identifier of the current thread (e.g. the number of the current core)
    /// 
    /// # Note
    /// This function is used to lookup thread-local data, so it is essential that a) the ID is always the same for a
    /// given thread and b) IDs are not reused across different threads.
    pub(crate) fn _runtime_threadid_ZhZIZBv3() -> usize;
    /// Tests whether we are currently in an interrupt context or not
    pub(crate) fn _runtime_isinterrupted_v5tnnoC7() -> bool;
}
