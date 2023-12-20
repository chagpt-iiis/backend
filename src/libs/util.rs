#[macro_export]
macro_rules! assume {
    ($cond:expr) => {
        #[cfg(debug_assertions)]
        assert!($cond);

        #[cfg(not(debug_assertions))]
        unsafe {
            core::intrinsics::assume($cond);
        }
    };
}
