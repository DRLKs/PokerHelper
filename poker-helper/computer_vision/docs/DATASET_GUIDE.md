# Guía de Preparación del Dataset de Entrenamiento

Esta guía explica cómo preparar y formatear correctamente las imágenes y anotaciones para entrenar el modelo YOLO v8 de detección de cartas de poker.

## 📋 Índice

1. [Estructura del Dataset](#estructura-del-dataset)
2. [Formato de Anotaciones YOLO](#formato-de-anotaciones-yolo)
3. [Nomenclatura de Clases](#nomenclatura-de-clases)
4. [Cómo Anotar Imágenes](#cómo-anotar-imágenes)
5. [Entrenar el Modelo YOLO](#entrenar-el-modelo-yolo)
6. [Validación del Dataset](#validación-del-dataset)
7. [Consejos para Mejores Resultados](#consejos-para-mejores-resultados)

---

## 📁 Estructura del Dataset

### Formato YOLO v8 (Recomendado)

```
dataset/
├── data.yaml                 # Configuración del dataset
├── train/
│   ├── images/
│   │   ├── imagen001.jpg
│   │   └── ...
│   └── labels/
│       ├── imagen001.txt
│       └── ...
├── valid/
│   ├── images/
│   │   └── ...
│   └── labels/
│       └── ...
└── test/
    ├── images/
    │   └── ...
    └── labels/
        └── ...
```

### Archivo data.yaml

```yaml
# data.yaml
path: /path/to/dataset
train: train/images
val: valid/images
test: test/images

# Clases (52 cartas)
names:
  0: Ah   # As de corazones
  1: 2h   # 2 de corazones
  2: 3h
  # ... hasta 51
  51: Ks  # Rey de picas
```

### Distribución recomendada

| Conjunto | Porcentaje | Uso |
|----------|------------|-----|
| Train | 70% | Entrenamiento del modelo |
| Valid | 20% | Validación durante entrenamiento |
| Test | 10% | Evaluación final |

---

## 📄 Formato de Anotaciones YOLO

Cada imagen tiene un archivo `.txt` correspondiente con el mismo nombre.

### Formato de línea

```
<class_id> <x_center> <y_center> <width> <height>
```

Donde todos los valores están **normalizados (0.0 a 1.0)**:
- `x_center`: Centro X del bounding box / ancho de imagen
- `y_center`: Centro Y del bounding box / alto de imagen
- `width`: Ancho del bounding box / ancho de imagen
- `height`: Alto del bounding box / alto de imagen

### Ejemplo

Para una imagen `mesa_poker_01.jpg` de 1920x1080 con un As de corazones:

```
# mesa_poker_01.txt
0 0.45 0.65 0.08 0.12
```

Esto representa:
- Clase 0 (Ah - As de corazones)
- Centro en (864, 702) píxeles
- Tamaño 154x130 píxeles

---

## 🎴 Nomenclatura de Clases

### Formato de etiqueta: `{Rank}{Suit}`

### Ranks (Valores)

| Símbolo | Nombre | Rank (Rust) |
|---------|--------|-------------|
| `A` | As | `14` |
| `2` | Dos | `2` |
| `3` | Tres | `3` |
| `4` | Cuatro | `4` |
| `5` | Cinco | `5` |
| `6` | Seis | `6` |
| `7` | Siete | `7` |
| `8` | Ocho | `8` |
| `9` | Nueve | `9` |
| `T` o `10` | Diez | `10` |
| `J` | Jota | `11` |
| `Q` | Reina | `12` |
| `K` | Rey | `13` |

### Suits (Palos)

| Código | Nombre | Símbolo | Rust (suit) |
|--------|--------|---------|-------------|
| `h` | Hearts | ♥ | `'h'` |
| `d` | Diamonds | ♦ | `'d'` |
| `c` | Clubs | ♣ | `'c'` |
| `s` | Spades | ♠ | `'s'` |

### Mapeo de 52 clases

```python
# Generar las 52 clases
classes = []
ranks = ['A', '2', '3', '4', '5', '6', '7', '8', '9', 'T', 'J', 'Q', 'K']
suits = ['h', 'd', 'c', 's']

for suit in suits:
    for rank in ranks:
        classes.append(f"{rank}{suit}")

# Resultado:
# 0: Ah, 1: 2h, 2: 3h, ..., 12: Kh
# 13: Ad, 14: 2d, ..., 25: Kd
# 26: Ac, 27: 2c, ..., 38: Kc
# 39: As, 40: 2s, ..., 51: Ks
```

### Mapeo legacy (formato CSV anterior)

| Legacy (cl1) | YOLO | Descripción |
|--------------|------|-------------|
| `C 1` | `Ah` (0) | As de Corazones |
| `C 13` | `Kh` (12) | Rey de Corazones |
| `D 1` | `Ad` (13) | As de Diamantes |
| `T 1` | `Ac` (26) | As de Tréboles |
| `P 1` | `As` (39) | As de Picas |

---

## ✏️ Cómo Anotar Imágenes

### Opción 1: Roboflow (Recomendado)

1. Crea una cuenta en [roboflow.com](https://roboflow.com)
2. Crea un nuevo proyecto de tipo "Object Detection"
3. Sube tus imágenes
4. Anota las cartas con bounding boxes
5. Usa etiquetas en formato `{Rank}{Suit}` (ej: `Ah`, `Kd`, `7c`)
6. Exporta en formato **YOLOv8**

### Opción 2: LabelImg

```bash
pip install labelImg
labelImg
```

1. Abre la carpeta de imágenes
2. Cambia el formato a YOLO
3. Dibuja rectángulos y asigna clases
4. Guarda las anotaciones

### Opción 3: CVAT

1. Instala CVAT localmente o usa [app.cvat.ai](https://app.cvat.ai)
2. Crea un proyecto con las 52 clases
3. Anota las imágenes
4. Exporta en formato YOLO

---

## 🚀 Entrenar el Modelo YOLO

### Instalación

```bash
pip install ultralytics
```

### Entrenamiento básico

```python
from ultralytics import YOLO

# Cargar modelo base
model = YOLO('yolov8n.pt')  # nano (más rápido)
# model = YOLO('yolov8s.pt')  # small (balance)
# model = YOLO('yolov8m.pt')  # medium (más preciso)

# Entrenar
results = model.train(
    data='dataset/data.yaml',
    epochs=100,
    imgsz=640,
    batch=16,
    device='cpu',  # o 'cuda:0' si tienes GPU
    patience=20,
    save=True,
    project='runs/poker_cards'
)
```

### Entrenamiento optimizado para CPU

```python
from ultralytics import YOLO

model = YOLO('yolov8n.pt')

results = model.train(
    data='dataset/data.yaml',
    epochs=50,
    imgsz=416,        # Menor resolución para CPU
    batch=8,          # Batch más pequeño
    device='cpu',
    workers=4,        # Ajustar según CPU cores
    cache=True,       # Cachear imágenes en RAM
    amp=False,        # Desactivar mixed precision en CPU
    patience=15,
    optimizer='SGD',
    lr0=0.01,
    lrf=0.01,
)
```

### Exportar modelo entrenado

```python
# El mejor modelo se guarda en runs/poker_cards/weights/best.pt
# Para usar en el proyecto:

from ultralytics import YOLO
model = YOLO('runs/poker_cards/weights/best.pt')

# Inferencia
results = model.predict('test_image.jpg', conf=0.5)
```

---

## ✅ Validación del Dataset

### Script de validación

```python
import os
from pathlib import Path

def validate_yolo_dataset(dataset_path):
    """Valida estructura y formato de dataset YOLO."""
    errors = []
    warnings = []
    
    base = Path(dataset_path)
    
    # Verificar data.yaml
    if not (base / 'data.yaml').exists():
        errors.append("Falta data.yaml")
    
    # Verificar carpetas
    for split in ['train', 'valid', 'test']:
        img_dir = base / split / 'images'
        lbl_dir = base / split / 'labels'
        
        if not img_dir.exists():
            errors.append(f"Falta carpeta: {split}/images")
            continue
        
        if not lbl_dir.exists():
            errors.append(f"Falta carpeta: {split}/labels")
            continue
        
        # Verificar que cada imagen tenga su label
        images = list(img_dir.glob('*.jpg')) + list(img_dir.glob('*.png'))
        
        for img in images:
            label = lbl_dir / (img.stem + '.txt')
            if not label.exists():
                warnings.append(f"Sin anotación: {img.name}")
                continue
            
            # Validar formato de label
            with open(label) as f:
                for i, line in enumerate(f):
                    parts = line.strip().split()
                    if len(parts) != 5:
                        errors.append(f"{label.name}:{i+1} - Formato incorrecto")
                        continue
                    
                    try:
                        class_id = int(parts[0])
                        values = [float(p) for p in parts[1:]]
                        
                        if class_id < 0 or class_id > 51:
                            errors.append(f"{label.name}:{i+1} - Class ID inválido: {class_id}")
                        
                        for v in values:
                            if v < 0 or v > 1:
                                errors.append(f"{label.name}:{i+1} - Valor fuera de rango: {v}")
                    except ValueError:
                        errors.append(f"{label.name}:{i+1} - Valores no numéricos")
    
    return errors, warnings

# Uso
errors, warnings = validate_yolo_dataset('dataset/')
print(f"Errores: {len(errors)}")
print(f"Warnings: {len(warnings)}")
for e in errors[:10]:
    print(f"  ❌ {e}")
```

---

## 💡 Consejos para Mejores Resultados

### Diversidad de datos

- [ ] Incluir diferentes clientes de poker (PokerStars, 888, etc.)
- [ ] Variar resoluciones de pantalla
- [ ] Incluir diferentes temas/skins
- [ ] Capturar en diferentes condiciones de iluminación
- [ ] Incluir cartas parcialmente visibles (oclusión)

### Calidad de anotaciones

- [ ] El bounding box debe incluir toda la carta
- [ ] Dejar un pequeño margen (2-5%) alrededor
- [ ] No incluir elementos que no sean la carta
- [ ] Anotar TODAS las cartas visibles en cada imagen

### Cantidad mínima recomendada

| Conjunto | Mínimo | Recomendado |
|----------|--------|-------------|
| Train | 1000 imágenes | 5000+ imágenes |
| Valid | 200 imágenes | 1000+ imágenes |
| Test | 100 imágenes | 500+ imágenes |

### Balance de clases

```python
# Script para verificar balance
from collections import Counter

class_counts = Counter()
for label_file in Path('dataset/train/labels').glob('*.txt'):
    with open(label_file) as f:
        for line in f:
            class_id = int(line.split()[0])
            class_counts[class_id] += 1

# Ver distribución
for class_id, count in sorted(class_counts.items()):
    print(f"Clase {class_id}: {count}")
```

---

## 🔄 Conversión desde formato CSV legacy

```python
import pandas as pd
import os

def csv_to_yolo(csv_path, output_dir, class_mapping):
    """
    Convierte anotaciones CSV legacy a formato YOLO.
    
    class_mapping: Dict[str, int] que mapea 'T 1' -> 26, etc.
    """
    df = pd.read_csv(csv_path)
    os.makedirs(output_dir, exist_ok=True)
    
    for filename, group in df.groupby('filename'):
        label_file = os.path.join(output_dir, filename.rsplit('.', 1)[0] + '.txt')
        
        with open(label_file, 'w') as f:
            for _, row in group.iterrows():
                # Convertir clase
                class_id = class_mapping.get(row['cl1'], 0)
                
                # Normalizar coordenadas
                x_center = ((row['xmin'] + row['xmax']) / 2) / row['width']
                y_center = ((row['ymin'] + row['ymax']) / 2) / row['height']
                width = (row['xmax'] - row['xmin']) / row['width']
                height = (row['ymax'] - row['ymin']) / row['height']
                
                f.write(f"{class_id} {x_center:.6f} {y_center:.6f} {width:.6f} {height:.6f}\n")

# Crear mapping
def create_class_mapping():
    mapping = {}
    ranks = ['1', '2', '3', '4', '5', '6', '7', '8', '9', '10', '11', '12', '13']
    suits_legacy = ['C', 'D', 'T', 'P']  # Corazones, Diamantes, Tréboles, Picas
    suits_yolo = ['h', 'd', 'c', 's']
    
    class_id = 0
    for suit_l, suit_y in zip(suits_legacy, suits_yolo):
        for rank in ranks:
            mapping[f"{suit_l} {rank}"] = class_id
            class_id += 1
    
    return mapping

# Uso
mapping = create_class_mapping()
csv_to_yolo('anotaciones/train_annotations.csv', 'dataset/train/labels', mapping)
```

---

## ❓ FAQ

### ¿Qué modelo YOLO usar?

| Modelo | Velocidad | Precisión | CPU | GPU |
|--------|-----------|-----------|-----|-----|
| yolov8n | ⚡⚡⚡⚡ | ⭐⭐ | ✅ | ✅ |
| yolov8s | ⚡⚡⚡ | ⭐⭐⭐ | ✅ | ✅ |
| yolov8m | ⚡⚡ | ⭐⭐⭐⭐ | 🐌 | ✅ |
| yolov8l | ⚡ | ⭐⭐⭐⭐⭐ | ❌ | ✅ |

**Recomendación para CPU:** `yolov8n` o `yolov8s`

### ¿Cuántas épocas necesito?

- Mínimo: 50 épocas
- Recomendado: 100-200 épocas
- Con early stopping (`patience=20`) el entrenamiento parará si no mejora

### ¿Qué resolución de imagen usar?

| Resolución | Velocidad | Precisión |
|------------|-----------|-----------|
| 320 | ⚡⚡⚡⚡ | ⭐⭐ |
| 416 | ⚡⚡⚡ | ⭐⭐⭐ |
| 640 | ⚡⚡ | ⭐⭐⭐⭐ |

**Recomendación para CPU:** 416 o 320

---

## 📞 Soporte

Si tienes dudas sobre la preparación del dataset, revisa los ejemplos en `RoboFlowDataset/` o abre un issue en el repositorio.
