#!/usr/bin/env python3
"""
CV Sidecar: computer vision service for PokerHelper
================================================================

Runs as a child process (sidecar) of the Rust backend.
It communicates over stdin/stdout with JSON messages.

Technologies:
- YOLO v8 (Ultralytics): card detection
- EasyOCR: number extraction (pot, bets, stacks)
- mss: cross-platform screen capture

Responsibilities:
- List the available windows
- Capture screenshots of specific windows
- Analyze images to detect the poker game state

Extracted data:
- The player's hole cards (2 cards)
- Community cards (0-5 cards)
- Number of active opponents
- Pot size
- Amount to call
"""

import sys
import os
import json
import time
from datetime import datetime
from PIL import Image
import numpy as np
import base64
from io import BytesIO
import platform
from typing import Dict, Any, List, Optional, Tuple

CURRENT_DIR = os.path.dirname(os.path.abspath(__file__))
if CURRENT_DIR not in sys.path:
    sys.path.insert(0, CURRENT_DIR)


try:
    import mss
    MSS_AVAILABLE = True
except ImportError:
    MSS_AVAILABLE = False
    from PIL import ImageGrab

if platform.system() == "Linux":
    try:
        from ewmh import EWMH
        ewmh = EWMH()
        EWMH_AVAILABLE = True
    except ImportError:
        ewmh = None
        EWMH_AVAILABLE = False
else:
    ewmh = None
    EWMH_AVAILABLE = False

try:
    from predictor.CardPredictor import CardPredictor, get_predictor, AnalysisResult
    card_predictor = get_predictor(verbose=False)
    PREDICTOR_AVAILABLE = card_predictor.is_ready()
    PREDICTOR_ERROR = None if PREDICTOR_AVAILABLE else "Predictor not ready"
except Exception as e:
    card_predictor = None
    PREDICTOR_AVAILABLE = False
    PREDICTOR_ERROR = str(e)

try:
    from predictor.core.number_extractor import get_extractor
    number_extractor = get_extractor()
    OCR_AVAILABLE = True
except Exception as e:
    number_extractor = None
    OCR_AVAILABLE = False


FractionalRegion = Tuple[float, float, float, float]

DEFAULT_REGIONS: Dict[str, FractionalRegion] = {
    "hand": (0.40, 0.70, 0.60, 0.90),
    "community": (0.30, 0.35, 0.70, 0.50),
    "pot": (0.40, 0.25, 0.60, 0.35),
    "call": (0.45, 0.85, 0.55, 0.95),
    "player_stack": (0.40, 0.92, 0.60, 0.98),
}

HAND_SIZE = 2
BOARD_SIZE = 5

RANK_VALUES = {
    'A': 14, 'K': 13, 'Q': 12, 'J': 11,
    '10': 10, 'T': 10,
    '9': 9, '8': 8, '7': 7, '6': 6,
    '5': 5, '4': 4, '3': 3, '2': 2
}

SUIT_LETTERS = {
    'hearts': 'h', 'diamonds': 'd',
    'clubs': 'c', 'spades': 's',
    'h': 'h', 'd': 'd', 'c': 'c', 's': 's'
}

OPPONENT_REGIONS = [
    {"name": "opponent_1", "position": (0.10, 0.30, 0.25, 0.50)},
    {"name": "opponent_2", "position": (0.10, 0.50, 0.25, 0.70)},
    {"name": "opponent_3", "position": (0.35, 0.05, 0.65, 0.20)},
    {"name": "opponent_4", "position": (0.75, 0.30, 0.90, 0.50)},
    {"name": "opponent_5", "position": (0.75, 0.50, 0.90, 0.70)},
]


def bgra_to_rgb(pixels: np.ndarray) -> np.ndarray:
    return pixels[:, :, :3][:, :, ::-1]


def capture_screen_region(bbox: Tuple[int, int, int, int]) -> Optional[np.ndarray]:
    """
    Captures a region of the screen.
    
    Args:
        bbox: (left, top, right, bottom) in absolute pixels
        
    Returns:
        RGB numpy array, or None on failure
    """
    try:
        if MSS_AVAILABLE:
            with mss.mss() as sct:
                monitor = {
                    "left": bbox[0],
                    "top": bbox[1],
                    "width": bbox[2] - bbox[0],
                    "height": bbox[3] - bbox[1]
                }
                screenshot = sct.grab(monitor)
                return bgra_to_rgb(np.array(screenshot))
        else:
            screenshot = ImageGrab.grab(bbox=bbox)
            return np.array(screenshot)
    except Exception as e:
        print(f"[ERROR] capture_screen_region: {e}", file=sys.stderr)
        return None


def capture_full_screen() -> Optional[np.ndarray]:
    """Captures the full screen."""
    try:
        if MSS_AVAILABLE:
            with mss.mss() as sct:
                primary_monitor = sct.monitors[1]
                screenshot = sct.grab(primary_monitor)
                return bgra_to_rgb(np.array(screenshot))
        else:
            screenshot = ImageGrab.grab()
            return np.array(screenshot)
    except Exception as e:
        print(f"[ERROR] capture_full_screen: {e}", file=sys.stderr)
        return None


def get_poker_windows() -> List[str]:
    """Returns the list of available windows."""
    try:
        if platform.system() == "Linux" and EWMH_AVAILABLE and ewmh:
            windows = ewmh.getClientList()
            poker_windows = []
            for win in windows:
                try:
                    name = ewmh.getWmName(win)
                    if name:
                        if isinstance(name, bytes):
                            name = name.decode('utf-8', errors='ignore')
                        poker_windows.append(name)
                except Exception:
                    continue
            return poker_windows
        else:
            try:
                import pygetwindow as gw
                windows = gw.getAllTitles()
                return [w for w in windows if w.strip()]
            except ImportError:
                return []
    except Exception as e:
        print(f"[ERROR] get_poker_windows: {e}", file=sys.stderr)
        return []


def get_window_bbox(window_title: str) -> Optional[Tuple[int, int, int, int]]:
    """
    Returns the bounding box of a window by title.
    
    Returns:
        (left, top, right, bottom) or None
    """
    try:
        if platform.system() == "Linux" and EWMH_AVAILABLE and ewmh:
            windows = ewmh.getClientList()
            for win in windows:
                try:
                    name = ewmh.getWmName(win)
                    if isinstance(name, bytes):
                        name = name.decode('utf-8', errors='ignore')
                    if name == window_title:
                        geo = ewmh.getWmGeometry(win)
                        return (geo[0], geo[1], geo[0] + geo[2], geo[1] + geo[3])
                except Exception:
                    continue
            return None
        else:
            try:
                import pygetwindow as gw
                windows = gw.getWindowsWithTitle(window_title)
                if windows:
                    w = windows[0]
                    return (w.left, w.top, w.right, w.bottom)
                return None
            except ImportError:
                return None
    except Exception:
        return None


def capture_window(window_title: str) -> Dict[str, Any]:
    """
    Captures a specific window and returns the image.
    
    Args:
        window_title: Title of the window to capture
        
    Returns:
        Dict with the base64 image and its dimensions, or an error
    """
    try:
        bbox = get_window_bbox(window_title)
        
        if not bbox:
            return {"error": f"Window not found: {window_title}"}
        
        img_array = capture_screen_region(bbox)
        
        if img_array is None:
            return {"error": "Failed to capture screen"}
        
        image = Image.fromarray(img_array)
        buffered = BytesIO()
        image.save(buffered, format="PNG")
        img_str = base64.b64encode(buffered.getvalue()).decode()
        
        return {
            "image": img_str,
            "width": image.width,
            "height": image.height,
            "bbox": list(bbox)
        }
    except Exception as e:
        return {"error": str(e)}


def extract_region(image: np.ndarray, region: Tuple[float, float, float, float]) -> np.ndarray:
    """
    Crops a region of the image using relative coordinates (0-1).
    
    Args:
        image: Image as a numpy array
        region: (x1, y1, x2, y2) as fractions between 0 and 1
        
    Returns:
        Cropped region as a numpy array
    """
    height, width = image.shape[:2]
    x1, y1, x2, y2 = region
    
    left = int(x1 * width)
    top = int(y1 * height)
    right = int(x2 * width)
    bottom = int(y2 * height)
    
    return image[top:bottom, left:right]


def detect_cards_in_region(
    image: np.ndarray,
    region_name: str
) -> List[Dict[str, Any]]:
    """
    Detects cards in an image using the predictor.
    
    Args:
        image: Image as a numpy array (already cropped)
        region_name: Region name used for logging
        
    Returns:
        Detected cards, in the format the Rust backend expects
    """
    if not PREDICTOR_AVAILABLE or card_predictor is None:
        return []
    
    try:
        result = card_predictor.analyze_image(
            image,
            detect_cards=True,
            extract_numbers=False
        )
        
        cards = []
        for card in result.cards:
            rank = RANK_VALUES.get(card.rank.upper(), 0)
            suit = SUIT_LETTERS.get(card.suit.lower(), 'h')
            
            if rank > 0:
                cards.append({
                    "suit": suit,
                    "rank": rank,
                    "confidence": card.confidence
                })
        
        return cards
        
    except Exception as e:
        print(f"[ERROR] detect_cards_in_region ({region_name}): {e}", file=sys.stderr)
        return []


def extract_number_from_region(
    image: np.ndarray,
    region: Tuple[float, float, float, float],
    region_name: str = "unknown"
) -> float:
    """
    Extracts a number from a region using EasyOCR.
    
    Args:
        image: Full image as a numpy array
        region: Relative coordinates (0-1)
        region_name: Region name
        
    Returns:
        Extracted value, or 0.0
    """
    if not OCR_AVAILABLE or number_extractor is None:
        return 0.0
    
    try:
        height, width = image.shape[:2]
        x1, y1, x2, y2 = region
        roi = (int(x1 * width), int(y1 * height), int(x2 * width), int(y2 * height))
        
        results = number_extractor.extract_from_region(image, roi)
        
        if results:
            amounts = [amount for amount, *_ in results if amount > 0]
            if amounts:
                return max(amounts)
        
        return 0.0
        
    except Exception as e:
        print(f"[ERROR] extract_number_from_region ({region_name}): {e}", file=sys.stderr)
        return 0.0


def count_active_opponents(image: np.ndarray) -> int:
    """
    Counts the active opponents at the table.
    
    TODO: Detect opponents with YOLO or heuristics
    """
    return 1


def analyze_game_state(image: np.ndarray) -> Dict[str, Any]:
    """
    Analyzes a full image of the poker table and extracts the game state.
    
    Args:
        image: RGB numpy array of the poker table
    
    Returns:
        Dict compatible with the Rust CvState (poker_agent)
    """
    start_time = time.time()
    
    result = {
        "status": "ok",
        "small_blind": 0.0,
        "pot": 0.0,
        "call_amount": 0.0,
        "opponents": [],
        "hand": [],
        "community_cards": [],
        "num_opponents": 0,
        "timestamp": datetime.utcnow().isoformat(),
        "processing_time_ms": 0.0,
        "confidence_avg": 0.0
    }
    
    try:
        hand_region = extract_region(image, DEFAULT_REGIONS["hand"])
        hand_cards = detect_cards_in_region(hand_region, "hand")
        result["hand"] = hand_cards[:HAND_SIZE]
        
        community_region = extract_region(image, DEFAULT_REGIONS["community"])
        community_cards = detect_cards_in_region(community_region, "community")
        result["community_cards"] = community_cards[:BOARD_SIZE]
        
        result["pot"] = extract_number_from_region(
            image, DEFAULT_REGIONS["pot"], "pot"
        )
        result["call_amount"] = extract_number_from_region(
            image, DEFAULT_REGIONS["call"], "call"
        )
        
        num_opponents = count_active_opponents(image)
        result["num_opponents"] = num_opponents
        result["opponents"] = [{"opponent_bet": 0.0} for _ in range(num_opponents)]
        
        all_cards = result["hand"] + result["community_cards"]
        if all_cards:
            confidences = [c.get("confidence", 0) for c in all_cards]
            result["confidence_avg"] = sum(confidences) / len(confidences)
        
        if not result["hand"]:
            result["status"] = "no_hand_detected"
        
    except Exception as e:
        result["status"] = "error"
        result["error"] = str(e)
        print(f"[ERROR] analyze_game_state: {e}", file=sys.stderr)
    
    result["processing_time_ms"] = (time.time() - start_time) * 1000
    return result


def analyze_from_capture(window_title: str) -> Dict[str, Any]:
    """
    Captures a window and analyzes its content.
    
    Args:
        window_title: Title of the window to capture and analyze
        
    Returns:
        Game state, or an error
    """
    bbox = get_window_bbox(window_title)
    
    if not bbox:
        return {
            "status": "error",
            "error": f"Window not found: {window_title}",
            "hand": [],
            "community_cards": [],
            "opponents": [],
            "pot": 0.0,
            "call_amount": 0.0,
            "num_opponents": 0,
            "small_blind": 0.0
        }
    
    img_array = capture_screen_region(bbox)
    
    if img_array is None:
        return {
            "status": "error",
            "error": "Failed to capture screen",
            "hand": [],
            "community_cards": [],
            "opponents": [],
            "pot": 0.0,
            "call_amount": 0.0,
            "num_opponents": 0,
            "small_blind": 0.0
        }
    
    return analyze_game_state(img_array)


def main():
    """
    Main loop that processes commands from stdin.
    
    Supported commands:
    - ping: Check connectivity
    - status: Sidecar status
    - list_windows: List available windows
    - capture: Capture a window as an image
    - analyze: Analyze a window and extract the game state
    - analyze_image: Analyze a provided base64 image
    - set_regions: Update the detection regions
    """
    print(f"[CV Sidecar] Started on {platform.system()}", file=sys.stderr)
    print(f"[CV Sidecar] YOLO: {PREDICTOR_AVAILABLE}, OCR: {OCR_AVAILABLE}", file=sys.stderr)
    
    while True:
        try:
            line = sys.stdin.readline()
            if not line:
                break
            
            command = json.loads(line.strip())
            response = {}
            
            cmd_type = command.get("type", "")
            
            if cmd_type == "ping":
                response = {
                    "status": "pong",
                    "predictor_available": PREDICTOR_AVAILABLE,
                    "ocr_available": OCR_AVAILABLE,
                    "platform": platform.system(),
                    "mss_available": MSS_AVAILABLE
                }
            
            elif cmd_type == "status":
                predictor_status = {}
                if card_predictor:
                    predictor_status = card_predictor.get_status()
                
                response = {
                    "status": "ok",
                    "predictor_available": PREDICTOR_AVAILABLE,
                    "predictor_error": PREDICTOR_ERROR if not PREDICTOR_AVAILABLE else None,
                    "predictor_details": predictor_status,
                    "ocr_available": OCR_AVAILABLE,
                    "platform": platform.system(),
                    "mss_available": MSS_AVAILABLE,
                    "ewmh_available": EWMH_AVAILABLE,
                    "regions": DEFAULT_REGIONS
                }
            
            elif cmd_type == "list_windows":
                response = {"windows": get_poker_windows()}
            
            elif cmd_type == "capture":
                window_title = command.get("window_title", "")
                response = capture_window(window_title)
            
            elif cmd_type == "analyze":
                window_title = command.get("window_title", "")
                response = analyze_from_capture(window_title)
            
            elif cmd_type == "analyze_image":
                img_b64 = command.get("image", "")
                if img_b64:
                    try:
                        img_data = base64.b64decode(img_b64)
                        image = Image.open(BytesIO(img_data))
                        img_array = np.array(image)
                        response = analyze_game_state(img_array)
                    except Exception as e:
                        response = {"status": "error", "error": str(e)}
                else:
                    response = {"status": "error", "error": "No image provided"}
            
            elif cmd_type == "set_regions":
                new_regions = command.get("regions", {})
                for key in new_regions:
                    if key in DEFAULT_REGIONS:
                        DEFAULT_REGIONS[key] = tuple(new_regions[key])
                response = {"status": "ok", "regions": DEFAULT_REGIONS}
            
            elif cmd_type == "get_regions":
                response = {"status": "ok", "regions": DEFAULT_REGIONS}
            
            else:
                response = {"error": f"Unknown command: {cmd_type}"}
            
            print(json.dumps(response))
            sys.stdout.flush()
            
        except json.JSONDecodeError as e:
            print(json.dumps({"error": f"Invalid JSON: {str(e)}"}))
            sys.stdout.flush()
        except Exception as e:
            print(json.dumps({"error": str(e)}))
            sys.stdout.flush()


if __name__ == "__main__":
    main()
