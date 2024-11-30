use windows::Win32::UI::Input::KeyboardAndMouse::SendInput;
use windows::Win32::UI::Input::KeyboardAndMouse::INPUT;
use windows::Win32::UI::Input::KeyboardAndMouse::INPUT_0;
use windows::Win32::UI::Input::KeyboardAndMouse::INPUT_KEYBOARD;
use windows::Win32::UI::Input::KeyboardAndMouse::INPUT_MOUSE;
use windows::Win32::UI::Input::KeyboardAndMouse::KEYBDINPUT;
use windows::Win32::UI::Input::KeyboardAndMouse::KEYBD_EVENT_FLAGS;
use windows::Win32::UI::Input::KeyboardAndMouse::KEYEVENTF_KEYUP;
use windows::Win32::UI::Input::KeyboardAndMouse::MOUSEEVENTF_ABSOLUTE;
use windows::Win32::UI::Input::KeyboardAndMouse::MOUSEEVENTF_HWHEEL;
use windows::Win32::UI::Input::KeyboardAndMouse::MOUSEEVENTF_LEFTDOWN;
use windows::Win32::UI::Input::KeyboardAndMouse::MOUSEEVENTF_LEFTUP;
use windows::Win32::UI::Input::KeyboardAndMouse::MOUSEEVENTF_MIDDLEDOWN;
use windows::Win32::UI::Input::KeyboardAndMouse::MOUSEEVENTF_MIDDLEUP;
use windows::Win32::UI::Input::KeyboardAndMouse::MOUSEEVENTF_MOVE;
use windows::Win32::UI::Input::KeyboardAndMouse::MOUSEEVENTF_RIGHTDOWN;
use windows::Win32::UI::Input::KeyboardAndMouse::MOUSEEVENTF_RIGHTUP;
use windows::Win32::UI::Input::KeyboardAndMouse::MOUSEEVENTF_VIRTUALDESK;
use windows::Win32::UI::Input::KeyboardAndMouse::MOUSEEVENTF_WHEEL;
use windows::Win32::UI::Input::KeyboardAndMouse::MOUSEEVENTF_XDOWN;
use windows::Win32::UI::Input::KeyboardAndMouse::MOUSEEVENTF_XUP;
use windows::Win32::UI::Input::KeyboardAndMouse::MOUSEINPUT;
use windows::Win32::UI::Input::KeyboardAndMouse::MOUSE_EVENT_FLAGS;
use windows::Win32::UI::Input::KeyboardAndMouse::VIRTUAL_KEY;
use windows::Win32::UI::WindowsAndMessaging::GetSystemMetrics;
use windows::Win32::UI::WindowsAndMessaging::SM_CXVIRTUALSCREEN;
use windows::Win32::UI::WindowsAndMessaging::SM_CYVIRTUALSCREEN;
use windows::Win32::UI::WindowsAndMessaging::WHEEL_DELTA;

use crate::common::DWORD;
use crate::common::LONG;
use crate::common::WORD;
use crate::keycodes::code_from_key;
use crate::win_binder::Button;
use crate::win_binder::EventType;
use crate::win_binder::SimulateError;
use std::convert::TryFrom;
use std::mem::size_of;
use std::os::raw::c_int;
use std::os::raw::c_short;

static KEYEVENTF_KEYDOWN: KEYBD_EVENT_FLAGS = KEYBD_EVENT_FLAGS(0);

fn sim_mouse_event(
    flags: MOUSE_EVENT_FLAGS,
    data: DWORD,
    dx: LONG,
    dy: LONG,
) -> Result<(), SimulateError> {
    let mut union: INPUT_0 = unsafe { std::mem::zeroed() };
    union.mi = MOUSEINPUT {
        dx,
        dy,
        mouseData: data,
        dwFlags: flags,
        time: 0,
        dwExtraInfo: 0,
    };
    let input = [INPUT {
        r#type: INPUT_MOUSE,
        Anonymous: union,
    }; 1];
    let value = unsafe { SendInput(&input, size_of::<INPUT>() as c_int) };
    if value != 1 {
        Err(SimulateError)
    } else {
        Ok(())
    }
}

fn sim_keyboard_event(
    flags: KEYBD_EVENT_FLAGS,
    vk: VIRTUAL_KEY,
    scan: WORD,
) -> Result<(), SimulateError> {
    let mut union: INPUT_0 = unsafe { std::mem::zeroed() };
    union.ki = KEYBDINPUT {
        wVk: vk,
        wScan: scan,
        dwFlags: flags,
        time: 0,
        dwExtraInfo: 0,
    };
    let input = [INPUT {
        r#type: INPUT_KEYBOARD,
        Anonymous: union,
    }; 1];
    let value = unsafe { SendInput(&input, size_of::<INPUT>() as c_int) };
    if value != 1 {
        Err(SimulateError)
    } else {
        Ok(())
    }
}

pub fn simulate(event_type: &EventType) -> Result<(), SimulateError> {
    match event_type {
        EventType::KeyPress(key) => {
            let code = code_from_key(*key).ok_or(SimulateError)?;
            sim_keyboard_event(KEYEVENTF_KEYDOWN, VIRTUAL_KEY(code), 0)
        }
        EventType::KeyRelease(key) => {
            let code = code_from_key(*key).ok_or(SimulateError)?;
            sim_keyboard_event(KEYEVENTF_KEYUP, VIRTUAL_KEY(code), 0)
        }
        EventType::ButtonPress(button) => match button {
            Button::Left => sim_mouse_event(MOUSEEVENTF_LEFTDOWN, 0, 0, 0),
            Button::Middle => sim_mouse_event(MOUSEEVENTF_MIDDLEDOWN, 0, 0, 0),
            Button::Right => sim_mouse_event(MOUSEEVENTF_RIGHTDOWN, 0, 0, 0),
            Button::Unknown(code) => sim_mouse_event(MOUSEEVENTF_XDOWN, (*code).into(), 0, 0),
        },
        EventType::ButtonRelease(button) => match button {
            Button::Left => sim_mouse_event(MOUSEEVENTF_LEFTUP, 0, 0, 0),
            Button::Middle => sim_mouse_event(MOUSEEVENTF_MIDDLEUP, 0, 0, 0),
            Button::Right => sim_mouse_event(MOUSEEVENTF_RIGHTUP, 0, 0, 0),
            Button::Unknown(code) => sim_mouse_event(MOUSEEVENTF_XUP, (*code).into(), 0, 0),
        },
        EventType::Wheel { delta_x, delta_y } => {
            if *delta_x != 0 {
                sim_mouse_event(
                    MOUSEEVENTF_HWHEEL,
                    (c_short::try_from(*delta_x).map_err(|_| SimulateError)? * WHEEL_DELTA as i16)
                        as u32,
                    0,
                    0,
                )?;
            }

            if *delta_y != 0 {
                sim_mouse_event(
                    MOUSEEVENTF_WHEEL,
                    (c_short::try_from(*delta_y).map_err(|_| SimulateError)? * WHEEL_DELTA as i16)
                        as u32,
                    0,
                    0,
                )?;
            }
            Ok(())
        }
        EventType::MouseMove { x, y } => {
            let width = unsafe { GetSystemMetrics(SM_CXVIRTUALSCREEN) };
            let height = unsafe { GetSystemMetrics(SM_CYVIRTUALSCREEN) };
            if width == 0 || height == 0 {
                return Err(SimulateError);
            }

            sim_mouse_event(
                MOUSEEVENTF_MOVE | MOUSEEVENTF_ABSOLUTE | MOUSEEVENTF_VIRTUALDESK,
                0,
                (*x as i32 + 1) * 65535 / width,
                (*y as i32 + 1) * 65535 / height,
            )
        }
    }
}
