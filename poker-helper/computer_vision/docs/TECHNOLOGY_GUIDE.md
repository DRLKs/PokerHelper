# Guía de Tecnologías: YOLO v8 + EasyOCR

Esta guía explica cómo funcionan las tecnologías de visión por computadora usadas en el módulo CV del proyecto.

## 📋 Índice

1. [YOLO v8 - Detección de Objetos](#yolo-v8---detección-de-objetos)
2. [EasyOCR - Reconocimiento de Texto](#easyocr---reconocimiento-de-texto)
3. [Integración en el Proyecto](#integración-en-el-proyecto)
4. [Optimización para CPU](#optimización-para-cpu)

---

## YOLO v8 - Detección de Objetos

### ¿Qué es YOLO?

**YOLO** (You Only Look Once) es una familia de modelos de detección de objetos en tiempo real. A diferencia de métodos tradicionales que analizan la imagen en múltiples pasos, YOLO procesa toda la imagen en una sola pasada de red neuronal.

### ¿Cómo funciona?

```
┌─────────────────────────────────────────────────────────────┐
│                      IMAGEN DE ENTRADA                      │
│                        (640x640 px)                         │
└─────────────────────────────────────────────────────────────┘
                              │
                              ▼
┌─────────────────────────────────────────────────────────────┐
│                    1. DIVISIÓN EN GRID                      │
│  ┌───┬───┬───┬───┬───┐                                      │
│  │   │   │   │   │   │  La imagen se divide en una          │
│  ├───┼───┼───┼───┼───┤  cuadrícula (ej: 20x20 celdas)       │
│  │   │ 🂡│   │   │   │                                      │
│  ├───┼───┼───┼───┼───┤  Cada celda es responsable de        │
│  │   │   │   │ 🂮│   │  detectar objetos cuyo centro        │
│  ├───┼───┼───┼───┼───┤  caiga dentro de ella.               │
│  │   │   │   │   │   │                                      │
│  └───┴───┴───┴───┴───┘                                      │
└─────────────────────────────────────────────────────────────┘
                              │
                              ▼
┌─────────────────────────────────────────────────────────────┐
│              2. PREDICCIÓN POR CELDA                        │
│                                                             │
│  Para cada celda, YOLO predice:                             │
│                                                             │
│  • Bounding Boxes (B): Coordenadas del rectángulo           │
│    - x, y (centro)                                          │
│    - w, h (ancho, alto)                                     │
│    - confidence (probabilidad de que haya objeto)           │
│                                                             │
│  • Clase: Probabilidad para cada una de las 52 cartas       │
│    [Ah: 0.02, 2h: 0.01, ..., Ks: 0.95, ...]                 │
└─────────────────────────────────────────────────────────────┘
                              │
                              ▼
┌─────────────────────────────────────────────────────────────┐
│           3. NON-MAXIMUM SUPPRESSION (NMS)                  │
│                                                             │
│  Problema: Múltiples celdas pueden detectar el mismo objeto │
│                                                             │
│  ┌─────────┐                    ┌─────────┐                 │
│  │ ┌─────┐ │   NMS elimina      │         │                 │
│  │ │ 🂡  │ │   boxes redundantes│   🂡    │                  │
│  │ └─────┘ │   ────────────►    │         │                 │
│  └─────────┘                    └─────────┘                 │
│   3 boxes                        1 box final                │
└─────────────────────────────────────────────────────────────┘
                              │
                              ▼
┌─────────────────────────────────────────────────────────────┐
│                    4. SALIDA FINAL                          │
│                                                             │
│  [                                                          │
│    {"class": "Ks", "conf": 0.95, "bbox": [100,200,50,70]},  │
│    {"class": "Ah", "conf": 0.92, "bbox": [200,200,50,70]}   │
│  ]                                                          │
└─────────────────────────────────────────────────────────────┘
```

### Arquitectura de YOLOv8

```
Input Image (640x640x3)
        │
        ▼
┌───────────────────┐
│     BACKBONE      │  Extrae características de la imagen
│   (CSPDarknet)    │  Convoluciones progresivas que reducen
│                   │  resolución pero aumentan profundidad
└───────────────────┘
        │
        ▼
┌───────────────────┐
│       NECK        │  Fusiona características de diferentes
│   (PANet + FPN)   │  escalas para detectar objetos pequeños
│                   │  y grandes simultáneamente
└───────────────────┘
        │
        ▼
┌───────────────────┐
│       HEAD        │  Produce las predicciones finales:
│   (Detect Layer)  │  bounding boxes + clases + confianza
└───────────────────┘
```

### Variantes de YOLOv8

| Modelo | Parámetros | Velocidad (CPU) | Precisión | Uso recomendado |
|--------|------------|-----------------|-----------|-----------------|
| YOLOv8n | 3.2M | ⚡⚡⚡⚡ ~45ms | mAP 37.3 | **CPU (este proyecto)** |
| YOLOv8s | 11.2M | ⚡⚡⚡ ~100ms | mAP 44.9 | CPU potente |
| YOLOv8m | 25.9M | ⚡⚡ ~300ms | mAP 50.2 | GPU recomendada |
| YOLOv8l | 43.7M | ⚡ ~500ms | mAP 52.9 | Solo GPU |
| YOLOv8x | 68.2M | 🐌 ~800ms | mAP 53.9 | Solo GPU |

### Código de uso

```python
from ultralytics import YOLO

# Cargar modelo
model = YOLO("cards.pt")  # Modelo entrenado para cartas

# Inferencia
results = model.predict(
    source="screenshot.png",
    conf=0.5,      # Umbral de confianza mínimo
    iou=0.45,      # Umbral para NMS
    device="cpu",  # Forzar CPU
    verbose=False
)

# Procesar resultados
for result in results:
    for box in result.boxes:
        clase = model.names[int(box.cls)]  # "Ah", "Ks", etc.
        confianza = float(box.conf)
        x1, y1, x2, y2 = box.xyxy[0].tolist()
```

---

## EasyOCR - Reconocimiento de Texto

### ¿Qué es OCR?

**OCR** (Optical Character Recognition) es la tecnología que convierte imágenes de texto en texto editable. EasyOCR es una biblioteca que usa deep learning para OCR multilenguaje.

### ¿Cómo funciona EasyOCR?

```
┌─────────────────────────────────────────────────────────────┐
│                    IMAGEN DE ENTRADA                        │
│                                                             │
│           ┌─────────────────────┐                           │
│           │   POT: $1,250       │                           │
│           └─────────────────────┘                           │
└─────────────────────────────────────────────────────────────┘
                              │
                              ▼
┌─────────────────────────────────────────────────────────────┐
│              1. DETECCIÓN DE TEXTO (CRAFT)                  │
│                                                             │
│  CRAFT (Character Region Awareness for Text) localiza       │
│  regiones donde hay texto en la imagen.                     │
│                                                             │
│  Salida: Bounding boxes de cada palabra/línea               │
│  ┌─────────────────────┐                                    │
│  │ [POT:] [$1,250]     │ ← Dos regiones detectadas          │
│  └─────────────────────┘                                    │
└─────────────────────────────────────────────────────────────┘
                              │
                              ▼
┌─────────────────────────────────────────────────────────────┐
│           2. RECONOCIMIENTO (CRNN + CTC)                    │
│                                                             │
│  Para cada región detectada:                                │
│                                                             │
│  ┌──────────────────────────────────────────────────┐       │
│  │                      CRNN                        │       │
│  │  ┌─────────┐   ┌─────────┐   ┌─────────┐         │       │
│  │  │   CNN   │ → │  LSTM   │ → │   CTC   │         │       │
│  │  │(features)│  │(secuencia)│ │(decode) │         │       │
│  │  └─────────┘   └─────────┘   └─────────┘         │       │
│  └──────────────────────────────────────────────────┘       │
│                                                             │
│  CNN: Extrae características visuales de cada carácter      │
│  LSTM: Modela la secuencia (contexto izq-derecha)           │
│  CTC: Decodifica la secuencia a texto                       │
└─────────────────────────────────────────────────────────────┘
                              │
                              ▼
┌─────────────────────────────────────────────────────────────┐
│                    3. SALIDA FINAL                          │
│                                                             │
│  [                                                          │
│    ("POT:", [[x1,y1], [x2,y2], ...], 0.98),                 │
│    ("$1,250", [[x1,y1], [x2,y2], ...], 0.95)                │
│  ]                                                          │
│                                                             │
│  Formato: (texto, coordenadas, confianza)                   │
└─────────────────────────────────────────────────────────────┘
```

### Arquitectura detallada

```
                    Imagen de región de texto
                              │
                              ▼
┌─────────────────────────────────────────────────────────────┐
│                          CNN                                │
│  ┌─────┐   ┌─────┐   ┌─────┐   ┌─────┐                      │
│  │Conv1│ → │Conv2│ → │Conv3│ → │Conv4│ → Feature Map        │
│  └─────┘   └─────┘   └─────┘   └─────┘                      │
│                                                             │
│  La imagen se convierte en una secuencia de features        │
│  Cada columna del feature map representa una "slice"        │
│  vertical de la imagen                                      │
└─────────────────────────────────────────────────────────────┘
                              │
                              ▼
┌─────────────────────────────────────────────────────────────┐
│                    LSTM Bidireccional                       │
│                                                             │
│     ←←← LSTM Backward ←←←                                   │
│  ┌───┐ ┌───┐ ┌───┐ ┌───┐ ┌───┐                              │
│  │ h │─│ h │─│ h │─│ h │─│ h │  Cada timestep procesa       │
│  └───┘ └───┘ └───┘ └───┘ └───┘  una columna del feature     │
│     →→→ LSTM Forward →→→                                    │
│                                                             │
│  El contexto bidireccional ayuda a resolver ambigüedades    │
│  (ej: distinguir "0" de "O")                                │
└─────────────────────────────────────────────────────────────┘
                              │
                              ▼
┌─────────────────────────────────────────────────────────────┐
│                    CTC Decoding                             │
│                                                             │
│  Entrada: Probabilidades por timestep                       │
│  [P,$,1,1,_,2,2,5,5,0,0]  (_ = blank token)                 │
│                                                             │
│  CTC colapsa repeticiones y elimina blanks:                 │
│  [P,$,1,_,2,5,0] → "$1,250"                                 │
│                                                             │
│  Ventaja: No requiere alineación carácter-por-carácter      │
│  durante el entrenamiento                                   │
└─────────────────────────────────────────────────────────────┘
```

### Código de uso

```python
import easyocr

# Crear reader (descarga modelos la primera vez)
reader = easyocr.Reader(
    ['en'],           # Idiomas
    gpu=False,        # Forzar CPU
    model_storage_directory='~/.EasyOCR'
)

# Leer texto de imagen
results = reader.readtext(
    image,
    detail=1,         # Incluir coordenadas
    paragraph=False,  # No agrupar en párrafos
    min_size=10,      # Tamaño mínimo de texto
    text_threshold=0.7
)

# Procesar resultados
for (bbox, text, confidence) in results:
    print(f"Texto: {text}, Confianza: {confidence:.2f}")
```

### Preprocesamiento para mejorar OCR

```python
import cv2
import numpy as np

def preprocess_for_ocr(image):
    """Preprocesa imagen para mejorar reconocimiento OCR."""
    
    # 1. Convertir a escala de grises
    gray = cv2.cvtColor(image, cv2.COLOR_BGR2GRAY)
    
    # 2. Escalar (EasyOCR funciona mejor con texto grande)
    scale = 2.0
    scaled = cv2.resize(gray, None, fx=scale, fy=scale, 
                        interpolation=cv2.INTER_CUBIC)
    
    # 3. Binarización adaptativa (separa texto del fondo)
    binary = cv2.adaptiveThreshold(
        scaled, 255,
        cv2.ADAPTIVE_THRESH_GAUSSIAN_C,
        cv2.THRESH_BINARY,
        11, 2
    )
    
    # 4. Reducir ruido
    denoised = cv2.fastNlMeansDenoising(binary, None, 10, 7, 21)
    
    return denoised
```

---

## Integración en el Proyecto

### Flujo de datos completo

```
┌─────────────────────────────────────────────────────────────┐
│                    cv_sidecar.py                            │
│                   (Punto de entrada)                        │
└─────────────────────────────────────────────────────────────┘
                              │
            ┌─────────────────┼─────────────────┐
            ▼                 ▼                 ▼
     ┌─────────────┐   ┌─────────────┐   ┌─────────────┐
     │   mss       │   │   YOLO      │   │  EasyOCR    │
     │  (captura)  │   │  (cartas)   │   │  (números)  │
     └─────────────┘   └─────────────┘   └─────────────┘
            │                 │                 │
            ▼                 ▼                 ▼
     ┌─────────────────────────────────────────────────────┐
     │                  CardPredictor                      │
     │           (Orquesta YOLO + EasyOCR)                 │
     └─────────────────────────────────────────────────────┘
                              │
                              ▼
     ┌─────────────────────────────────────────────────────┐
     │                   JSON Response                     │
     │  {                                                  │
     │    "cards": [{"card": "Ah", "confidence": 0.95}],   │
     │    "numbers": [{"value": 1250, "region": "pot"}]    │
     │  }                                                  │
     └─────────────────────────────────────────────────────┘
                              │
                              ▼
     ┌─────────────────────────────────────────────────────┐
     │              Rust Backend (Tauri)                   │
     │              poker_agent crate                      │
     └─────────────────────────────────────────────────────┘
```

### Separación de responsabilidades

| Componente | Archivo | Responsabilidad |
|------------|---------|-----------------|
| YOLOCardDetector | `yolo_detector.py` | Detectar y clasificar cartas |
| NumberExtractor | `number_extractor.py` | Extraer valores numéricos |
| CardPredictor | `CardPredictor.py` | Combinar ambos, formato de salida |
| cv_sidecar | `cv_sidecar.py` | Comunicación stdin/stdout con Rust |

---

## Optimización para CPU

### Por qué optimizar para CPU

En este proyecto no asumimos GPU disponible. Las optimizaciones clave:

### 1. Modelo YOLO pequeño

```python
# ❌ NO usar modelos grandes
model = YOLO("yolov8l.pt")  # 43.7M params, muy lento en CPU

# ✅ Usar modelo nano
model = YOLO("yolov8n.pt")  # 3.2M params, rápido en CPU
```

### 2. Resolución de imagen reducida

```python
# ❌ Resolución alta innecesaria
results = model.predict(image, imgsz=640)  # Más lento

# ✅ Resolución optimizada
results = model.predict(image, imgsz=416)  # 40% más rápido
```

### 3. PyTorch CPU-only

```bash
# requirements.txt
# ❌ PyTorch con CUDA (descarga pesada, no se usa)
torch>=2.0.0

# ✅ PyTorch solo CPU
--extra-index-url https://download.pytorch.org/whl/cpu
torch>=2.0.0
```

### 4. EasyOCR sin GPU

```python
# ✅ Forzar CPU explícitamente
reader = easyocr.Reader(['en'], gpu=False)
```

### 5. Caché de modelos

```python
# Singleton pattern para evitar recargar modelos
_detector = None

def get_detector():
    global _detector
    if _detector is None:
        _detector = YOLOCardDetector()
        _detector.load_model()
    return _detector
```

### Benchmarks esperados (CPU i5/Ryzen 5)

| Operación | Tiempo esperado |
|-----------|-----------------|
| Captura de pantalla (mss) | ~5ms |
| YOLO inferencia (416px) | ~50-100ms |
| EasyOCR (región pequeña) | ~100-200ms |
| **Total por frame** | **~200-350ms** |

---

## Recursos adicionales

- [Documentación Ultralytics YOLO](https://docs.ultralytics.com/)
- [Repositorio EasyOCR](https://github.com/JaidedAI/EasyOCR)
- [Paper YOLO original](https://arxiv.org/abs/1506.02640)
- [Paper CRAFT (detección de texto)](https://arxiv.org/abs/1904.01941)
- [Guía de entrenamiento YOLO](./DATASET_GUIDE.md)
