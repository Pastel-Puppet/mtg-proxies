#![no_std]
extern crate alloc;

pub mod generate_proxies;
mod logging;
pub mod printings;
mod user_options;

use core::{fmt::Display, arch::wasm32::unreachable};
use alloc::string::ToString;
use log::error;
use wasm_bindgen::prelude::*;

use crate::logging::WasmLogger;

#[global_allocator]
static ALLOCATOR: talc::TalckWasm = unsafe { talc::TalckWasm::new_global() };

#[panic_handler]
fn panic(_panic: &core::panic::PanicInfo<'_>) -> ! {
    unreachable()
}

fn rust_error_to_js<T: Display>(error: T) -> JsValue {
    error!("{error}");
    JsValue::from_str(&error.to_string())
}

// Called when the Wasm module is instantiated
#[wasm_bindgen(start)]
fn initialise() -> Result<(), JsValue> {
    log::set_max_level(log::LevelFilter::Debug);
    log::set_logger(&WasmLogger {}).map_err(rust_error_to_js)?;

    Ok(())
}