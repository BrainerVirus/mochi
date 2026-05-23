//! Shared test synchronization for process-wide environment mutations.

#[cfg(test)]
use std::sync::Mutex;

#[cfg(test)]
pub static LOCK: Mutex<()> = Mutex::new(());
