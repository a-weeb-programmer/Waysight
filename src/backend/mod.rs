use std::env;

pub mod drm;
pub mod winit;

pub fn backend_init_from_name(name: &str) {
    match name {
        "drm" => drm::initialize(),
        "winit" => winit::initialize(),
        _ => {
            tracing::error!("Unknown backend");
        }
    }
}

pub fn backend_autoinit() {
    if env::var("WAYLAND_DISPLAY").is_ok() || env::var("DISPLAY").is_ok() {
        backend_init_from_name("winit");
    } else {
        backend_init_from_name("drm");
    }
}
