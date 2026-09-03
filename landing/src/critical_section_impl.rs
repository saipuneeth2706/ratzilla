// Provides a critical-section implementation for the single-threaded
// wasm32-unknown-unknown target. `ratatui-core` uses `critical-section`
// (via the `layout-cache` feature), but no backend ships for wasm without
// atomics or std, so we provide a no-op implementation here.

#[cfg(target_arch = "wasm32")]
mod wasm_critical_section {
    use critical_section::RawRestoreState;

    struct WasmCriticalSection;

    critical_section::set_impl!(WasmCriticalSection);

    unsafe impl critical_section::Impl for WasmCriticalSection {
        unsafe fn acquire() -> RawRestoreState {}
        unsafe fn release(_restore_state: RawRestoreState) {}
    }
}
