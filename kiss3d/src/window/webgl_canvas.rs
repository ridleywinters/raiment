#![allow(unused_results)]

use std::cell::RefCell;
use std::ops::DerefMut;
use std::rc::Rc;
use std::sync::mpsc::Sender;

use crate::event::{Action, Key, Modifiers, MouseButton, TouchAction, WindowEvent};
use image::{GenericImage, Pixel};
use stdweb::web::event as webevent;
use stdweb::web::event::{
    ConcreteEvent, IEvent, IKeyboardEvent, IMouseEvent, ITouchEvent, IUiEvent,
};
use stdweb::web::{
    self, html_element::CanvasElement, EventListenerHandle, IElement, IEventTarget, IHtmlElement,
    IParentNode,
};
use stdweb::{unstable::TryInto, Reference};
use crate::window::{AbstractCanvas, CanvasSetup};

#[derive(Clone, Debug, PartialEq, Eq, ReferenceType)]
#[reference(instance_of = "Event")] // TODO: Better type check.
pub struct WheelEvent(Reference);

impl IEvent for WheelEvent {}
impl IUiEvent for WheelEvent {}
impl IMouseEvent for WheelEvent {}
impl ConcreteEvent for WheelEvent {
    const EVENT_TYPE: &'static str = "wheel";
}

struct WebGLCanvasData {
    canvas: CanvasElement,
    cursor_pos: Option<(f64, f64)>,
    key_states: [Action; Key::Unknown as usize + 1],
    button_states: [Action; MouseButton::Button8 as usize + 1],
    pending_events: Vec<WindowEvent>,
    out_events: Sender<WindowEvent>,
    hidpi_factor: f64,
    mouse_capture_state: MouseCaptureState,
}

#[derive(PartialEq, Eq)]
enum MouseCaptureState {
    NotCaptured,
    Captured,
    OtherElement,
}

/// A canvas based on WebGL and stdweb.
pub struct WebGLCanvas {
    data: Rc<RefCell<WebGLCanvasData>>,
    event_listeners: Vec<EventListenerHandle>,
}

impl Drop for WebGLCanvas {
    fn drop(&mut self) {
        use crate::context::Context;
        // Remove event listeners to free memory:
        let event_listeners = std::mem::replace(&mut self.event_listeners, Vec::new());
        for listener in event_listeners {
            listener.remove();
        }
        // Clear the remnants of the last frame:
        // HACK: This uses the global context.
        let ctxt = Context::get();
        verify!(ctxt.active_texture(Context::TEXTURE0));
        verify!(ctxt.clear_color(1.0, 1.0, 1.0, 1.0));
        verify!(ctxt.clear(Context::COLOR_BUFFER_BIT));
        verify!(ctxt.clear(Context::DEPTH_BUFFER_BIT));
        // TODO: Free other resources such as textures?
    }
}

impl AbstractCanvas for WebGLCanvas {
    fn open(
        _: &str,
        _: bool,
        _: u32,
        _: u32,
        setup: Option<CanvasSetup>,
        out_events: Sender<WindowEvent>,
    ) -> Self {
        fn get_hidpi_factor() -> f64 {
            (js! { return window.devicePixelRatio; })
                .try_into()
                .unwrap()
        }
        let initial_hidpi_factor = get_hidpi_factor();
        let canvas: CanvasElement = web::document()
            .query_selector("#canvas")
            .expect("No canvas found.")
            .unwrap()
            .try_into()
            .unwrap();
        canvas.set_width((canvas.offset_width() as f64 * initial_hidpi_factor) as u32);
        canvas.set_height((canvas.offset_height() as f64 * initial_hidpi_factor) as u32);
        // We set tabIndex to make the canvas focusable to allow keyboard
        // events to be received, but only if it is not already set to any
        // specific values. This is done to keep old code working without
        // changes since the keyboard event listeners are now added to the
        // canvas element instead of the window.
        if !canvas.has_attribute("tabindex") {
            canvas.set_attribute("tabindex", "0");
        }
        let data = Rc::new(RefCell::new(WebGLCanvasData {
            canvas,
            cursor_pos: None,
            key_states: [Action::Release; Key::Unknown as usize + 1],
            button_states: [Action::Release; MouseButton::Button8 as usize + 1],
            pending_events: Vec::new(),
            out_events,
            hidpi_factor: initial_hidpi_factor,
            mouse_capture_state: MouseCaptureState::NotCaptured,
        }));
        let mut event_listeners = Vec::new();

        let edata = data.clone();
        let listener = web::window().add_event_listener(move |_: webevent::ResizeEvent| {
            let mut edata = edata.borrow_mut();
            // Here we update the hidpi factor with the assumption that a resize
            // event will always be triggered whenever window.devicePixelRatio
            // changes. This is the easiest way to detect a change of the hidpi
            // factor.
            let hidpi_factor = get_hidpi_factor();
            edata.hidpi_factor = hidpi_factor;
            let (w, h) = (
                (edata.canvas.offset_width() as f64 * hidpi_factor) as u32,
                (edata.canvas.offset_height() as f64 * hidpi_factor) as u32,
            );
            edata.canvas.set_width(w);
            edata.canvas.set_height(h);
            let _ = edata
                .pending_events
                .push(WindowEvent::FramebufferSize(w, h));
            let _ = edata.pending_events.push(WindowEvent::Size(w, h));
        });
        event_listeners.push(listener);

        let edata = data.clone();
        let listener = web::window().add_event_listener(move |e: webevent::MouseDownEvent| {
            let mut edata = edata.borrow_mut();
            match edata.mouse_capture_state {
                MouseCaptureState::NotCaptured => {
                    if e.target()
                        .map_or(false, |target| target.as_ref() != edata.canvas.as_ref())
                    {
                        // Stop handling mouse events after the mouse is pressed
                        // outside of the canvas.
                        edata.mouse_capture_state = MouseCaptureState::OtherElement;
                        return;
                    }
                }
                MouseCaptureState::Captured => {}
                MouseCaptureState::OtherElement => {
                    return;
                }
            }
            edata.mouse_capture_state = MouseCaptureState::Captured;
            let button = translate_mouse_button(&e);
            let _ = edata.pending_events.push(WindowEvent::MouseButton(
                button,
                Action::Press,
                translate_mouse_modifiers(&e),
            ));
            edata.button_states[button as usize] = Action::Press;
        });
        event_listeners.push(listener);

        let edata = data.clone();
        let listener = web::window().add_event_listener(move |e: webevent::MouseUpEvent| {
            let mut edata = edata.borrow_mut();
            match edata.mouse_capture_state {
                MouseCaptureState::NotCaptured => {
                    // This shouldn't happen but we'll ignore it.
                    return;
                }
                MouseCaptureState::Captured => {}
                MouseCaptureState::OtherElement => {
                    use stdweb::web::event::MouseButton::*;
                    let buttons = e.buttons();
                    if [Left, Wheel, Right, Button4, Button5]
                        .iter()
                        .all(|&button| !buttons.is_down(button))
                    {
                        // Resume handling mouse events after mouse buttons are
                        // released.
                        edata.mouse_capture_state = MouseCaptureState::NotCaptured;
                    }
                    return;
                }
            }
            let button = translate_mouse_button(&e);
            let _ = edata.pending_events.push(WindowEvent::MouseButton(
                button,
                Action::Release,
                translate_mouse_modifiers(&e),
            ));
            edata.button_states[button as usize] = Action::Release;
            if edata
                .button_states
                .iter()
                .all(|&state| state == Action::Release)
            {
                edata.mouse_capture_state = MouseCaptureState::NotCaptured;
            }
        });
        event_listeners.push(listener);

        let edata = data.clone();
        let listener = web::window().add_event_listener(move |e: webevent::MouseMoveEvent| {
            let mut edata = edata.borrow_mut();
            match edata.mouse_capture_state {
                MouseCaptureState::NotCaptured => {
                    if e.target()
                        .map_or(false, |target| target.as_ref() != edata.canvas.as_ref())
                    {
                        // Don't handle hover events outside of the canvas.
                        return;
                    }
                }
                MouseCaptureState::Captured => {}
                MouseCaptureState::OtherElement => {
                    return;
                }
            }
            let hidpi_factor = edata.hidpi_factor;
            let bounding_client_rect = edata.canvas.get_bounding_client_rect();
            let x = (e.client_x() as f64 - bounding_client_rect.get_x()) * hidpi_factor;
            let y = (e.client_y() as f64 - bounding_client_rect.get_y()) * hidpi_factor;
            edata.cursor_pos = Some((x, y));
            let _ = edata.pending_events.push(WindowEvent::CursorPos(
                x,
                y,
                translate_mouse_modifiers(&e),
            ));
        });
        event_listeners.push(listener);

        let edata = data.clone();
        let listener = web::window().add_event_listener(move |e: webevent::TouchStart| {
            let mut edata = edata.borrow_mut();
            let hidpi_factor = edata.hidpi_factor;
            for t in e.changed_touches() {
                let _ = edata.pending_events.push(WindowEvent::Touch(
                    t.identifier() as u64,
                    t.client_x() as f64 * hidpi_factor,
                    t.client_y() as f64 * hidpi_factor,
                    TouchAction::Start,
                    translate_touch_modifiers(&e),
                ));
            }
        });
        event_listeners.push(listener);

        let edata = data.clone();
        let listener = web::window().add_event_listener(move |e: webevent::TouchEnd| {
            let mut edata = edata.borrow_mut();
            let hidpi_factor = edata.hidpi_factor;
            for t in e.changed_touches() {
                let _ = edata.pending_events.push(WindowEvent::Touch(
                    t.identifier() as u64,
                    t.client_x() as f64 * hidpi_factor,
                    t.client_y() as f64 * hidpi_factor,
                    TouchAction::End,
                    translate_touch_modifiers(&e),
                ));
            }
        });
        event_listeners.push(listener);

        let edata = data.clone();
        let listener = web::window().add_event_listener(move |e: webevent::TouchCancel| {
            let mut edata = edata.borrow_mut();
            let hidpi_factor = edata.hidpi_factor;
            for t in e.changed_touches() {
                let _ = edata.pending_events.push(WindowEvent::Touch(
                    t.identifier() as u64,
                    t.client_x() as f64 * hidpi_factor,
                    t.client_y() as f64 * hidpi_factor,
                    TouchAction::Cancel,
                    translate_touch_modifiers(&e),
                ));
            }
        });
        event_listeners.push(listener);

        let edata = data.clone();
        let listener = web::window().add_event_listener(move |e: webevent::TouchMove| {
            let mut edata = edata.borrow_mut();
            let hidpi_factor = edata.hidpi_factor;

            for t in e.changed_touches() {
                edata.cursor_pos = Some((
                    t.client_x() as f64 * hidpi_factor,
                    t.client_y() as f64 * hidpi_factor,
                ));
                let _ = edata.pending_events.push(WindowEvent::Touch(
                    t.identifier() as u64,
                    t.client_x() as f64 * hidpi_factor,
                    t.client_y() as f64 * hidpi_factor,
                    TouchAction::Move,
                    translate_touch_modifiers(&e),
                ));
            }
        });
        event_listeners.push(listener);

        let edata = data.clone();
        let listener = data
            .borrow()
            .canvas
            .add_event_listener(move |e: WheelEvent| {
                let delta_x: f64 = js!(
                    return @{e.as_ref()}.deltaX;
                )
                .try_into()
                .ok()
                .unwrap_or(0.0);
                let delta_y: f64 = js!(
                    return @{e.as_ref()}.deltaY;
                )
                .try_into()
                .ok()
                .unwrap_or(0.0);
                // The values of deltaMode:
                // 0x00 => DOM_DELTA_PIXEL
                // 0x01 => DOM_DELTA_LINE
                // 0x02 => DOM_DELTA_PAGE
                let delta_mode = js!(
                    return @{e.as_ref()}.deltaMode;
                )
                .try_into()
                .ok()
                .unwrap_or(0u32);
                let (delta_x, delta_y) = match delta_mode {
                    // It doesn't really make much sense to scroll a "page" in
                    // case of scrolling the cameras so we treat DOM_DELTA_PAGE
                    // the same way as DOM_DELTA_LINE.
                    0x01 | 0x02 => (delta_x * 10.0, delta_y * 10.0),
                    _ => (delta_x, delta_y),
                };
                let mut edata = edata.borrow_mut();
                let _ = edata.pending_events.push(WindowEvent::Scroll(
                    delta_x / 10.0,
                    -delta_y / 10.0,
                    translate_mouse_modifiers(&e),
                ));
            });
        event_listeners.push(listener);

        let edata = data.clone();
        let listener = data
            .borrow()
            .canvas
            .add_event_listener(move |e: webevent::KeyDownEvent| {
                let mut edata = edata.borrow_mut();
                let key = translate_key(&e);
                let _ = edata.pending_events.push(WindowEvent::Key(
                    key,
                    Action::Press,
                    translate_key_modifiers(&e),
                ));
                edata.key_states[key as usize] = Action::Press;
            });
        event_listeners.push(listener);

        let edata = data.clone();
        let listener = data
            .borrow()
            .canvas
            .add_event_listener(move |e: webevent::KeyUpEvent| {
                let mut edata = edata.borrow_mut();
                let key = translate_key(&e);
                let _ = edata.pending_events.push(WindowEvent::Key(
                    key,
                    Action::Release,
                    translate_key_modifiers(&e),
                ));
                edata.key_states[key as usize] = Action::Release;
            });
        event_listeners.push(listener);

        WebGLCanvas {
            data,
            event_listeners,
        }
    }

    fn render_loop(mut callback: impl FnMut(f64) -> bool + 'static) {
        let _ = web::window().request_animation_frame(move |t| {
            if callback(t) {
                let _ = Self::render_loop(callback);
            }
        });
    }

    fn hidpi_factor(&self) -> f64 {
        self.data.borrow().hidpi_factor
    }

    fn poll_events(&mut self) {
        let mut data_borrow = self.data.borrow_mut();
        let data = data_borrow.deref_mut();

        for e in data.pending_events.drain(..) {
            let _ = data.out_events.send(e);
        }
    }

    fn swap_buffers(&mut self) {
        // Nothing to do.
    }

    fn size(&self) -> (u32, u32) {
        let hidpi_factor = self.hidpi_factor();
        (
            (self.data.borrow().canvas.offset_width() as f64 * hidpi_factor) as u32,
            (self.data.borrow().canvas.offset_height() as f64 * hidpi_factor) as u32,
        )
    }

    fn cursor_pos(&self) -> Option<(f64, f64)> {
        self.data.borrow().cursor_pos
    }

    fn set_title(&mut self, _: &str) {
        // Not supported.
    }

    fn set_icon(&mut self, icon: impl GenericImage<Pixel = impl Pixel<Subpixel = u8>>) {
        // Not supported.
    }

    fn set_cursor_grab(&self, grab: bool) {
        // Not supported.
    }

    fn hide(&mut self) {
        // Not supported.
    }

    fn show(&mut self) {
        // Not supported.
    }

    fn get_mouse_button(&self, button: MouseButton) -> Action {
        self.data.borrow().button_states[button as usize]
    }
    fn get_key(&self, key: Key) -> Action {
        self.data.borrow().key_states[key as usize]
    }
}

fn translate_mouse_modifiers<E: IMouseEvent>(event: &E) -> Modifiers {
    let mut res = Modifiers::empty();
    if event.shift_key() {
        res.insert(Modifiers::Shift)
    }
    if event.ctrl_key() {
        res.insert(Modifiers::Control)
    }
    if event.alt_key() {
        res.insert(Modifiers::Alt)
    }
    if event.meta_key() {
        res.insert(Modifiers::Super)
    }

    res
}

fn translate_touch_modifiers<E: ITouchEvent>(event: &E) -> Modifiers {
    let mut res = Modifiers::empty();
    if event.shift_key() {
        res.insert(Modifiers::Shift)
    }
    if event.ctrl_key() {
        res.insert(Modifiers::Control)
    }
    if event.alt_key() {
        res.insert(Modifiers::Alt)
    }
    if event.meta_key() {
        res.insert(Modifiers::Super)
    }

    res
}

fn translate_key_modifiers<E: IKeyboardEvent>(event: &E) -> Modifiers {
    let mut res = Modifiers::empty();
    if event.shift_key() {
        res.insert(Modifiers::Shift)
    }
    if event.ctrl_key() {
        res.insert(Modifiers::Control)
    }
    if event.alt_key() {
        res.insert(Modifiers::Alt)
    }
    if event.meta_key() {
        res.insert(Modifiers::Super)
    }

    res
}

fn translate_mouse_button<E: IMouseEvent>(event: &E) -> MouseButton {
    match event.button() {
        webevent::MouseButton::Left => MouseButton::Button1,
        webevent::MouseButton::Right => MouseButton::Button2,
        webevent::MouseButton::Wheel => MouseButton::Button3,
        webevent::MouseButton::Button4 => MouseButton::Button4,
        webevent::MouseButton::Button5 => MouseButton::Button5,
    }
}

fn translate_key<E: IKeyboardEvent>(event: &E) -> Key {
    // FIXME: some of thos mapping may not be correct.
    match event.key().as_str() {
        "1" => Key::Key1,
        "2" => Key::Key2,
        "3" => Key::Key3,
        "4" => Key::Key4,
        "5" => Key::Key5,
        "6" => Key::Key6,
        "7" => Key::Key7,
        "8" => Key::Key8,
        "9" => Key::Key9,
        "0" => Key::Key0,
        "A" | "a" => Key::A,
        "B" | "b" => Key::B,
        "C" | "c" => Key::C,
        "D" | "d" => Key::D,
        "E" | "e" => Key::E,
        "F" | "f" => Key::F,
        "G" | "g" => Key::G,
        "H" | "h" => Key::H,
        "I" | "i" => Key::I,
        "J" | "j" => Key::J,
        "K" | "k" => Key::K,
        "L" | "l" => Key::L,
        "M" | "m" => Key::M,
        "N" | "n" => Key::N,
        "O" | "o" => Key::O,
        "P" | "p" => Key::P,
        "Q" | "q" => Key::Q,
        "R" | "r" => Key::R,
        "S" | "s" => Key::S,
        "T" | "t" => Key::T,
        "U" | "u" => Key::U,
        "V" | "v" => Key::V,
        "W" | "w" => Key::W,
        "X" | "x" => Key::X,
        "Y" | "y" => Key::Y,
        "Z" | "z" => Key::Z,
        "Escape" => Key::Escape,
        "F1" => Key::F1,
        "F2" => Key::F2,
        "F3" => Key::F3,
        "F4" => Key::F4,
        "F5" => Key::F5,
        "F6" => Key::F6,
        "F7" => Key::F7,
        "F8" => Key::F8,
        "F9" => Key::F9,
        "F10" => Key::F10,
        "F11" => Key::F11,
        "F12" => Key::F12,
        "F13" => Key::F13,
        "F14" => Key::F14,
        "F15" => Key::F15,
        "F16" => Key::F16,
        "F17" => Key::F17,
        "F18" => Key::F18,
        "F19" => Key::F19,
        "F20" => Key::F20,
        "F21" => Key::F21,
        "F22" => Key::F22,
        "F23" => Key::F23,
        "F24" => Key::F24,
        "Snapshot" => Key::Snapshot,
        "Scroll" => Key::Scroll,
        "Pause" => Key::Pause,
        "Insert" => Key::Insert,
        "Home" => Key::Home,
        "Delete" => Key::Delete,
        "End" => Key::End,
        "PageDown" => Key::PageDown,
        "PageUp" => Key::PageUp,
        "ArrowLeft" => Key::Left,
        "ArrowUp" => Key::Up,
        "ArrowRight" => Key::Right,
        "ArrowDown" => Key::Down,
        "Back" => Key::Back,
        "Return" => Key::Return,
        " " => Key::Space,
        "Compose" => Key::Compose,
        "Caret" => Key::Caret,
        "Numlock" => Key::Numlock,
        "Numpad0" => Key::Numpad0,
        "Numpad1" => Key::Numpad1,
        "Numpad2" => Key::Numpad2,
        "Numpad3" => Key::Numpad3,
        "Numpad4" => Key::Numpad4,
        "Numpad5" => Key::Numpad5,
        "Numpad6" => Key::Numpad6,
        "Numpad7" => Key::Numpad7,
        "Numpad8" => Key::Numpad8,
        "Numpad9" => Key::Numpad9,
        "AbntC1" => Key::AbntC1,
        "AbntC2" => Key::AbntC2,
        "+" => Key::Add,
        "'" => Key::Apostrophe,
        "Apps" => Key::Apps,
        "At" => Key::At,
        "Ax" => Key::Ax,
        "\\" => Key::Backslash,
        "Calculator" => Key::Calculator,
        "Capital" => Key::Capital,
        ":" => Key::Colon,
        "," => Key::Comma,
        "Convert" => Key::Convert,
        "Decimal" => Key::Decimal,
        // "/" => Key::Divide,
        "=" => Key::Equals,
        "Grave" => Key::Grave,
        "Kana" => Key::Kana,
        "Kanji" => Key::Kanji,
        "LAlt" => Key::LAlt,
        "{" => Key::LBracket,
        "LControl" => Key::LControl,
        "LShift" => Key::LShift,
        "LWin" => Key::LWin,
        "Mail" => Key::Mail,
        "MediaSelect" => Key::MediaSelect,
        "MediaStop" => Key::MediaStop,
        "-" => Key::Minus,
        "*" => Key::Multiply,
        "Mute" => Key::Mute,
        "MyComputer" => Key::MyComputer,
        "NavigateForward" => Key::NavigateForward,
        "NavigateBackward" => Key::NavigateBackward,
        "NextTrack" => Key::NextTrack,
        "NoConvert" => Key::NoConvert,
        "NumpadComma" => Key::NumpadComma,
        "NumpadEnter" => Key::NumpadEnter,
        "NumpadEquals" => Key::NumpadEquals,
        "OEM102" => Key::OEM102,
        "." => Key::Period,
        "PlayPause" => Key::PlayPause,
        "Power" => Key::Power,
        "PrevTrack" => Key::PrevTrack,
        "RAlt" => Key::RAlt,
        "}" => Key::RBracket,
        "RControl" => Key::RControl,
        "RShift" => Key::RShift,
        "RWin" => Key::RWin,
        ";" => Key::Semicolon,
        "/" => Key::Slash,
        "Sleep" => Key::Sleep,
        "Stop" => Key::Stop,
        // "-" => Key::Subtract,
        "Sysrq" => Key::Sysrq,
        "Tab" => Key::Tab,
        "_" => Key::Underline,
        "Unlabeled" => Key::Unlabeled,
        "VolumeDown" => Key::VolumeDown,
        "VolumeUp" => Key::VolumeUp,
        "Wake" => Key::Wake,
        "WebBack" => Key::WebBack,
        "WebFavorites" => Key::WebFavorites,
        "WebForward" => Key::WebForward,
        "WebHome" => Key::WebHome,
        "WebRefresh" => Key::WebRefresh,
        "WebSearch" => Key::WebSearch,
        "WebStop" => Key::WebStop,
        "Yen" => Key::Yen,
        "Copy" => Key::Copy,
        "Paste" => Key::Paste,
        "Cut" => Key::Cut,
        _ => Key::Unknown,
    }
}
