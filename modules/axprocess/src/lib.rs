//! This module provides the process management API for the operating system.
#![feature(stmt_expr_attributes)]
#![cfg_attr(not(test), no_std)]
mod api;
pub use api::*;
mod process;
pub use process::{Process, PID2PC, TID2TASK};

pub mod flags;
pub mod futex;
pub mod link;
mod stdio;

mod fd_manager;
#[cfg(feature = "signal")]
pub mod signal;
