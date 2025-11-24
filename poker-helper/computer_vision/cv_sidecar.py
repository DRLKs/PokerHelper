import sys
import json
import time
from PIL import ImageGrab
import numpy as np
import base64
from io import BytesIO
import platform

# Linux specific imports
if platform.system() == "Linux":
    from ewmh import EWMH
    ewmh = EWMH()

# This script will be called by the Tauri app as a sidecar
# It reads commands from stdin and writes JSON to stdout

def get_poker_windows():
    try:
        if platform.system() == "Linux":
            windows = ewmh.getClientList()
            poker_windows = []
            for win in windows:
                name = ewmh.getWmName(win)
                if name:
                    # Decode bytes if necessary
                    if isinstance(name, bytes):
                        name = name.decode('utf-8', errors='ignore')
                    poker_windows.append(name)
            return poker_windows
        else:
            # Fallback for Windows/Mac (if pygetwindow was installed)
            import pygetwindow as gw
            windows = gw.getAllTitles()
            poker_windows = [w for w in windows if w.strip()]
            return poker_windows
    except Exception as e:
        return []

def capture_window(window_title):
    try:
        if platform.system() == "Linux":
            windows = ewmh.getClientList()
            target_win = None
            for win in windows:
                name = ewmh.getWmName(win)
                if isinstance(name, bytes):
                    name = name.decode('utf-8', errors='ignore')
                if name == window_title:
                    target_win = win
                    break
            
            if not target_win:
                return {"error": "Window not found"}
            
            # Get geometry
            geo = ewmh.getWmGeometry(target_win)
            # x, y, width, height
            # Note: ImageGrab on Linux usually grabs the whole screen, so we crop
            # Or we can use xlib to grab specific window, but ImageGrab with bbox is easier if coordinates are correct relative to root
            # EWMH geometry is usually relative to root
            
            # However, ImageGrab.grab(bbox=...) on Linux with X11 backend works
            # geo returns (x, y, width, height)
            # bbox is (left, top, right, bottom)
            bbox = (geo[0], geo[1], geo[0] + geo[2], geo[1] + geo[3])
            screenshot = ImageGrab.grab(bbox=bbox)
            
        else:
            import pygetwindow as gw
            windows = gw.getWindowsWithTitle(window_title)
            if not windows:
                return {"error": "Window not found"}
            window = windows[0]
            bbox = (window.left, window.top, window.right, window.bottom)
            screenshot = ImageGrab.grab(bbox=bbox)
        
        # Convert to base64 for sending back (or process directly here)
        buffered = BytesIO()
        screenshot.save(buffered, format="PNG")
        img_str = base64.b64encode(buffered.getvalue()).decode()
        
        return {"image": img_str, "width": screenshot.width, "height": screenshot.height}
    except Exception as e:
        return {"error": str(e)}

def main():
    # Simple command loop
    while True:
        try:
            line = sys.stdin.readline()
            if not line:
                break
            
            command = json.loads(line)
            response = {}
            
            if command["type"] == "list_windows":
                response = {"windows": get_poker_windows()}
            elif command["type"] == "capture":
                response = capture_window(command["window_title"])
            elif command["type"] == "ping":
                response = {"status": "pong"}
            else:
                response = {"error": "Unknown command"}
                
            print(json.dumps(response))
            sys.stdout.flush()
            
        except Exception as e:
            print(json.dumps({"error": str(e)}))
            sys.stdout.flush()

if __name__ == "__main__":
    main()
