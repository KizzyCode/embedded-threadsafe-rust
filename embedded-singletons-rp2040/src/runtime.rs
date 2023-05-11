//! Provides the runtime specific functions for an rp2040 platform

use core::sync::atomic::{self, Ordering};
use cortex_m::{
    interrupt,
    peripheral::{scb::VectActive, SCB},
    register::primask,
};
use rp2040_hal::sio::Sio;

/// Ensures that `code` is run exclusively, without being subject to multicore/-thread race conditions or interrupts
#[no_mangle]
#[doc(hidden)]
#[allow(non_snake_case)]
pub fn _runtime_threadsafe_e0LtH0x3(code: &mut dyn FnMut()) {
    critical_section::with(|_| code())
}

/// Ensures that `code` is run exclusively, without being subject to interrupts
///
/// # Note
/// Unlike `_runtime_threadsafe_e0LtH0x3`, this function does not protect against multicore/-thread race conditions
#[no_mangle]
#[doc(hidden)]
#[allow(non_snake_case)]
pub fn _runtime_interruptsafe_1l52Ge5e(code: &mut dyn FnMut()) {
    // Disable interrupts, ensure the compiler doesn't re-order accesses and violate safety here
    let interrupts_active = primask::read().is_active();
    interrupt::disable();
    atomic::compiler_fence(Ordering::SeqCst);

    // Execute our code, ensure the compiler doesn't re-order accesses and violate safety here
    code();
    atomic::compiler_fence(Ordering::SeqCst);

    // Re-enable interrupts if appropriate
    if interrupts_active {
        unsafe { interrupt::enable() };
    }
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
