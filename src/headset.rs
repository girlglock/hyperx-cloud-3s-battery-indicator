use crate::State;
use hidapi::{HidApi, HidDevice};
use std::{
    sync::{Arc, Mutex},
    thread,
    time::Duration,
};
use windows_sys::Win32::UI::WindowsAndMessaging::{PostThreadMessageW, WM_APP};

const VID: u16 = 0x03F0;
const PID: u16 = 0x06BE;
const REPORT_ID: u8 = 0x0C;
const CMD_BATTERY: u8 = 0x06;

fn request() -> [u8; 64] {
    let mut r = [0u8; 64];
    r[0] = REPORT_ID;
    r[1] = 0x02;
    r[2] = 0x03;
    r[3] = 0x01;
    r[5] = CMD_BATTERY;
    r
}

fn parse(buf: &[u8], n: usize) -> Option<u8> {
    if n >= 7 && buf[0] == REPORT_ID && buf[5] == CMD_BATTERY {
        return Some(buf[6]);
    }
    if n >= 6 && buf[0] == 0x02 && buf[4] == CMD_BATTERY {
        return Some(buf[5]);
    }
    None
}

fn probe(api: &HidApi) -> Option<HidDevice> {
    let req = request();
    for info in api.device_list() {
        if info.vendor_id() != VID || info.product_id() != PID {
            continue;
        }
        let Ok(dev) = info.open_device(api) else {
            continue;
        };
        if dev.write(&req).is_err() {
            continue;
        }
        thread::sleep(Duration::from_millis(200));
        let mut buf = [0u8; 64];
        if let Ok(n) = dev.read_timeout(&mut buf, 1000) {
            if parse(&buf, n).is_some() {
                return Some(dev);
            }
        }
    }
    None
}

fn query(dev: &HidDevice) -> Option<u8> {
    let req = request();
    dev.write(&req).ok()?;
    thread::sleep(Duration::from_millis(100));
    let mut buf = [0u8; 64];
    let n = dev.read_timeout(&mut buf, 1000).ok()?;
    parse(&buf, n)
}

fn notify(tid: u32) {
    unsafe { PostThreadMessageW(tid, WM_APP, 0, 0) };
}

pub fn run(state: Arc<Mutex<State>>, tid: u32) {
    loop {
        let api = match HidApi::new() {
            Ok(a) => a,
            Err(_) => {
                thread::sleep(Duration::from_secs(10));
                continue;
            }
        };

        match probe(&api) {
            None => {
                state.lock().unwrap().headset = None;
                notify(tid);
                thread::sleep(Duration::from_secs(30));
            }
            Some(dev) => loop {
                match query(&dev) {
                    Some(pct) => {
                        state.lock().unwrap().headset = Some(pct);
                        notify(tid);
                        thread::sleep(Duration::from_secs(300));
                    }
                    None => {
                        state.lock().unwrap().headset = None;
                        notify(tid);
                        thread::sleep(Duration::from_secs(30));
                        break;
                    }
                }
            },
        }
    }
}
