#[allow(clippy::upper_case_acronyms)]
mod common;
mod display;
#[cfg(feature = "unstable_grab")]
mod grab;
mod keyboard;
mod keycodes;
mod listen;
mod simulate;
mod win_binder;

pub use crate::display::display_size;
#[cfg(feature = "unstable_grab")]
pub use crate::grab::grab;
pub use crate::keyboard::Keyboard;
pub use crate::listen::listen;
pub use crate::simulate::simulate;
pub use crate::win_binder::Button;
pub use crate::win_binder::DisplayError;
pub use crate::win_binder::Event;
pub use crate::win_binder::EventType;
pub use crate::win_binder::GrabCallback;
pub use crate::win_binder::GrabError;
pub use crate::win_binder::Key;
pub use crate::win_binder::KeyboardState;
pub use crate::win_binder::ListenError;
pub use crate::win_binder::SimulateError;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_keyboard_state() {
        // S
        let mut keyboard = Keyboard::new().unwrap();
        let char_s = keyboard.add(&EventType::KeyPress(Key::KeyS)).unwrap();
        assert_eq!(
            char_s,
            "s".to_string(),
            "This test should pass only on Qwerty layout !"
        );
        let n = keyboard.add(&EventType::KeyRelease(Key::KeyS));
        assert_eq!(n, None);

        // Shift + S
        keyboard.add(&EventType::KeyPress(Key::ShiftLeft));
        let char_s = keyboard.add(&EventType::KeyPress(Key::KeyS)).unwrap();
        assert_eq!(char_s, "S".to_string());
        let n = keyboard.add(&EventType::KeyRelease(Key::KeyS));
        assert_eq!(n, None);
        keyboard.add(&EventType::KeyRelease(Key::ShiftLeft));

        // Reset
        keyboard.add(&EventType::KeyPress(Key::ShiftLeft));
        keyboard.reset();
        let char_s = keyboard.add(&EventType::KeyPress(Key::KeyS)).unwrap();
        assert_eq!(char_s, "s".to_string());
        let n = keyboard.add(&EventType::KeyRelease(Key::KeyS));
        assert_eq!(n, None);
        keyboard.add(&EventType::KeyRelease(Key::ShiftLeft));

        // CapsLock
        let char_c = keyboard.add(&EventType::KeyPress(Key::KeyC)).unwrap();
        assert_eq!(char_c, "c".to_string());
        keyboard.add(&EventType::KeyPress(Key::CapsLock));
        keyboard.add(&EventType::KeyRelease(Key::CapsLock));
        let char_c = keyboard.add(&EventType::KeyPress(Key::KeyC)).unwrap();
        assert_eq!(char_c, "C".to_string());
        let n = keyboard.add(&EventType::KeyRelease(Key::KeyS));
        assert_eq!(n, None);
        keyboard.add(&EventType::KeyPress(Key::CapsLock));
        keyboard.add(&EventType::KeyRelease(Key::CapsLock));
        let char_c = keyboard.add(&EventType::KeyPress(Key::KeyC)).unwrap();
        assert_eq!(char_c, "c".to_string());
        let n = keyboard.add(&EventType::KeyRelease(Key::KeyS));
        assert_eq!(n, None);

        // UsIntl layout required
        // let n = keyboard.add(&EventType::KeyPress(Key::Quote));
        // assert_eq!(n, Some("".to_string()));
        // let m = keyboard.add(&EventType::KeyRelease(Key::Quote));
        // assert_eq!(m, None);
        // let e = keyboard.add(&EventType::KeyPress(Key::KeyE)).unwrap();
        // assert_eq!(e, "é".to_string());
        // keyboard.add(&EventType::KeyRelease(Key::KeyE));
    }
}
