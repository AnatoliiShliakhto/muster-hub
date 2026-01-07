#[cfg(feature = "server")]
pub mod server;
#[cfg(not(target_arch = "wasm32"))]
pub mod config;
pub mod prelude;
pub mod security;
pub mod system;

pub use nanoid::nanoid;

#[macro_export]
macro_rules! safe_nanoid {
    () => {
        // Professional alphabet: 2-9, A-Z (minus I, O), a-z (minus l)
        $crate::nanoid!(12, &"23456789ABCDEFGHJKLMNPQRSTUVWXYZabcdefghijkmnopqrstuvwxyz")
    };
    ($size:expr) => {
        $crate::nanoid!($size, &"23456789ABCDEFGHJKLMNPQRSTUVWXYZabcdefghijkmnopqrstuvwxyz")
    };
}
