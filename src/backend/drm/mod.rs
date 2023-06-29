mod device;
use crate::state::Backend;

pub struct DrmBackend {}

impl Backend for DrmBackend {
    fn seat_name(&self) -> String {
        todo!()
    }
}

pub fn initialize() {}
