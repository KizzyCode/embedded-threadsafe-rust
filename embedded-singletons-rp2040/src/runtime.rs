//! Provides the runtime specific functions for an rp2040 platform

use cortex_m::peripheral::{scb::VectActive, SCB};
use rp2040_hal::sio::Sio;

/// Ensures that `code` is run exclusively, without being subject to race conditions or interrupts
#[no_mangle]
#[doc(hidden)]
#[allow(non_snake_case)]
pub fn _runtime_threadsafe_e0LtH0x3(code: &mut dyn FnMut()) {
    critical_section::with(|_| code())
}

/// Gets the __unique__ and __persistent__ identifier of the current thread (e.g. the number of the current core)
///
/// # Note
/// This function is used to lookup thread-local data, so it is essential that a) the ID is always the same for a
/// given thread and b) IDs are not reused across different threads.
#[no_mangle]
#[doc(hidden)]
#[allow(non_snake_case)]
pub fn _runtime_threadid_ZhZIZBv3() -> usize {
    Sio::core() as usize
}

/// Tests whether we are currently in an interrupt context or not
#[no_mangle]
#[doc(hidden)]
#[allow(non_snake_case)]
pub fn _runtime_isinterrupted_v5tnnoC7() -> bool {
    SCB::vect_active() != VectActive::ThreadMode
}
