use std::{os::fd::BorrowedFd, sync::Arc};

use smithay::{
    input::SeatState,
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
        shell::xdg::XdgShellState,
        shm::ShmState,
        socket::ListeningSocketSource,
    },
};

// Our loop data
pub struct CalloopData<B: Backend + 'static> {
    display: Display<Waysight<B>>,
    state: Waysight<B>,
}

// Base struct for storing any wayland globals and handling requests
pub struct Waysight<B: Backend + 'static> {
    pub display_handle: DisplayHandle,
    pub compositor: CompositorState,
    pub loop_handle: LoopHandle<'static, CalloopData<B>>,
    pub loop_signal: LoopSignal,
    pub xdg_shell_state: XdgShellState,
    pub shm_state: ShmState,
    pub seat_state: SeatState<Self>,
    pub seat_name: String,
    pub socket_name: String,
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
    display: &'static mut Display<Waysight<B>>,
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
            Generic::new(display.backend().poll_fd(), Interest::READ, Mode::Level),
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
        event_loop: EventLoop<'static, CalloopData<B>>,
        display: &'static mut Display<Waysight<B>>,
        backend_data: B,
    ) {
        let display_handle = display.handle().clone();
        let socket_name = init_wl_socket(&event_loop.handle(), display);
        let loop_handle = event_loop.handle();
        let loop_signal = event_loop.get_signal();
    }
}

pub trait Backend {
    fn initialize();
    // TODO: add more methods
}
