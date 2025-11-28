# Computer Vision Sidecar

Módulo de visión por computadora para el agente de poker. Detecta cartas y extrae valores numéricos de la pantalla usando YOLO v8 + EasyOCR.

## Arquitectura

```
computer_vision/
├── cv_sidecar.py          # Punto de entrada (stdin/stdout JSON)
├── requirements.txt       # Dependencias
├── predictor/
│   ├── __init__.py
│   ├── CardPredictor.py   # Orquestador principal
│   └── core/
│       ├── __init__.py
│       ├── yolo_detector.py      # Detección de cartas (YOLO v8)
│       └── number_extractor.py   # Extracción de números (EasyOCR)
└── RoboFlowDataset/       # Dataset para entrenar YOLO
```

## Tecnología

| Componente | Tecnología | Propósito |
|------------|------------|-----------|
| Detección de cartas | YOLO v8 (ultralytics) | Identificar cartas en pantalla |
| Extracción de texto | EasyOCR | Leer pot, bets, stacks |
| Captura de pantalla | mss | Cross-platform, rápido |
| Ventanas (Linux) | ewmh + python-xlib | Listar/posicionar ventanas |

> **Nota:** Todo optimizado para CPU. No requiere GPU.

## Instalación

```bash
cd computer_vision
python -m venv .venv
source .venv/bin/activate
pip install -r requirements.txt
```

## Uso

El sidecar se comunica via stdin/stdout con JSON:

```bash
# Ejemplo manual
echo '{"command": "ping"}' | python cv_sidecar.py
# {"success": true, "data": {"message": "pong", "version": "3.0.0"}}
```

### Comandos Disponibles

| Comando | Descripción |
|---------|-------------|
| `ping` | Health check |
| `status` | Estado del sidecar (modelo cargado, regiones) |
| `list_windows` | Listar ventanas disponibles (Linux) |
| `capture` | Capturar screenshot de ventana/región |
| `analyze` | Capturar + detectar cartas + extraer números |
| `analyze_image` | Analizar imagen base64 |
| `set_regions` | Configurar regiones de interés |

### Ejemplo: Analizar pantalla

```json
{"command": "analyze", "window_title": "PokerStars"}
```

Respuesta:
```json
{
  "success": true,
  "data": {
    "cards": [
      {"card": "As", "confidence": 0.95, "bbox": [100, 200, 150, 280]},
      {"card": "Kh", "confidence": 0.92, "bbox": [160, 200, 210, 280]}
    ],
    "numbers": [
      {"value": 1500.0, "region": "pot", "raw_text": "1,500"},
      {"value": 50.0, "region": "bet", "raw_text": "50"}
    ]
  }
}
```

## Modelo YOLO

El detector espera un modelo YOLO v8 entrenado para detectar las 52 cartas de poker.

### Clases del modelo (52 cartas)

```
2c, 2d, 2h, 2s, 3c, 3d, 3h, 3s, 4c, 4d, 4h, 4s,
5c, 5d, 5h, 5s, 6c, 6d, 6h, 6s, 7c, 7d, 7h, 7s,
8c, 8d, 8h, 8s, 9c, 9d, 9h, 9s, Tc, Td, Th, Ts,
Jc, Jd, Jh, Js, Qc, Qd, Qh, Qs, Kc, Kd, Kh, Ks,
Ac, Ad, Ah, As
```

### Ubicación del modelo

```
computer_vision/models/cards.pt
```

Ver `docs/DATASET_GUIDE.md` para entrenar un modelo personalizado.

## Integración con Tauri

El sidecar es invocado por Rust via `tauri::api::process::Command`:

```rust
let (rx, child) = Command::new_sidecar("cv_sidecar")?
    .spawn()?;

// Enviar comando
child.write(r#"{"command": "analyze"}"#.as_bytes())?;

// Leer respuesta
let response = rx.recv().await?;
```

## Desarrollo

```bash
# Ejecutar tests
pytest tests/

# Verificar tipos
mypy predictor/

# Formato
black predictor/ cv_sidecar.py
```

## Troubleshooting

### EasyOCR descarga modelos la primera vez
Es normal. Los modelos se cachean en `~/.EasyOCR/`.

### "No module named 'ewmh'"
Solo disponible en Linux. En Windows usar `pygetwindow`.

### YOLO muy lento
Asegúrate de usar la versión `n` (nano) del modelo:
```python
model = YOLO("yolov8n.pt")  # Más rápido
```
