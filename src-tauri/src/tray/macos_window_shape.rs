use std::ffi::c_void;
use std::ptr::NonNull;

use objc2_app_kit::{NSView, NSWindow};
use objc2_core_foundation::CGFloat;
use objc2_quartz_core::CALayer;

/// Clips the tray popover `NSWindow` content view to match frontend `--radius-tray-panel`.
pub fn clip_window_content_view(ns_window: *mut c_void, radius: f64) {
    if ns_window.is_null() {
        return;
    }

    unsafe {
        let window = NonNull::new(ns_window.cast::<NSWindow>()).expect("ns window ptr");
        let window = window.as_ref();
        let Some(content_view) = window.contentView() else {
            return;
        };

        clip_view_tree(&content_view, radius);
    }
}

fn clip_view_tree(view: &NSView, radius: f64) {
    view.setWantsLayer(true);
    if let Some(layer) = view.layer() {
        set_layer_corner_radius(&layer, radius);
    }

    for subview in view.subviews().iter() {
        clip_view_tree(&subview, radius);
    }
}

fn set_layer_corner_radius(layer: &CALayer, radius: f64) {
    layer.setCornerRadius(radius as CGFloat);
    layer.setMasksToBounds(true);
}
