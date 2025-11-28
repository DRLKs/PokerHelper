# Changelog - Computer Vision Module

Todos los cambios notables en el módulo `computer_vision` serán documentados aquí.

## [3.0.0] - 2025-01-XX

### 🚀 Cambio de Stack Tecnológico

Esta versión reemplaza TensorFlow/Pytesseract con YOLO v8 + EasyOCR, optimizado para CPU.

#### Tecnologías Actualizadas

| Anterior | Nuevo | Razón |
|----------|-------|-------|
| TensorFlow (clasificación) | YOLO v8 (detección) | Más rápido, detección end-to-end |
| Pytesseract (OCR) | EasyOCR | Mejor con fuentes estilizadas |
| PIL.ImageGrab | mss | Más rápido, cross-platform |

### ✨ Nuevos Componentes

#### `predictor/core/yolo_detector.py`
Detector de cartas basado en YOLO v8:
- Detección y clasificación en un solo paso
- Optimizado para CPU (sin CUDA requerido)
- Soporta modelos custom y pretrained
- Funciones: `detect_cards()`, `detect_and_classify()`

#### `predictor/core/number_extractor.py`
Extractor de números basado en EasyOCR:
- Maneja formatos de poker (k, M, B suffixes)
- Extracción por regiones (ROI)
- Preprocesamiento de imagen incluido
- Funciones: `extract_numbers()`, `extract_from_region()`

### 📦 Dependencias Actualizadas

```txt
# requirements.txt
ultralytics>=8.0.0      # YOLO v8
easyocr>=1.7.0          # OCR
torch>=2.0.0            # CPU version
mss>=9.0.0              # Screen capture
opencv-python>=4.8.0    # Image processing
numpy>=1.24.0
Pillow>=9.0.0
pydantic>=2.0.0
ewmh>=0.1.6             # Linux window management
python-xlib>=0.33       # Required by ewmh
```

### 📐 CardPredictor Actualizado

El `CardPredictor` ahora usa el nuevo stack:

```python
from predictor import CardPredictor, get_predictor

# Crear predictor (singleton)
predictor = get_predictor()

# Analizar imagen
result = predictor.analyze_image("screenshot.png")

# Acceder a resultados
for card in result.cards:
    print(f"{card.rank}{card.suit} (conf: {card.confidence:.2f})")

print(f"Pot: ${result.pot_size}")
print(f"Tiempo: {result.processing_time_ms:.2f}ms")
```

### 📁 Estructura de Archivos

```
predictor/
├── __init__.py              # Exports principales
├── CardPredictor.py         # Orquestador YOLO + EasyOCR
└── core/
    ├── __init__.py          # Exports del core
    ├── yolo_detector.py     # Detector de cartas YOLO v8
    └── number_extractor.py  # Extractor OCR (EasyOCR)
```

### 🗑️ Código Legacy Eliminado

Los siguientes archivos fueron eliminados en favor del nuevo stack:
- `card_detector.py` (reemplazado por `yolo_detector.py`)
- `image_preprocessor.py` (innecesario con YOLO)
- `model_loader.py` (TensorFlow eliminado)
- `visualizer.py` (innecesario)
- `Predictor.py` (wrapper obsoleto)

---

## [2.0.0] - 2025-11-28

### 🔄 Cambios Importantes (Breaking Changes)

Esta versión actualiza completamente la estructura de datos para alinearse con el módulo `poker_agent` de Rust.

#### Nuevo formato de respuesta

El sidecar ahora devuelve un objeto `GameState` completo en lugar de solo detecciones de cartas:

```json
{
    "status": "ok",
    "small_blind": 10.0,
    "pot": 150.0,
    "call_amount": 20.0,
    "num_opponents": 3,
    "opponents": [
        {"opponent_bet": 20.0},
        {"opponent_bet": 0.0},
        {"opponent_bet": 10.0}
    ],
    "hand": [
        {"suit": "h", "rank": 14, "confidence": 0.95},
        {"suit": "s", "rank": 13, "confidence": 0.92}
    ],
    "community_cards": [
        {"suit": "c", "rank": 7, "confidence": 0.88},
        {"suit": "d", "rank": 2, "confidence": 0.91},
        {"suit": "h", "rank": 10, "confidence": 0.87}
    ],
    "timestamp": "2025-11-28T12:00:00Z",
    "processing_time_ms": 45.2,
    "confidence_avg": 0.906
}
```

### ✨ Nuevas Funcionalidades

#### Nuevos comandos del sidecar

| Comando | Descripción |
|---------|-------------|
| `analyze` | Captura una ventana y devuelve el estado completo del juego |
| `analyze_image` | Analiza una imagen base64 proporcionada |
| `status` | Devuelve información del sistema (predictor disponible, plataforma, regiones) |
| `set_regions` | Actualiza las regiones de detección en tiempo de ejecución |

#### Nuevos modelos de datos

- **`CardModel`**: Representa una carta con `suit`, `rank` y `confidence`
- **`OpponentModel`**: Representa el estado de un oponente
- **`GameStateResponse`**: Estado completo del juego compatible con Rust
- **`ScreenRegions`**: Configuración de regiones de pantalla

### 📁 Archivos Modificados

| Archivo | Cambio |
|---------|--------|
| `cv_sidecar.py` | Reescrito completamente para soportar análisis de estado del juego |
| `api/models/game_state.py` | **NUEVO** - Modelos Pydantic para el estado del juego |
| `api/models/__init__.py` | Actualizado para exportar nuevos modelos |

### 🗺️ Mapeo de Datos

#### Formato de cartas

| Dataset (cl1) | Palo (suit) | Descripción |
|---------------|-------------|-------------|
| `T` | `c` (clubs) | Tréboles ♣ |
| `P` | `s` (spades) | Picas ♠ |
| `C` | `h` (hearts) | Corazones ♥ |
| `D` | `d` (diamonds) | Diamantes ♦ |

| Dataset (cl1) | Rank | Descripción |
|---------------|------|-------------|
| `1` | `14` | As (A) |
| `2-10` | `2-10` | Número |
| `11` | `11` | Jota (J) |
| `12` | `12` | Reina (Q) |
| `13` | `13` | Rey (K) |

### 📐 Regiones de Detección por Defecto

Las coordenadas son relativas (0.0 a 1.0) respecto al tamaño de la imagen:

```python
DEFAULT_REGIONS = {
    "hand": (0.40, 0.70, 0.60, 0.90),        # Cartas de mano
    "community": (0.30, 0.35, 0.70, 0.50),   # Cartas comunitarias
    "pot": (0.40, 0.25, 0.60, 0.35),         # Bote (OCR)
    "call": (0.45, 0.85, 0.55, 0.95),        # Cantidad a call (OCR)
}
```

### 🔧 TODO / Pendiente

- [x] Implementar OCR para extracción de valores numéricos (pot, call_amount) ✅ v3.0
- [x] Reemplazar TensorFlow con YOLO v8 ✅ v3.0
- [ ] Implementar detección de oponentes activos
- [ ] Calibrar regiones para diferentes plataformas de poker
- [ ] Añadir soporte para múltiples resoluciones de pantalla

---

## [1.0.0] - Versión Anterior

### Funcionalidades Originales
- Detección básica de cartas usando modelo de clasificación
- Comandos: `list_windows`, `capture`, `ping`
- Soporte para Linux (ewmh) y Windows (pygetwindow)
