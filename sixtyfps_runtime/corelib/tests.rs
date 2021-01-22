/* LICENSE BEGIN
    This file is part of the SixtyFPS Project -- https://sixtyfps.io
    Copyright (c) 2020 Olivier Goffart <olivier.goffart@sixtyfps.io>
    Copyright (c) 2020 Simon Hausmann <simon.hausmann@sixtyfps.io>

    SPDX-License-Identifier: GPL-3.0-only
    This file is also available under commercial licensing terms.
    Please contact info@sixtyfps.io for more information.
LICENSE END */
//! Functions usefull for testing
#![warn(missing_docs)]

use crate::input::{KeyEvent, KeyboardModifiers, MouseEvent, MouseEventType};
use crate::window::ComponentWindow;
use crate::SharedString;

/// SixtyFPS animations do not use real time, but use a mocked time.
/// Normally, the event loop update the time of the animation using
/// real time, but in tests, it is more convinient to use the fake time.
/// This function will add some milliseconds to the fake time
#[no_mangle]
pub extern "C" fn sixtyfps_mock_elapsed_time(time_in_ms: u64) {
    crate::animations::CURRENT_ANIMATION_DRIVER.with(|driver| {
        let mut tick = driver.current_tick();
        tick += instant::Duration::from_millis(time_in_ms);
        driver.update_animations(tick)
    })
}

/// Simulate a click on a position within the component.
#[no_mangle]
pub extern "C" fn sixtyfps_send_mouse_click(
    component: &crate::component::ComponentRc,
    x: f32,
    y: f32,
    window: &ComponentWindow,
) {
    let mut state = crate::input::MouseInputState::default();
    vtable::VRc::borrow_pin(component).as_ref().apply_layout(window.0.get_geometry());

    let pos = euclid::point2(x, y);

    state = crate::input::process_mouse_input(
        component.clone(),
        MouseEvent { pos, what: MouseEventType::MouseMoved },
        window,
        state,
    );
    state = crate::input::process_mouse_input(
        component.clone(),
        MouseEvent { pos, what: MouseEventType::MousePressed },
        window,
        state,
    );
    sixtyfps_mock_elapsed_time(50);
    crate::input::process_mouse_input(
        component.clone(),
        MouseEvent { pos, what: MouseEventType::MouseReleased },
        window,
        state,
    );
}

/// Simulate a character input event.
#[no_mangle]
pub extern "C" fn send_keyboard_string_sequence(
    sequence: &crate::SharedString,
    modifiers: KeyboardModifiers,
    window: &ComponentWindow,
) {
    for ch in sequence.chars() {
        let mut modifiers = modifiers;
        if ch.is_ascii_uppercase() {
            modifiers.shift = true;
        }
        let text: SharedString = ch.to_string().into();

        window.process_key_input(&KeyEvent::KeyPressed { text: text.clone(), modifiers });
        window.process_key_input(&KeyEvent::KeyReleased { text, modifiers });
    }
}
