#![no_std]
#![allow(internal_features)]  // TODO
#![feature(lang_items,naked_functions)]

pub mod entry_point;
pub mod lang_items;

#[cfg(target_arch = "arm")]
#[path = "arm_syscalls.rs"]
pub mod syscalls;
