use windows::Win32::Foundation::HWND;
use windows::Win32::Foundation::LPARAM;
use windows::Win32::Foundation::LRESULT;
use windows::Win32::Foundation::WPARAM;
use windows::Win32::UI::WindowsAndMessaging::CallNextHookEx;
use windows::Win32::UI::WindowsAndMessaging::GetMessageA;
use windows::Win32::UI::WindowsAndMessaging::HC_ACTION;

use crate::common::convert;
use crate::common::set_key_hook;
use crate::common::set_mouse_hook;
use crate::common::HookError;
use crate::common::HOOK;
use crate::common::KEYBOARD;
use crate::win_binder::Event;
use crate::win_binder::EventType;
use crate::win_binder::ListenError;
use std::os::raw::c_int;
use std::ptr::null_mut;
use std::time::SystemTime;

static mut GLOBAL_CALLBACK: Option<Box<dyn FnMut(Event)>> = None;

impl From<HookError> for ListenError {
    fn from(error: HookError) -> Self {
        match error {
            HookError::Mouse(code) => ListenError::MouseHookError(code),
            HookError::Key(code) => ListenError::KeyHookError(code),
        }
    }
}

#[allow(static_mut_refs)]
unsafe extern "system" fn raw_callback(code: c_int, param: WPARAM, lpdata: LPARAM) -> LRESULT {
    if (code as u32) == HC_ACTION {
        let opt = convert(param, lpdata);
        if let Some(event_type) = opt {
            let name = match &event_type {
                EventType::KeyPress(_key) => match (*KEYBOARD).lock() {
                    Ok(mut keyboard) => keyboard.get_name(lpdata),
                    Err(_) => None,
                },
                _ => None,
            };
            let event = Event {
                event_type,
                time: SystemTime::now(),
                name,
            };

            unsafe {
                if let Some(ref mut callback) = &mut GLOBAL_CALLBACK {
                    callback(event); // Call the closure
                }
            }
        }
    }

    CallNextHookEx(HOOK, code, param, lpdata)
}

pub fn listen<T>(callback: T) -> Result<(), ListenError>
where
    T: FnMut(Event) + 'static,
{
    unsafe {
        GLOBAL_CALLBACK = Some(Box::new(callback));
        set_key_hook(raw_callback)?;
        set_mouse_hook(raw_callback)?;

        let _ = GetMessageA(null_mut(), HWND(null_mut()), 0, 0);
    }

    Ok(())
}
