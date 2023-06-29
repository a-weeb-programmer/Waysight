use std::{
    os::fd::AsRawFd,
    sync::{Arc, Mutex},
};

use smithay::{
    delegate_output,
    input::{pointer::CursorImageStatus, Seat, SeatState},
    reexports::{
        calloop::{
            generic::Generic, EventLoop, Interest, LoopHandle, LoopSignal, Mode, PostAction,
        },
        wayland_server::{
            backend::{ClientData, ClientId},
            Display, DisplayHandle,
        },
    },
    wayland::{
        compositor::{CompositorClientState, CompositorState},
        output::OutputManagerState,
        shell::xdg::XdgShellState,
        shm::ShmState,
        socket::ListeningSocketSource,
    },
};
use static_init::lazy::Lazy;

use crate::config::WaysightConfig;

pub static CONFIG: Lazy<WaysightConfig> = Lazy::from_generator(WaysightConfig::load_config);

// Our loop data
pub struct CalloopData<B: Backend + 'static> {
    pub display: Display<Waysight<B>>,
    pub state: Waysight<B>,
}

// Base struct for storing any wayland globals and handling requests
pub struct Waysight<B: Backend + 'static> {
    pub display_handle: DisplayHandle,
    pub cursor_image_status: Arc<Mutex<CursorImageStatus>>,
    pub compositor: CompositorState,
    pub loop_handle: LoopHandle<'static, CalloopData<B>>,
    pub loop_signal: LoopSignal,
    // pub xdg_shell_state: XdgShellState,
    pub shm_state: ShmState,
    pub seat_state: SeatState<Self>,
    pub seat_name: String,
    pub seat: Seat<Self>,
    pub socket_name: String,
    pub output_state: OutputManagerState,
}

#[derive(Default)]
pub struct ClientState {
    pub compositor_client: CompositorClientState,
}
impl ClientData for ClientState {
    fn initialized(&self, client_id: ClientId) {
        tracing::debug!(
            "Client with id {:?} has connected to the compositor.",
            client_id
        )
    }
}

fn init_wl_socket<B: Backend + 'static>(
    handle: &LoopHandle<'static, CalloopData<B>>,
    display: &mut Display<Waysight<B>>,
) -> Option<String> {
    let socket_source = match ListeningSocketSource::new_auto() {
        Ok(socket) => socket,
        Err(err) => {
            tracing::error!("Error when initializing the listening socket: {}", err);
            return None;
        }
    };
    let socket_name = socket_source.socket_name().to_string_lossy().into_owned();
    handle
        .insert_source(socket_source, |stream, _, data| {
            data.display
                .handle()
                .insert_client(stream, Arc::new(ClientState::default()))
                .expect("Failed to insert client into display.");
        })
        .unwrap();

    handle
        .insert_source(
            Generic::new(
                display.backend().poll_fd().as_raw_fd(),
                Interest::READ,
                Mode::Level,
            ),
            |_, _: &mut _, data| {
                data.display.dispatch_clients(&mut data.state).unwrap();
                Ok(PostAction::Continue)
            },
        )
        .unwrap();

    Some(socket_name)
}

impl<B: Backend + 'static> Waysight<B> {
    pub fn new(
        event_loop: &EventLoop<'static, CalloopData<B>>,
        display: &mut Display<Self>,
        backend_data: B,
    ) -> Self {
        let display_handle = display.handle().clone();
        let socket_name = init_wl_socket(&event_loop.handle(), display).unwrap();
        let loop_handle = event_loop.handle();
        let loop_signal = event_loop.get_signal();

        let mut seat_state = SeatState::<Self>::new();
        let seat_name = backend_data.seat_name();
        let seat = seat_state.new_wl_seat(&display_handle, &seat_name);

        let compositor = CompositorState::new::<Self>(&display_handle);
        let shm_state = ShmState::new::<Self>(&display_handle, []);

        let output_state = OutputManagerState::new_with_xdg_output::<Self>(&display_handle);
        let cursor_image_status = Arc::new(Mutex::new(CursorImageStatus::Default));

        let output_state = OutputManagerState::new_with_xdg_output::<Self>(&display_handle);

        Waysight {
            display_handle,
            cursor_image_status,
            compositor,
            loop_handle,
            loop_signal,
            shm_state,
            seat_state,
            seat_name,
            seat,
            socket_name,
            output_state,
        }
    }
}

pub trait Backend {
    fn seat_name(&self) -> String;
    // TODO: add more methods
}

delegate_output!(@<B: Backend + 'static> Waysight<B>);
