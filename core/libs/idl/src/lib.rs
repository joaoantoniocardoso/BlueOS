#![no_std]

extern crate alloc;

pub mod cdr;
pub mod error;
pub mod message;

pub use error::Error;
pub use message::{CdrStruct, Message};

mod generated {
    include!(concat!(env!("OUT_DIR"), "/generated_mod.rs"));
}

pub use generated::{msg, schema};
