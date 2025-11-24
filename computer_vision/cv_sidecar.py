import sys
import json
import time
import pygetwindow as gw
from PIL import ImageGrab
import numpy as np
import base64
from io import BytesIO

# This script will be called by the Tauri app as a sidecar
# It reads commands from stdin and writes JSON to stdout

def get_poker_windows():
    try:
        # Get all windows
        windows = gw.getAllTitles()
        # Filter for potential poker windows (you might want to adjust this filter)
        # For now, let's return all non-empty titles to let the user choose
        poker_windows = [w for w in windows if w.strip()]
        return poker_windows
    except Exception as e:
        return []

def capture_window(window_title):
    try:
        windows = gw.getWindowsWithTitle(window_title)
        if not windows:
            return {"error": "Window not found"}
        
        window = windows[0]
        
        # Activate window (optional, might be intrusive)
        # window.activate() 
        
        # Capture
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
