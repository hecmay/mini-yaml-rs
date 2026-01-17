#![cfg(test)]

#[macro_use]
mod macros;

mod tests;

#[cfg(feature = "wasm")]
mod wasm_tests;
