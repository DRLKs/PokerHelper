#!/usr/bin/env python3
"""
CV Sidecar - Servicio de Visión por Computador para PokerHelper
================================================================

Este script se ejecuta como un proceso secundario (sidecar) del backend de Rust.
Se comunica mediante stdin/stdout usando mensajes JSON.

Tecnologías utilizadas:
- YOLO v8 (Ultralytics): Detección de cartas
- EasyOCR: Extracción de números (pot, bets, stacks)
- mss: Captura de pantalla cross-platform

Responsabilidades:
- Listar ventanas disponibles
- Capturar pantallas de ventanas específicas
- Analizar imágenes para detectar el estado del juego de poker

Datos que extrae:
- Cartas de la mano del jugador (2 cartas)
- Cartas comunitarias (0-5 cartas)
- Número de oponentes activos
- Tamaño del bote
- Cantidad a igualar (call amount)
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

# Añadir directorio actual al path
CURRENT_DIR = os.path.dirname(os.path.abspath(__file__))
if CURRENT_DIR not in sys.path:
    sys.path.insert(0, CURRENT_DIR)

# =============================================================================
# Importaciones condicionales
# =============================================================================

# Screen capture - mss es preferido, fallback a PIL
try:
    import mss
    MSS_AVAILABLE = True
except ImportError:
    MSS_AVAILABLE = False
    from PIL import ImageGrab

# Linux window management
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

# Intentar importar el predictor actualizado (YOLO + EasyOCR)
try:
    from predictor.CardPredictor import CardPredictor, get_predictor, AnalysisResult
    card_predictor = get_predictor(verbose=False)
    PREDICTOR_AVAILABLE = card_predictor.is_ready()
    PREDICTOR_ERROR = None if PREDICTOR_AVAILABLE else "Predictor not ready"
except Exception as e:
    card_predictor = None
    PREDICTOR_AVAILABLE = False
    PREDICTOR_ERROR = str(e)

# Intentar importar el extractor de números directamente
try:
    from predictor.core.number_extractor import get_extractor
    number_extractor = get_extractor()
    OCR_AVAILABLE = True
except Exception as e:
    number_extractor = None
    OCR_AVAILABLE = False


# =============================================================================
# CONFIGURACIÓN DE REGIONES (valores por defecto, calibrar según la plataforma)
# =============================================================================

# Regiones definidas como porcentajes de la imagen (x1, y1, x2, y2)
DEFAULT_REGIONS = {
    # Región de las cartas de mano (coordenadas relativas 0-1)
    "hand": (0.40, 0.70, 0.60, 0.90),
    # Región de cartas comunitarias
    "community": (0.30, 0.35, 0.70, 0.50),
    # Región del bote (para OCR)
    "pot": (0.40, 0.25, 0.60, 0.35),
    # Región de cantidad a call
    "call": (0.45, 0.85, 0.55, 0.95),
    # Región del stack del jugador
    "player_stack": (0.40, 0.92, 0.60, 0.98),
}

# Regiones para detección de oponentes (posiciones en la mesa)
OPPONENT_REGIONS = [
    {"name": "opponent_1", "position": (0.10, 0.30, 0.25, 0.50)},
    {"name": "opponent_2", "position": (0.10, 0.50, 0.25, 0.70)},
    {"name": "opponent_3", "position": (0.35, 0.05, 0.65, 0.20)},
    {"name": "opponent_4", "position": (0.75, 0.30, 0.90, 0.50)},
    {"name": "opponent_5", "position": (0.75, 0.50, 0.90, 0.70)},
]


# =============================================================================
# FUNCIONES DE CAPTURA DE PANTALLA
# =============================================================================

def capture_screen_region(bbox: Tuple[int, int, int, int]) -> Optional[np.ndarray]:
    """
    Captura una región de la pantalla.
    
    Args:
        bbox: (left, top, right, bottom) en píxeles absolutos
        
    Returns:
        numpy array RGB o None si falla
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
                # mss devuelve BGRA, convertir a RGB
                img_array = np.array(screenshot)
                return img_array[:, :, :3][:, :, ::-1]  # BGRA -> RGB
        else:
            screenshot = ImageGrab.grab(bbox=bbox)
            return np.array(screenshot)
    except Exception as e:
        print(f"[ERROR] capture_screen_region: {e}", file=sys.stderr)
        return None


def capture_full_screen() -> Optional[np.ndarray]:
    """Captura la pantalla completa."""
    try:
        if MSS_AVAILABLE:
            with mss.mss() as sct:
                monitor = sct.monitors[1]  # Monitor principal
                screenshot = sct.grab(monitor)
                img_array = np.array(screenshot)
                return img_array[:, :, :3][:, :, ::-1]
        else:
            screenshot = ImageGrab.grab()
            return np.array(screenshot)
    except Exception as e:
        print(f"[ERROR] capture_full_screen: {e}", file=sys.stderr)
        return None


# =============================================================================
# FUNCIONES DE GESTIÓN DE VENTANAS
# =============================================================================

def get_poker_windows() -> List[str]:
    """Obtiene la lista de ventanas disponibles."""
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
    Obtiene el bounding box de una ventana por título.
    
    Returns:
        (left, top, right, bottom) o None
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
    Captura una ventana específica y devuelve la imagen.
    
    Args:
        window_title: Título de la ventana a capturar
        
    Returns:
        Dict con imagen base64, dimensiones, o error
    """
    try:
        bbox = get_window_bbox(window_title)
        
        if not bbox:
            return {"error": f"Window not found: {window_title}"}
        
        img_array = capture_screen_region(bbox)
        
        if img_array is None:
            return {"error": "Failed to capture screen"}
        
        # Convertir a base64
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


# =============================================================================
# FUNCIONES DE ANÁLISIS DE IMAGEN
# =============================================================================

def extract_region(image: np.ndarray, region: Tuple[float, float, float, float]) -> np.ndarray:
    """
    Extrae una región de la imagen usando coordenadas relativas (0-1).
    
    Args:
        image: numpy array de la imagen
        region: (x1, y1, x2, y2) como fracciones 0-1
        
    Returns:
        numpy array de la región recortada
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
    Detecta cartas en una imagen usando el predictor.
    
    Args:
        image: numpy array de la imagen (ya recortada)
        region_name: Nombre de la región para logging
        
    Returns:
        Lista de cartas detectadas con formato compatible con Rust
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
            # Mapear rank string a número
            rank_map = {
                'A': 14, 'K': 13, 'Q': 12, 'J': 11, 
                '10': 10, 'T': 10,
                '9': 9, '8': 8, '7': 7, '6': 6, 
                '5': 5, '4': 4, '3': 3, '2': 2
            }
            
            # Mapear suit a letra
            suit_map = {
                'hearts': 'h', 'diamonds': 'd', 
                'clubs': 'c', 'spades': 's',
                'h': 'h', 'd': 'd', 'c': 'c', 's': 's'
            }
            
            rank = rank_map.get(card.rank.upper(), 0)
            suit = suit_map.get(card.suit.lower(), 'h')
            
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
    Extrae un número de una región usando EasyOCR.
    
    Args:
        image: Imagen completa como numpy array
        region: Coordenadas relativas (0-1)
        region_name: Nombre de la región
        
    Returns:
        Valor numérico extraído o 0.0
    """
    if not OCR_AVAILABLE or number_extractor is None:
        return 0.0
    
    try:
        # Convertir región relativa a absoluta
        height, width = image.shape[:2]
        x1, y1, x2, y2 = region
        roi = (int(x1 * width), int(y1 * height), int(x2 * width), int(y2 * height))
        
        # Extraer números de la región
        results = number_extractor.extract_from_region(image, roi)
        
        if results:
            # Devolver el valor más grande encontrado (suele ser el correcto para pot)
            values = [r[0] for r in results if r[0] > 0]
            if values:
                return max(values)
        
        return 0.0
        
    except Exception as e:
        print(f"[ERROR] extract_number_from_region ({region_name}): {e}", file=sys.stderr)
        return 0.0


def count_active_opponents(image: np.ndarray) -> int:
    """
    Cuenta el número de oponentes activos en la mesa.
    
    TODO: Implementar detección de oponentes con YOLO o heurísticas
    """
    # Por ahora, devolver 1 como placeholder
    # En el futuro: detectar fichas/avatares en posiciones de oponentes
    return 1


def analyze_game_state(image: np.ndarray) -> Dict[str, Any]:
    """
    Analiza una imagen completa de la mesa de poker y extrae el estado del juego.
    
    Args:
        image: numpy array RGB de la mesa de poker
    
    Returns:
        Dict compatible con CvState de Rust (poker_agent)
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
        # 1. Detectar cartas de la mano
        hand_region = extract_region(image, DEFAULT_REGIONS["hand"])
        hand_cards = detect_cards_in_region(hand_region, "hand")
        result["hand"] = hand_cards[:2]  # Máximo 2 cartas
        
        # 2. Detectar cartas comunitarias
        community_region = extract_region(image, DEFAULT_REGIONS["community"])
        community_cards = detect_cards_in_region(community_region, "community")
        result["community_cards"] = community_cards[:5]  # Máximo 5 cartas
        
        # 3. Extraer valores numéricos con OCR
        result["pot"] = extract_number_from_region(
            image, DEFAULT_REGIONS["pot"], "pot"
        )
        result["call_amount"] = extract_number_from_region(
            image, DEFAULT_REGIONS["call"], "call"
        )
        
        # 4. Contar oponentes
        num_opponents = count_active_opponents(image)
        result["num_opponents"] = num_opponents
        result["opponents"] = [{"opponent_bet": 0.0} for _ in range(num_opponents)]
        
        # 5. Calcular confianza promedio
        all_cards = result["hand"] + result["community_cards"]
        if all_cards:
            confidences = [c.get("confidence", 0) for c in all_cards]
            result["confidence_avg"] = sum(confidences) / len(confidences)
        
        # 6. Estado de la detección
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
    Captura una ventana y analiza su contenido.
    
    Args:
        window_title: Título de la ventana a capturar y analizar
        
    Returns:
        Estado del juego o error
    """
    # Obtener bbox de la ventana
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
    
    # Capturar la región
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


# =============================================================================
# LOOP PRINCIPAL DE COMUNICACIÓN
# =============================================================================

def main():
    """
    Loop principal que procesa comandos desde stdin.
    
    Comandos soportados:
    - ping: Verificar conectividad
    - status: Estado del sidecar
    - list_windows: Listar ventanas disponibles
    - capture: Capturar ventana como imagen
    - analyze: Analizar ventana y extraer estado
    - analyze_image: Analizar imagen base64 proporcionada
    - set_regions: Actualizar regiones de detección
    """
    # Mensaje de inicio (stderr para debug)
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
            
            # ====== Comandos de diagnóstico ======
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
            
            # ====== Comandos de ventanas ======
            elif cmd_type == "list_windows":
                response = {"windows": get_poker_windows()}
            
            elif cmd_type == "capture":
                window_title = command.get("window_title", "")
                response = capture_window(window_title)
            
            # ====== Comandos de análisis ======
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
            
            # ====== Comandos de configuración ======
            elif cmd_type == "set_regions":
                new_regions = command.get("regions", {})
                for key in new_regions:
                    if key in DEFAULT_REGIONS:
                        DEFAULT_REGIONS[key] = tuple(new_regions[key])
                response = {"status": "ok", "regions": DEFAULT_REGIONS}
            
            elif cmd_type == "get_regions":
                response = {"status": "ok", "regions": DEFAULT_REGIONS}
            
            # ====== Comando desconocido ======
            else:
                response = {"error": f"Unknown command: {cmd_type}"}
            
            # Enviar respuesta
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
