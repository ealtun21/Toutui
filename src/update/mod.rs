//! The update of the program.
//!
//! The program receives the archive of its own target, it compares the sum,
//! and it moves the new binary on to the old binary. The program runs no file
//! that it receives. See T-21.

pub mod attest;
pub mod install;
pub mod release;
