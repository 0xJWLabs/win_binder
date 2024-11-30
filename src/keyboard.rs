use std::ptr::null_mut;

use windows::Win32::Foundation::BOOL;
use windows::Win32::Foundation::FALSE;
use windows::Win32::Foundation::LPARAM;
use windows::Win32::Foundation::TRUE;
use windows::Win32::System::Threading::AttachThreadInput;
use windows::Win32::System::Threading::GetCurrentThreadId;
use windows::Win32::UI::Input::KeyboardAndMouse::GetKeyState;
use windows::Win32::UI::Input::KeyboardAndMouse::GetKeyboardLayout;
use windows::Win32::UI::Input::KeyboardAndMouse::GetKeyboardState;
use windows::Win32::UI::Input::KeyboardAndMouse::ToUnicodeEx;
use windows::Win32::UI::Input::KeyboardAndMouse::HKL;
use windows::Win32::UI::Input::KeyboardAndMouse::VK_CAPITAL;
use windows::Win32::UI::Input::KeyboardAndMouse::VK_LSHIFT;
use windows::Win32::UI::Input::KeyboardAndMouse::VK_RSHIFT;
use windows::Win32::UI::Input::KeyboardAndMouse::VK_SHIFT;
use windows::Win32::UI::WindowsAndMessaging::GetForegroundWindow;
use windows::Win32::UI::WindowsAndMessaging::GetWindowThreadProcessId;
use windows::Win32::UI::WindowsAndMessaging::HSHELL_HIGHBIT;

use crate::common::get_code;
use crate::common::get_scan_code;
use crate::common::BYTE;
use crate::common::UINT;
use crate::keycodes::code_from_key;
use crate::win_binder::EventType;
use crate::win_binder::Key;
use crate::win_binder::KeyboardState;

pub struct Keyboard {
    last_code: UINT,
    last_scan_code: UINT,
    last_state: [BYTE; 256],
    last_is_dead: bool,
}

impl Keyboard {
    pub fn new() -> Option<Self> {
        Some(Self {
            last_code: 0,
            last_scan_code: 0,
            last_state: [0; 256],
            last_is_dead: false,
        })
    }

    pub(crate) unsafe fn get_name(&mut self, lpdata: LPARAM) -> Option<String> {
        let code = get_code(lpdata);
        let scan_code = get_scan_code(lpdata);

        self.set_global_state()?;
        self.get_code_name(code, scan_code)
    }

    pub(crate) unsafe fn set_global_state(&mut self) -> Option<()> {
        let mut state = [0_u8; 256];

        let _shift = GetKeyState(VK_SHIFT.0 as i32);
        let current_window_thread_id =
            GetWindowThreadProcessId(GetForegroundWindow(), Some(null_mut()));
        let thread_id = GetCurrentThreadId();

        let status = if AttachThreadInput(thread_id, current_window_thread_id, TRUE) == BOOL(1) {
            let status = GetKeyboardState(&mut state);

            let _ = AttachThreadInput(thread_id, current_window_thread_id, FALSE);
            status
        } else {
            GetKeyboardState(&mut state)
        };

        if status.is_err() {
            return None;
        }

        self.last_state = state;
        Some(())
    }

    pub(crate) unsafe fn get_code_name(&mut self, code: UINT, scan_code: UINT) -> Option<String> {
        let current_window_thread_id =
            GetWindowThreadProcessId(GetForegroundWindow(), Some(null_mut()));
        const BUF_LEN: i32 = 32;
        let mut buff = [0_u16; BUF_LEN as usize];

        let layout = GetKeyboardLayout(current_window_thread_id);
        let len = ToUnicodeEx(code, scan_code, &self.last_state, &mut buff, 0, layout);

        let mut is_dead = false;

        let result = match len {
            0 => None,
            -1 => {
                is_dead = true;
                self.clear_keyboard_buffer(code, scan_code, layout);
                None
            }
            len if len > 0 => String::from_utf16(&buff[..len as usize]).ok(),
            _ => None,
        };

        if self.last_code != 0 && self.last_is_dead {
            buff = [0; 32];
            ToUnicodeEx(
                self.last_code,
                self.last_scan_code,
                &self.last_state,
                &mut buff,
                0,
                layout,
            );
            self.last_code = 0;
        } else {
            self.last_code = code;
            self.last_scan_code = scan_code;
            self.last_is_dead = is_dead;
        }

        result
    }

    unsafe fn clear_keyboard_buffer(&self, code: UINT, scan_code: UINT, layout: HKL) {
        const BUF_LEN: i32 = 32;
        let mut buff = [0_u16; BUF_LEN as usize];
        let state = [0_u8; 256];

        let mut len = -1;
        while len < 0 {
            len = ToUnicodeEx(code, scan_code, &state, &mut buff, 0, layout);
        }
    }
}

impl KeyboardState for Keyboard {
    fn add(&mut self, event_type: &EventType) -> Option<String> {
        match event_type {
            EventType::KeyPress(key) => match key {
                Key::ShiftLeft => {
                    self.last_state[VK_SHIFT.0 as usize] |= HSHELL_HIGHBIT as u8;
                    self.last_state[VK_LSHIFT.0 as usize] |= HSHELL_HIGHBIT as u8;
                    None
                }
                Key::ShiftRight => {
                    self.last_state[VK_SHIFT.0 as usize] |= HSHELL_HIGHBIT as u8;
                    self.last_state[VK_RSHIFT.0 as usize] |= HSHELL_HIGHBIT as u8;
                    None
                }
                Key::CapsLock => {
                    self.last_state[VK_CAPITAL.0 as usize] ^= 1;
                    None
                }
                key => {
                    let code = code_from_key(*key)?;
                    unsafe { self.get_code_name(code.into(), 0) }
                }
            },
            EventType::KeyRelease(key) => match key {
                Key::ShiftLeft => {
                    self.last_state[VK_SHIFT.0 as usize] &= HSHELL_HIGHBIT as u8;
                    self.last_state[VK_LSHIFT.0 as usize] &= HSHELL_HIGHBIT as u8;
                    None
                }
                Key::ShiftRight => {
                    self.last_state[VK_SHIFT.0 as usize] &= HSHELL_HIGHBIT as u8;
                    self.last_state[VK_RSHIFT.0 as usize] &= HSHELL_HIGHBIT as u8;
                    None
                }
                _ => None,
            },
            _ => None,
        }
    }

    fn reset(&mut self) {
        self.last_state[16] = 0;
        self.last_state[20] = 0;
    }
}
