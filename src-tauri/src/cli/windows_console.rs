//! Windows console attach/alloc so CLI output works in release builds.
//!
//! Release Windows builds set `windows_subsystem`, which detaches the
//! process from the parent console. [`ensure_console`] re-attaches (or
//! allocates a fallback console) before CLI output. Never panics.

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ConsoleAction {
    Attached,
    Allocated,
    Unavailable,
}

/// Pure decision table for console setup: prefer the parent console,
/// fall back to allocating one, otherwise report unavailable.
pub fn attach_or_alloc(attached: bool, allocated: bool) -> ConsoleAction {
    if attached {
        ConsoleAction::Attached
    } else if allocated {
        ConsoleAction::Allocated
    } else {
        ConsoleAction::Unavailable
    }
}

#[cfg(windows)]
pub fn ensure_console() {
    use windows_sys::Win32::System::Console::{AllocConsole, AttachConsole, ATTACH_PARENT_PROCESS};

    // SAFETY: attaching to the parent console (or allocating a new one)
    // carries no memory-safety contract; BOOL results are checked and every
    // outcome — including both calls failing — returns silently.
    if unsafe { AttachConsole(ATTACH_PARENT_PROCESS) } != 0 {
        return;
    }
    let _ = unsafe { AllocConsole() };
}

#[cfg(not(windows))]
pub fn ensure_console() {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn attach_success_skips_alloc() {
        assert_eq!(attach_or_alloc(true, false), ConsoleAction::Attached);
    }

    #[test]
    fn alloc_fallback_when_no_parent() {
        assert_eq!(attach_or_alloc(false, true), ConsoleAction::Allocated);
    }

    #[test]
    fn both_failures_unavailable() {
        assert_eq!(attach_or_alloc(false, false), ConsoleAction::Unavailable);
    }

    #[test]
    fn ensure_console_links() {
        ensure_console();
    }
}
