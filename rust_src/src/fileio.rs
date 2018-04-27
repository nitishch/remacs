//! Functions to deal with files
use std::path;

use remacs_macros::lisp_fn;
use remacs_sys::{Vfile_name_handler_alist, Vinhibit_file_name_handlers,
                 Vinhibit_file_name_operation};
use remacs_sys::{Qoperations};
use remacs_sys::{fast_string_match, maybe_quit};

use lisp::{LispCons, LispObject};
use lisp::defsubr;
use lists::{get, memq};
use multibyte::LispStringRef;

/// Return non-nil if NAME ends with a directory separator character.
#[lisp_fn]
pub fn directory_name_p(name: LispStringRef) -> bool {
    if name.len_bytes() == 0 {
        return false;
    }

    let b = name.byte_at(name.len_bytes() - 1);
    b as char == path::MAIN_SEPARATOR
}

/// Return FILENAME's handler function for OPERATION, if it has one.
/// Otherwise, return nil.
/// A file name is handled if one of the regular expressions in
/// `file-name-handler-alist' matches it.
///
/// If OPERATION equals `inhibit-file-name-operation', then ignore
/// any handlers that are members of `inhibit-file-name-handlers',
/// but still do run any other handlers.  This lets handlers
/// use the standard functions without calling themselves recursively.
#[lisp_fn]
pub fn find_file_name_handler(name: LispStringRef, operation: LispObject) -> LispObject {
    let mut inhibit = false;
    let mut result = LispObject::from_bool(false);
    unsafe{
        if operation.eq(LispObject::from_raw(Vinhibit_file_name_operation)) {
            //This operation is a candidate for inhibition
            inhibit = true;
        }
    }

    let mut current_pos = -1;
    for elt in LispObject::from_raw(unsafe{Vfile_name_handler_alist}).iter_cars() {
        if let Some(pair) = Option::<LispCons>::from(elt) {
            let regex = pair.car();
            let handler = pair.cdr();
            let mut allowed_operation = true;
            if let Some(handler_symbol) = handler.as_symbol() {
                let operations = get(handler_symbol, LispObject::from_raw(Qoperations));
                if !operations.is_nil() {
                    allowed_operation = memq(operation, operations).is_nil()
                }
            }
            if regex.is_string() && allowed_operation {
                let match_pos = unsafe { fast_string_match(regex.to_raw(), LispObject::from(name).to_raw()) };
                if match_pos > current_pos {
                    //This is a better match
                    if !inhibit || memq(handler, LispObject::from_raw(unsafe{Vinhibit_file_name_handlers})).is_nil() {
                        result = handler;
                        current_pos = match_pos;
                    }
                }
            }
        }

        unsafe {
            maybe_quit();
        };
    }

    return result;
}

include!(concat!(env!("OUT_DIR"), "/fileio_exports.rs"));
