#![cfg(test)]

#[macro_use]
mod macros;

mod test_scalars;
mod test_flow;
mod test_block;
mod test_tags;
mod test_json;
mod test_display;
mod test_misc;

#[cfg(feature = "wasm")]
mod test_wasm;
