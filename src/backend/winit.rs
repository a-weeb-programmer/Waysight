use crate::state::{Backend, CalloopData, Waysight};
use smithay::{
    backend::winit::WinitEvent,
    reexports::{calloop::EventLoop, wayland_server::Display},
};

pub struct WinitBackend {}

impl Backend for WinitBackend {
    fn seat_name(&self) -> String {
        "waysight-seat".to_owned()
    }
}
pub fn initialize() {
    let mut display = Display::<Waysight<WinitBackend>>::new().unwrap();
    let event_loop = EventLoop::<'static, CalloopData<WinitBackend>>::try_new().unwrap();

    let backend_data = WinitBackend {};

    let state = Waysight::new(event_loop, &mut display, backend_data);
}
