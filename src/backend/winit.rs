use std::{borrow::Borrow, time::Duration};

use crate::state::{Backend, CalloopData, Waysight};
use smithay::{
    backend::{
        renderer::glow::GlowRenderer,
        winit::{self, WinitError, WinitEvent, WinitEventLoop, WinitGraphicsBackend},
    },
    output::{Mode, Output, PhysicalProperties, Subpixel},
    reexports::{
        calloop::{
            timer::{TimeoutAction, Timer},
            EventLoop,
        },
        wayland_server::Display,
    },
    utils::{Rectangle, Transform},
};

pub struct WinitBackend {}

impl Backend for WinitBackend {
    fn seat_name(&self) -> String {
        "waysight-seat".to_owned()
    }
}
pub fn initialize() {
    let mut display = Display::<Waysight<WinitBackend>>::new().unwrap();
    let mut event_loop = EventLoop::<'static, CalloopData<WinitBackend>>::try_new().unwrap();

    let backend_data = WinitBackend {};

    let state = Waysight::new(&event_loop, &mut display, backend_data);
    let (mut backend, mut winit_event_loop) = match winit::init::<GlowRenderer>() {
        Ok((backend, winit_event_loop)) => (backend, winit_event_loop),
        Err(err) => {
            tracing::error!("Failure initializing winit backend: {}", err);
            return;
        }
    };

    let mode = Mode {
        size: backend.window_size().physical_size,
        refresh: 60_000,
    };

    let output = Output::new(
        "waysight".to_owned(),
        PhysicalProperties {
            size: (0, 0).into(),
            subpixel: Subpixel::Unknown,
            make: "Smithay".to_owned(),
            model: "Waysight".to_owned(),
        },
    );
    let _global = output.create_global::<Waysight<WinitBackend>>(&state.display_handle);
    output.change_current_state(
        Some(mode),
        Some(Transform::Flipped180),
        None,
        Some((0, 0).into()),
    );
    let mut data = CalloopData { display, state };
    let timer = Timer::immediate();
    data.state
        .loop_handle
        .insert_source(timer, move |_, _, data: &mut _| {
            dispatch_winit_events(data, &mut winit_event_loop, &output, &mut backend);
            TimeoutAction::ToDuration(Duration::from_millis(16))
        })
        .unwrap();

    event_loop
        .run(Duration::from_millis(8), &mut data, move |data| {
            data.display.flush_clients().unwrap();
        })
        .expect("Failure to run event loop");
}

fn dispatch_winit_events(
    data: &mut CalloopData<WinitBackend>,
    winit_event_loop: &mut WinitEventLoop,
    output: &Output,
    backend: &mut WinitGraphicsBackend<GlowRenderer>,
) {
    let ret = winit_event_loop.dispatch_new_events(|event| match event {
        WinitEvent::Input(event) => data.state.parse_input_event_winit(event),
        WinitEvent::Resized {
            size,
            scale_factor: _,
        } => {
            output.change_current_state(
                Some(Mode {
                    refresh: 60_000,
                    size,
                }),
                None,
                None,
                None,
            );
        }
        _ => {}
    });

    backend.window().borrow().set_title("Waysight");

    if let Err(WinitError::WindowClosed) = ret {
        tracing::info!("Closed winit window, stopping the loop");
        data.state.loop_signal.stop();
    }

    let size = backend.window_size().physical_size;
    let damage = Rectangle::from_loc_and_size((0, 0), size);

    backend.bind().unwrap();
    backend.submit(Some(&[damage])).unwrap();
}
