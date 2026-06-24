#![windows_subsystem = "windows"]

mod headset;
mod icon;

use std::{
    sync::{Arc, Mutex},
    thread,
};
use tray_icon::{
    menu::{CheckMenuItem, Menu, MenuEvent, MenuItem},
    Icon, TrayIconBuilder,
};
use windows_sys::Win32::System::Threading::GetCurrentThreadId;
use windows_sys::Win32::UI::WindowsAndMessaging::{
    DispatchMessageW, GetMessageW, PostQuitMessage, TranslateMessage, MSG, WM_APP,
};
use winreg::{enums::*, RegKey};

pub struct State {
    pub headset: Option<u8>,
}

const RUN_KEY: &str = "Software\\Microsoft\\Windows\\CurrentVersion\\Run";
const APP_NAME: &str = "buhttery";

fn autostart_enabled() -> bool {
    RegKey::predef(HKEY_CURRENT_USER)
        .open_subkey(RUN_KEY)
        .and_then(|k| k.get_value::<String, _>(APP_NAME))
        .is_ok()
}

fn autostart_set(enable: bool) {
    let hkcu = RegKey::predef(HKEY_CURRENT_USER);
    if enable {
        if let Ok(exe) = std::env::current_exe() {
            if let Ok((key, _)) = hkcu.create_subkey(RUN_KEY) {
                let path = exe.to_string_lossy().into_owned();
                let _ = key.set_value(APP_NAME, &path);
            }
        }
    } else if let Ok(key) = hkcu.open_subkey_with_flags(RUN_KEY, KEY_WRITE) {
        let _ = key.delete_value(APP_NAME);
    }
}

fn make_icon(headset: Option<u8>) -> Icon {
    Icon::from_rgba(icon::render(headset), 32, 32).expect("icon")
}

fn tooltip(headset: Option<u8>) -> String {
    match headset {
        Some(p) => format!("headset: {}%", p),
        None => "headset: --".into(),
    }
}

fn main() {
    let autostart_item = CheckMenuItem::new("Launch at startup", true, autostart_enabled(), None);
    let quit_item = MenuItem::new("Quit", true, None);
    let autostart_id = autostart_item.id().clone();
    let quit_id = quit_item.id().clone();

    let menu = Menu::new();
    let _ = menu.append(&autostart_item);
    let _ = menu.append(&quit_item);

    let tray = TrayIconBuilder::new()
        .with_icon(make_icon(None))
        .with_tooltip("headset: --")
        .with_menu(Box::new(menu))
        .build()
        .expect("tray icon");

    let state = Arc::new(Mutex::new(State { headset: None }));
    let tid = unsafe { GetCurrentThreadId() };

    let s1 = state.clone();
    thread::spawn(move || headset::run(s1, tid));

    let menu_rx = MenuEvent::receiver();

    unsafe {
        let mut msg = std::mem::zeroed::<MSG>();
        loop {
            let ret = GetMessageW(&mut msg, std::ptr::null_mut(), 0, 0);
            if ret == 0 || ret == -1 {
                break;
            }

            if msg.message == WM_APP {
                let headset = state.lock().unwrap().headset;
                let _ = tray.set_icon(Some(make_icon(headset)));
                let _ = tray.set_tooltip(Some(&tooltip(headset)));
                continue;
            }

            TranslateMessage(&msg);
            DispatchMessageW(&msg);

            while let Ok(ev) = menu_rx.try_recv() {
                if ev.id == autostart_id {
                    let enabled = autostart_enabled();
                    autostart_set(!enabled);
                    let _ = autostart_item.set_checked(!enabled);
                } else if ev.id == quit_id {
                    PostQuitMessage(0);
                }
            }
        }
    }
}
