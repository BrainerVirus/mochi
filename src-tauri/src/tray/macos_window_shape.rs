use std::ffi::c_void;
use std::ptr::NonNull;
use std::sync::atomic::{AtomicBool, Ordering};

use objc2_app_kit::{NSView, NSWindow};
use objc2_core_foundation::CGFloat;
use objc2_quartz_core::CALayer;

static TRAY_PANEL_SHAPE_APPLIED: AtomicBool = AtomicBool::new(false);

/// Clips the tray popover `NSWindow` content view to match frontend `--radius-tray-panel`.
pub fn clip_window_content_view(ns_window: *mut c_void, radius: f64) {
    if ns_window.is_null() || TRAY_PANEL_SHAPE_APPLIED.swap(true, Ordering::AcqRel) {
        return;
    }

    unsafe {
        let Some(window) = NonNull::new(ns_window.cast::<NSWindow>()) else {
            TRAY_PANEL_SHAPE_APPLIED.store(false, Ordering::Release);
            return;
        };
        let window = window.as_ref();
        let Some(content_view) = window.contentView() else {
            TRAY_PANEL_SHAPE_APPLIED.store(false, Ordering::Release);
            return;
        };

        clip_content_view_root(&content_view, radius);
    }
}

fn clip_content_view_root(view: &NSView, radius: f64) {
    view.setWantsLayer(true);
    if let Some(layer) = view.layer() {
        set_layer_corner_radius(&layer, radius);
    }
}

fn set_layer_corner_radius(layer: &CALayer, radius: f64) {
    layer.setCornerRadius(radius as CGFloat);
    layer.setMasksToBounds(true);
}
