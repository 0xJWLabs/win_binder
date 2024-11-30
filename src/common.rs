use crate::keyboard::Keyboard;
use crate::keycodes::key_from_code;
use crate::win_binder::Button;
use crate::win_binder::EventType;
use std::os::raw::c_int;
use std::os::raw::c_long;
use std::os::raw::c_short;
use std::os::raw::c_uchar;
use std::os::raw::c_uint;
use std::os::raw::c_ushort;
use std::ptr::null_mut;
use std::sync::LazyLock;
use std::sync::Mutex;
use windows::Win32::Foundation::GetLastError;
use windows::Win32::Foundation::HINSTANCE;
use windows::Win32::Foundation::LPARAM;
use windows::Win32::Foundation::LRESULT;
use windows::Win32::Foundation::WPARAM;
use windows::Win32::UI::WindowsAndMessaging::SetWindowsHookExA;
use windows::Win32::UI::WindowsAndMessaging::HHOOK;
use windows::Win32::UI::WindowsAndMessaging::KBDLLHOOKSTRUCT;
use windows::Win32::UI::WindowsAndMessaging::MSLLHOOKSTRUCT;
use windows::Win32::UI::WindowsAndMessaging::WHEEL_DELTA;
use windows::Win32::UI::WindowsAndMessaging::WH_KEYBOARD_LL;
use windows::Win32::UI::WindowsAndMessaging::WH_MOUSE_LL;
use windows::Win32::UI::WindowsAndMessaging::WM_KEYDOWN;
use windows::Win32::UI::WindowsAndMessaging::WM_KEYUP;
use windows::Win32::UI::WindowsAndMessaging::WM_LBUTTONDOWN;
use windows::Win32::UI::WindowsAndMessaging::WM_LBUTTONUP;
use windows::Win32::UI::WindowsAndMessaging::WM_MBUTTONDOWN;
use windows::Win32::UI::WindowsAndMessaging::WM_MBUTTONUP;
use windows::Win32::UI::WindowsAndMessaging::WM_MOUSEHWHEEL;
use windows::Win32::UI::WindowsAndMessaging::WM_MOUSEMOVE;
use windows::Win32::UI::WindowsAndMessaging::WM_MOUSEWHEEL;
use windows::Win32::UI::WindowsAndMessaging::WM_RBUTTONDOWN;
use windows::Win32::UI::WindowsAndMessaging::WM_RBUTTONUP;
use windows::Win32::UI::WindowsAndMessaging::WM_SYSKEYDOWN;
use windows::Win32::UI::WindowsAndMessaging::WM_SYSKEYUP;
use windows::Win32::UI::WindowsAndMessaging::WM_XBUTTONDOWN;
use windows::Win32::UI::WindowsAndMessaging::WM_XBUTTONUP;

pub type UINT = c_uint;
pub type LONG = c_long;
pub type DWORD = c_uint;
pub type BYTE = c_uchar;
pub type WORD = c_ushort;

pub(crate) static KEYBOARD: LazyLock<Mutex<Keyboard>> =
    LazyLock::new(|| Mutex::new(Keyboard::new().expect("Failed to create Keyboard")));

pub static mut HOOK: HHOOK = HHOOK(null_mut());

#[inline]
#[allow(non_snake_case)]
pub fn HIWORD(l: DWORD) -> WORD {
    ((l >> 16) & 0xffff) as WORD
}

pub unsafe fn get_code(lpdata: LPARAM) -> DWORD {
    let kb = *(lpdata.0 as *const KBDLLHOOKSTRUCT);
    kb.vkCode
}

pub unsafe fn get_scan_code(lpdata: LPARAM) -> DWORD {
    let kb = *(lpdata.0 as *const KBDLLHOOKSTRUCT);
    kb.scanCode
}

pub unsafe fn get_point(lpdata: LPARAM) -> (LONG, LONG) {
    let mouse = *(lpdata.0 as *const MSLLHOOKSTRUCT);
    (mouse.pt.x, mouse.pt.y)
}

pub unsafe fn get_delta(lpdata: LPARAM) -> WORD {
    let mouse = *(lpdata.0 as *const MSLLHOOKSTRUCT);
    HIWORD(mouse.mouseData)
}

pub unsafe fn get_button_code(lpdata: LPARAM) -> WORD {
    let mouse = *(lpdata.0 as *const MSLLHOOKSTRUCT);
    HIWORD(mouse.mouseData)
}

pub unsafe fn convert(param: WPARAM, lpdata: LPARAM) -> Option<EventType> {
    match param.0.try_into() {
        Ok(WM_KEYDOWN) | Ok(WM_SYSKEYDOWN) => {
            let code = get_code(lpdata);
            let key = key_from_code(code as u16);
            Some(EventType::KeyPress(key))
        }
        Ok(WM_KEYUP) | Ok(WM_SYSKEYUP) => {
            let code = get_code(lpdata);
            let key = key_from_code(code as u16);
            Some(EventType::KeyRelease(key))
        }
        Ok(WM_LBUTTONDOWN) => Some(EventType::ButtonPress(Button::Left)),
        Ok(WM_LBUTTONUP) => Some(EventType::ButtonRelease(Button::Left)),
        Ok(WM_MBUTTONDOWN) => Some(EventType::ButtonPress(Button::Middle)),
        Ok(WM_MBUTTONUP) => Some(EventType::ButtonRelease(Button::Middle)),
        Ok(WM_RBUTTONDOWN) => Some(EventType::ButtonPress(Button::Right)),
        Ok(WM_RBUTTONUP) => Some(EventType::ButtonRelease(Button::Right)),
        Ok(WM_XBUTTONDOWN) => {
            let code = get_button_code(lpdata) as u8;
            Some(EventType::ButtonPress(Button::Unknown(code)))
        }
        Ok(WM_XBUTTONUP) => {
            let code = get_button_code(lpdata) as u8;
            Some(EventType::ButtonRelease(Button::Unknown(code)))
        }
        Ok(WM_MOUSEMOVE) => {
            let (x, y) = get_point(lpdata);
            Some(EventType::MouseMove {
                x: x as f64,
                y: y as f64,
            })
        }
        Ok(WM_MOUSEWHEEL) => {
            let delta = get_delta(lpdata) as c_short;
            Some(EventType::Wheel {
                delta_x: 0,
                delta_y: (delta / WHEEL_DELTA as i16) as i64,
            })
        }
        Ok(WM_MOUSEHWHEEL) => {
            let delta = get_delta(lpdata) as c_short;
            Some(EventType::Wheel {
                delta_x: (delta / WHEEL_DELTA as i16) as i64,
                delta_y: 0,
            })
        }
        _ => None,
    }
}

type RawCallback = unsafe extern "system" fn(code: c_int, param: WPARAM, lpdata: LPARAM) -> LRESULT;

pub enum HookError {
    Mouse(DWORD),
    Key(DWORD),
}

pub unsafe fn set_key_hook(callback: RawCallback) -> Result<(), HookError> {
    let hook = SetWindowsHookExA(WH_KEYBOARD_LL, Some(callback), HINSTANCE(null_mut()), 0);

    if hook.is_err() {
        let error = GetLastError();
        return Err(HookError::Key(error.0));
    }
    HOOK = hook.unwrap();
    Ok(())
}

pub unsafe fn set_mouse_hook(callback: RawCallback) -> Result<(), HookError> {
    let hook = SetWindowsHookExA(WH_MOUSE_LL, Some(callback), HINSTANCE(null_mut()), 0);
    if hook.is_err() {
        let error = GetLastError();
        return Err(HookError::Mouse(error.0));
    }
    HOOK = hook.unwrap();
    Ok(())
}
