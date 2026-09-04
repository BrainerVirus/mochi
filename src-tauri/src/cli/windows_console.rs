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

/// Pure decision for std-handle repair: a GUI-subsystem process starts
/// with invalid std handles, and `AttachConsole`/`AllocConsole` alone do
/// not repair them — reopen `CONOUT$`/`CONIN$` whenever a console was
/// attached or allocated, never when none is available.
pub fn needs_handle_reopen(action: ConsoleAction) -> bool {
    action != ConsoleAction::Unavailable
}

#[cfg(windows)]
pub fn ensure_console() {
    use windows_sys::Win32::System::Console::{AllocConsole, AttachConsole, ATTACH_PARENT_PROCESS};

    // SAFETY: attaching to the parent console (or allocating a new one)
    // carries no memory-safety contract; BOOL results are checked and every
    // outcome — including both calls failing — returns silently.
    let action = if unsafe { AttachConsole(ATTACH_PARENT_PROCESS) } != 0 {
        ConsoleAction::Attached
    } else if unsafe { AllocConsole() } != 0 {
        ConsoleAction::Allocated
    } else {
        ConsoleAction::Unavailable
    };
    if needs_handle_reopen(action) {
        reopen_std_handles();
    }
}

/// Reopen std handles on the attached/allocated console (`CONOUT$` for
/// stdout/stderr, `CONIN$` for stdin) so `println!` reaches the console
/// on `windows_subsystem = "windows"` release builds. All failures silent.
#[cfg(windows)]
fn reopen_std_handles() {
    use windows_sys::Win32::Foundation::{GENERIC_READ, GENERIC_WRITE, INVALID_HANDLE_VALUE};
    use windows_sys::Win32::Storage::FileSystem::{
        CreateFileW, FILE_SHARE_READ, FILE_SHARE_WRITE, OPEN_EXISTING,
    };
    use windows_sys::Win32::System::Console::{
        SetStdHandle, STD_ERROR_HANDLE, STD_INPUT_HANDLE, STD_OUTPUT_HANDLE,
    };

    // SAFETY: raw-handle syscalls with checked results; a failed open
    // simply skips its `SetStdHandle`, and the function never panics.
    unsafe {
        let out = CreateFileW(
            windows_sys::core::w!("CONOUT$"),
            GENERIC_WRITE,
            FILE_SHARE_READ | FILE_SHARE_WRITE,
            std::ptr::null(),
            OPEN_EXISTING,
            0,
            std::ptr::null_mut(),
        );
        if out != INVALID_HANDLE_VALUE {
            SetStdHandle(STD_OUTPUT_HANDLE, out);
            SetStdHandle(STD_ERROR_HANDLE, out);
        }
        let inp = CreateFileW(
            windows_sys::core::w!("CONIN$"),
            GENERIC_READ,
            FILE_SHARE_READ | FILE_SHARE_WRITE,
            std::ptr::null(),
            OPEN_EXISTING,
            0,
            std::ptr::null_mut(),
        );
        if inp != INVALID_HANDLE_VALUE {
            SetStdHandle(STD_INPUT_HANDLE, inp);
        }
    }
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
    fn attach_takes_precedence_over_alloc() {
        assert_eq!(attach_or_alloc(true, true), ConsoleAction::Attached);
    }

    #[test]
    fn reopen_after_attach() {
        assert!(needs_handle_reopen(ConsoleAction::Attached));
    }

    #[test]
    fn reopen_after_alloc() {
        assert!(needs_handle_reopen(ConsoleAction::Allocated));
    }

    #[test]
    fn no_reopen_when_unavailable() {
        assert!(!needs_handle_reopen(ConsoleAction::Unavailable));
    }

    #[test]
    fn ensure_console_links() {
        ensure_console();
    }
}
