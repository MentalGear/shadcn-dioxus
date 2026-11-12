//! The page's view of the kernel: plain `extern "C"` exports over two byte buffers in linear memory.
//!
//! The page asks `dx_input(len)` for a buffer of `len` bytes, writes the request there, calls
//! `dx_decide_<kind>(len)`, and reads the answer from `dx_output()` for the returned length. Both buffers live
//! until the next call; the memory may grow during a call, so the page takes a fresh view of it each time.

#![allow(
    unsafe_code,
    reason = "`#[no_mangle]` is how a `wasm32-unknown-unknown` module exports a plain function; no unsafe block exists"
)]

use std::cell::RefCell;

/// The layout version of the exports and the value encoding; the page refuses a kernel that differs.
const ABI: u32 = 1;
/// The length a decision call returns when it has no answer (malformed request); the page stands down.
const NONE: u32 = u32::MAX;

thread_local! {
    static INPUT: RefCell<Vec<u8>> = const { RefCell::new(Vec::new()) };
    static OUTPUT: RefCell<Vec<u8>> = const { RefCell::new(Vec::new()) };
}

/// The ABI version.
#[no_mangle]
pub extern "C" fn dx_abi() -> u32 {
    ABI
}

/// A buffer of `len` bytes for the next request; returns its address.
#[no_mangle]
pub extern "C" fn dx_input(len: u32) -> *mut u8 {
    INPUT.with_borrow_mut(|buffer| {
        buffer.clear();
        buffer.resize(len as usize, 0);
        buffer.as_mut_ptr()
    })
}

/// The address of the last answer.
#[no_mangle]
pub extern "C" fn dx_output() -> *const u8 {
    OUTPUT.with_borrow(|buffer| buffer.as_ptr())
}

/// Decides for the fixture over the `len`-byte request in the input buffer; returns the answer's length.
#[no_mangle]
pub extern "C" fn dx_decide_fixture(len: u32) -> u32 {
    let answer = INPUT.with_borrow(|buffer| {
        buffer
            .get(..len as usize)
            .and_then(crate::wire::decide_fixture)
    });
    match answer {
        Some(bytes) => OUTPUT.with_borrow_mut(|out| {
            *out = bytes;
            out.len() as u32
        }),
        None => NONE,
    }
}
