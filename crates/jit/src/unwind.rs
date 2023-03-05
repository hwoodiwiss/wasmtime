cfg_if::cfg_if! {
    if #[cfg(all(windows, any(target_arch = "x86_64", target_arch = "aarch64")))] {
        mod windows;
        pub use self::windows::*;
    } else if #[cfg(unix)] {
        mod systemv;
        pub use self::systemv::*;
    } else {
        compile_error!("unsupported target platform for unwind");
    }
}
