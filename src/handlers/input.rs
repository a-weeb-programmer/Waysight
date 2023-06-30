use smithay::{
    backend::{
        input::{Device, DeviceCapability, Event, InputEvent, KeyState, KeyboardKeyEvent},
        libinput::LibinputInputBackend,
        winit::WinitInput,
    },
    input::keyboard::{keysyms, FilterResult, KeysymHandle, ModifiersState, XkbConfig},
    utils::SERIAL_COUNTER,
};

use crate::{
    backend::{drm::DrmBackend, winit::WinitBackend},
    state::{Waysight, CONFIG},
};

impl Waysight<DrmBackend> {
    pub fn parse_input_event(&mut self, event: InputEvent<LibinputInputBackend>) {
        match event {
            InputEvent::DeviceAdded { device } => {
                if Device::has_capability(&device, DeviceCapability::Keyboard) {
                    let xkb_config = XkbConfig {
                        layout: &CONFIG.input.keyboard_layout,
                        variant: &CONFIG.input.layout_variant,
                        ..XkbConfig::default()
                    };
                    self.seat
                        .add_keyboard(xkb_config, 200, 25)
                        .expect("Failure adding keyboard");
                }
            }
            InputEvent::Keyboard { event } => {
                let keyboard = match self.seat.get_keyboard() {
                    Some(kb) => kb,
                    None => return,
                };

                keyboard.input(
                    self,
                    event.key_code(),
                    KeyState::Pressed,
                    SERIAL_COUNTER.next_serial(),
                    Event::time_msec(&event),
                    |state, modifier_state, key_code| {
                        if modifier_state.alt && key_code.modified_sym() == keysyms::KEY_e {
                            state.loop_signal.stop();
                        }
                        FilterResult::<i32>::Forward
                    },
                );
            }
            _ => {}
        }
    }
}

impl Waysight<WinitBackend> {
    fn on_keyboard_input<T>(
        &mut self,
        modifier_state: &ModifiersState,
        keysym: KeysymHandle<'_>,
    ) -> FilterResult<T> {
        if modifier_state.alt && keysym.modified_sym() == keysyms::KEY_o {
            self.loop_signal.stop();
        }
        FilterResult::Forward
    }
    pub fn parse_input_event_winit(&mut self, event: InputEvent<WinitInput>) {
        match event {
            InputEvent::DeviceAdded { device } => {
                if Device::has_capability(&device, DeviceCapability::Keyboard) {
                    let xkb_config = XkbConfig {
                        layout: &CONFIG.input.keyboard_layout,
                        variant: &CONFIG.input.layout_variant,
                        ..XkbConfig::default()
                    };
                    self.seat
                        .add_keyboard(xkb_config, 200, 25)
                        .expect("Failure adding keyboard");
                }
            }
            InputEvent::Keyboard { event } => {
                let keyboard = match self.seat.get_keyboard() {
                    Some(kb) => kb,
                    None => return,
                };

                keyboard.input(
                    self,
                    event.key_code(),
                    event.state(),
                    SERIAL_COUNTER.next_serial(),
                    Event::time_msec(&event),
                    |state, modifier_state, keysym| {
                        state.on_keyboard_input::<i32>(modifier_state, keysym)
                    },
                );
            }
            _ => {}
        }
    }
}
