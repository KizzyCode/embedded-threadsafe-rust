//! Defines requires runtime-specific function stubs

extern "Rust" {
    /// Ensures that `code` is run exclusively, without being subject to multicore/-thread race conditions or interrupts
    pub(crate) fn _runtime_threadsafe_e0LtH0x3(code: &mut dyn FnMut());
    /// Ensures that `code` is run exclusively, without being subject to interrupts
    ///
    /// # Note
    /// Unlike `_runtime_threadsafe_e0LtH0x3`, this function does not protect against multicore/-thread race conditions
    pub(crate) fn _runtime_interruptsafe_1l52Ge5e(code: &mut dyn FnMut());

    /// Gets the __unique__ and __persistent__ identifier of the current thread (e.g. a session-unique thread ID or the
    /// index of the current CPU core on bare-metal systems).
    ///
    /// # Note
    /// This function is used to guard context-local data, so it is essential that a) the ID is always the same for a given
    /// context and b) IDs are not reused across different contexts during the lifetime of the application.
    pub(crate) fn _runtime_threadid_ZhZIZBv4() -> usize;
    /// Tests whether we are currently in an interrupt context or not
    pub(crate) fn _runtime_isinterrupted_v5tnnoC7() -> bool;
}
