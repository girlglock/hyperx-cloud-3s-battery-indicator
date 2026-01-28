import pystray
from PIL import Image
import hid
import time
import threading
import sys
import os

if getattr(sys, 'frozen', False):
    base_path = sys._MEIPASS
else:
    base_path = os.path.dirname(os.path.abspath(__file__))

ASSETS_PATH = os.path.join(base_path, 'assets')

VID = 0x03F0
PID = 0x06BE
REPORT_ID = 0x0C
CMD_BATTERY = 0x06
QUERY_INTERVAL = 300 # 300s is 5m

battery_percent = 0
icon = None

def get_battery_icon(percent):
    match percent:
        case p if p >= 90:
            return Image.open(os.path.join(ASSETS_PATH, "100.png"))
        case p if p >= 80:
            return Image.open(os.path.join(ASSETS_PATH, "90.png"))
        case p if p >= 70:
            return Image.open(os.path.join(ASSETS_PATH, "80.png"))
        case p if p >= 60:
            return Image.open(os.path.join(ASSETS_PATH, "70.png"))
        case p if p >= 50:
            return Image.open(os.path.join(ASSETS_PATH, "60.png"))
        case p if p >= 40:
            return Image.open(os.path.join(ASSETS_PATH, "50.png"))
        case p if p >= 30:
            return Image.open(os.path.join(ASSETS_PATH, "40.png"))
        case p if p >= 20:
            return Image.open(os.path.join(ASSETS_PATH, "30.png"))
        case p if p >= 10:
            return Image.open(os.path.join(ASSETS_PATH, "20.png"))
        case _:
            return Image.open(os.path.join(ASSETS_PATH, "10.png"))

def query_battery(device):
    req = [REPORT_ID, 0x02, 0x03, 0x01, 0x00, CMD_BATTERY, 0x00] + [0] * 57
    device.write(req)
    time.sleep(0.1)
    data = device.read(64, timeout_ms=1000)
    if data and len(data) >= 7:
        if data[0] == REPORT_ID and data[5] == CMD_BATTERY:
            return data[6]
        if data[0] == 0x02 and data[4] == CMD_BATTERY:
            return data[5]
    return None

def find_working_device():
    devices = hid.enumerate(VID, PID)
    for d in devices:
        try:
            device = hid.device()
            device.open_path(d['path'])
            req = [REPORT_ID, 0x02, 0x03, 0x01, 0x00, CMD_BATTERY, 0x00] + [0] * 57
            device.write(req)
            time.sleep(0.2)
            data = device.read(64, timeout_ms=1000)
            if data and len(data) >= 6:
                if (data[0] == REPORT_ID and data[5] == CMD_BATTERY) or (data[0] == 0x02 and data[4] == CMD_BATTERY):
                    device.close()
                    return d['path']
            device.close()
        except:
            pass
    return None

def battery_loop():
    global battery_percent, icon
    try:
        working_path = find_working_device()
        if not working_path:
            if icon:
                icon.title = "headset not found :c"
                icon.update_menu()
            return
        
        device = hid.device()
        device.open_path(working_path)
        
        while True:
            pct = query_battery(device)
            if pct is not None:
                battery_percent = pct
                if icon:
                    icon.icon = get_battery_icon(battery_percent)
                    icon.title = f"hyperx cloud III s\nbattery: {battery_percent}%"
                    icon.update_menu()
            time.sleep(QUERY_INTERVAL)
    except Exception as e:
        if icon:
            icon.title = f"error: {e}"
            icon.update_menu()

def on_quit(icon, item):
    icon.stop()

def main():
    global icon
    
    menu = pystray.Menu(pystray.MenuItem("quit", on_quit))
    icon = pystray.Icon("hyperx", get_battery_icon(0), "hyperx cloud III s\nloading...", menu)
    
    thread = threading.Thread(target=battery_loop, daemon=True)
    thread.start()
    
    icon.run()

if __name__ == "__main__":
    main()