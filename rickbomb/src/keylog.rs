use std::{
    fs::File,
    io::Write,
    path::{Path, PathBuf},
    sync::{atomic::AtomicBool, Arc, Mutex},
};

use bus::Bus;
use lazy_static::lazy_static;
use log::*;
use winapi::{
    shared::{
        minwindef::{DWORD, LPARAM, UINT, WPARAM},
        windef::{HWND, POINT},
    },
    um::{
        libloaderapi::GetModuleHandleW,
        winuser::{
            CallNextHookEx, DispatchMessageW, PeekMessageW, SetWindowsHookExW, TranslateMessage,
            UnhookWindowsHookEx, KBDLLHOOKSTRUCT, MSG, PM_REMOVE, WH_KEYBOARD_LL, WM_KEYDOWN,
            WM_KEYUP, WM_SYSKEYDOWN, WM_SYSKEYUP,
        },
    },
};

#[derive(Clone, Debug)]
pub struct KeystrokeInfo {
    pub kind: KeystrokeKind,
    pub vk_code: u32,
    pub scan_code: u32,
    pub flags: u32,
    pub time: u32,
}

#[derive(Clone, Copy, Debug)]
pub enum KeystrokeKind {
    Down,
    Up,
    SysDown,
    SysUp,
}

lazy_static! {
    static ref KEYSTROKE_BUS: Arc<Mutex<Bus<KeystrokeInfo>>> = Arc::new(Mutex::new(Bus::new(64)));
}

/// Initializes the keylogger. Send a unit on the oneshot channel in order to
/// stop the keylogger.
pub fn init_keylogger() -> oneshot::Sender<()> {
    let (terminate_tx, terminate_rx) = oneshot::channel();

    let message_loop = std::thread::spawn({
        move || unsafe {
            let mod_id = GetModuleHandleW(std::ptr::null());

            debug!("mod_id: {:?}", mod_id);

            let hook_id = SetWindowsHookExW(WH_KEYBOARD_LL, Some(hook_callback), mod_id, 0);

            debug!("hook_id: {:?}", hook_id);

            if hook_id.is_null() {
                println!("hook failed! exiting");
                return;
            }

            debug!("hooked!");

            let mut msg: MSG = MSG {
                hwnd: 0 as HWND,
                message: 0 as UINT,
                wParam: 0 as WPARAM,
                lParam: 0 as LPARAM,
                time: 0 as DWORD,
                pt: POINT { x: 0, y: 0 },
            };

            // call GetMessage to create a message loop, b/c hooks will not work
            // unless there is a message loop on the thread that created the
            // hook
            // https://docs.microsoft.com/en-us/previous-versions/windows/desktop/legacy/ms644985(v=vs.85)#remarks
            // https://docs.microsoft.com/en-us/windows/win32/winmsg/using-messages-and-message-queues#creating-a-message-loop
            // but since there are probably no messages to get, and we don't
            // want to block forever we will use PeekMessage instead and since
            // the message loop

            loop {
                let pm = PeekMessageW(&mut msg, 0 as HWND, 0, 0, PM_REMOVE);

                if let Ok(_) = terminate_rx.try_recv() {
                    break;
                }

                if pm == 0 {
                    continue;
                }

                TranslateMessage(&msg);
                DispatchMessageW(&msg);
            }

            UnhookWindowsHookEx(hook_id);

            debug!("unhooked!");
        }
    });

    terminate_tx
}

/// Returns a receiver that will receive keystroke events from the keylogger.
pub fn listen_keylogger() -> Option<bus::BusReader<KeystrokeInfo>> {
    if let Ok(mut keystroke_bus) = KEYSTROKE_BUS.lock() {
        Some(keystroke_bus.add_rx())
    } else {
        None
    }
}

extern "system" fn hook_callback(code: i32, w_param: usize, l_param: isize) -> isize {
    let kbd = unsafe { *(l_param as *const KBDLLHOOKSTRUCT) };

    let info = KeystrokeInfo {
        kind: match w_param as u32 {
            WM_KEYDOWN => KeystrokeKind::Down,
            WM_KEYUP => KeystrokeKind::Up,
            WM_SYSKEYDOWN => KeystrokeKind::SysDown,
            WM_SYSKEYUP => KeystrokeKind::SysUp,
            _ => panic!("unknown keystroke type"),
        },
        flags: kbd.flags,
        scan_code: kbd.scanCode,
        vk_code: kbd.vkCode,
        time: kbd.time,
    };

    if let Ok(mut keystroke_bus) = KEYSTROKE_BUS.lock() {
        keystroke_bus.broadcast(info);
    }

    unsafe { CallNextHookEx(std::ptr::null_mut(), code, w_param, l_param) }
}
