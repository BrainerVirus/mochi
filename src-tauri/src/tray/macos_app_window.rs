use std::ffi::c_void;
use std::ptr::NonNull;

use objc2::{MainThreadMarker, MainThreadOnly};
use objc2_app_kit::{NSToolbar, NSWindow, NSWindowTitleVisibility, NSWindowToolbarStyle};
use objc2_foundation::NSString;

/// Centers the native title in dedicated app windows (settings, about, update).
///
/// Attaching an empty toolbar makes macOS lay out the title like a standard app window
/// instead of left-aligning it under full-size content view.
pub fn configure_centered_titlebar(ns_window: *mut c_void) {
    if ns_window.is_null() {
        return;
    }

    let mtm = unsafe { MainThreadMarker::new_unchecked() };

    unsafe {
        let Some(window) = NonNull::new(ns_window.cast::<NSWindow>()) else {
            return;
        };
        let window = window.as_ref();

        let identifier = NSString::from_str("MochiAppWindowToolbar");
        let toolbar = NSToolbar::initWithIdentifier(NSToolbar::alloc(mtm), &identifier);
        window.setToolbar(Some(&toolbar));
        window.setTitleVisibility(NSWindowTitleVisibility::Visible);
        window.setToolbarStyle(NSWindowToolbarStyle::Unified);
    }
}
