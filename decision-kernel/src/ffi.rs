//! The page's view of the kernel: five fixed `extern "C"` exports over two byte buffers in linear memory.
//!
//! * `dx_abi()` the layout version of the exports and the value encoding;
//! * `dx_kinds()` the hash of the kind table ([`crate::table_hash`]);
//! * `dx_input(len)` an input buffer of at least `len` bytes (it only ever grows); returns its address;
//! * `dx_decide(kind_index, len)` decides for the kind at that index over the `len`-byte request in the input buffer and
//!   returns the answer's length, or `u32::MAX` when the kind or the request cannot be read;
//! * `dx_output()` the address of the last answer.
//!
//! The export count never grows with a behaviour: a kind is an index into the generated table, dispatched by
//! [`crate::decide`]. Both buffers live until the next call; the memory may grow during a call, so the page takes a
//! fresh view of it each time. The module keeps no state between calls but these two buffers.

#![allow(
    unsafe_code,
    reason = "`#[no_mangle]` is how a `wasm32-unknown-unknown` module exports an `extern \"C\"` item, and no unsafe block exists"
)]

use std::cell::RefCell;

/// The layout version of the exports and the value encoding; the page refuses a kernel that differs.
const ABI: u32 = 2;
/// The length a decision call returns when it has no answer; the page stands down and tells Rust.
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

/// The hash of the kind table this module was built with.
#[no_mangle]
pub extern "C" fn dx_kinds() -> u32 {
    crate::table_hash()
}

/// An input buffer of at least `len` bytes for the next request; returns its address.
#[no_mangle]
pub extern "C" fn dx_input(len: u32) -> *mut u8 {
    INPUT.with_borrow_mut(|buffer| {
        if buffer.len() < len as usize {
            buffer.resize(len as usize, 0);
        }
        buffer.as_mut_ptr()
    })
}

/// The address of the last answer.
#[no_mangle]
pub extern "C" fn dx_output() -> *const u8 {
    OUTPUT.with_borrow(|buffer| buffer.as_ptr())
}

/// Decides for the kind at `kind_index` over the `len`-byte request in the input buffer; returns the answer's length.
#[no_mangle]
pub extern "C" fn dx_decide(kind_index: u32, len: u32) -> u32 {
    let answer = INPUT.with_borrow(|buffer| {
        buffer
            .get(..len as usize)
            .and_then(|request| crate::decide(kind_index as usize, request))
    });
    match answer {
        Some(bytes) => OUTPUT.with_borrow_mut(|out| {
            *out = bytes;
            out.len() as u32
        }),
        None => NONE,
    }
}
