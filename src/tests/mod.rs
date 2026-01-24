#![cfg(test)]

#[macro_use]
mod macros;

mod test_block;
mod test_display;
mod test_flow;
mod test_json;
mod test_misc;
mod test_scalars;
mod test_tags;

#[cfg(feature = "wasm")]
mod test_wasm;
