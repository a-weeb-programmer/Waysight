use smithay::{
    backend::renderer::utils,
    delegate_compositor, delegate_shm,
    reexports::wayland_server::{
        protocol::{wl_buffer::WlBuffer, wl_surface::WlSurface},
        Client,
    },
    wayland::{
        buffer::BufferHandler,
        compositor::{self, CompositorHandler, CompositorState},
        shm::{ShmHandler, ShmState},
    },
};

use crate::state::{Backend, ClientState, Waysight};

impl<B: Backend + 'static> CompositorHandler for Waysight<B> {
    fn compositor_state(&mut self) -> &mut CompositorState {
        &mut self.compositor
    }

    fn client_compositor_state<'a>(
        &self,
        client: &'a Client,
    ) -> &'a smithay::wayland::compositor::CompositorClientState {
        &client.get_data::<ClientState>().unwrap().compositor_client
    }

    fn commit(&mut self, surface: &WlSurface) {
        utils::on_commit_buffer_handler::<Self>(surface);
        if !compositor::is_sync_subsurface(surface) {
            let mut root = surface.clone();
            while let Some(parent) = compositor::get_parent(&root) {
                root = parent;
            }
        }
    }
}

impl<B: Backend + 'static> ShmHandler for Waysight<B> {
    fn shm_state(&self) -> &ShmState {
        &self.shm_state
    }
}

impl<B: Backend + 'static> BufferHandler for Waysight<B> {
    fn buffer_destroyed(&mut self, _buffer: &WlBuffer) {}
}

delegate_compositor!(@<B: Backend + 'static> Waysight<B>);
delegate_shm!(@<B: Backend + 'static> Waysight<B>);
