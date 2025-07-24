# Sistema de Predicción de Cartas de Póker

Sistema modular para detectar y predecir cartas de póker en imágenes, especialmente diseñado para capturas de pantalla de PokerStars.

## 📁 Estructura del Proyecto

### Archivos Principales

- **`main_test.py`** - 🧪 Script principal de pruebas completas
- **`card_predictor.py`** - 🎯 Predictor principal integrado  
- **`card_detector.py`** - 👁️ Detecta regiones de cartas en imágenes
- **`image_preprocessor.py`** - 🖼️ Preprocesa imágenes para el modelo
- **`model_loader.py`** - 🧠 Carga y maneja modelos de TensorFlow
- **`visualizer.py`** - 📊 Visualiza resultados de detección/predicción
- **`Predictor.py`** - 🔄 Archivo de compatibilidad (legacy)

### Archivos Auxiliares

- **`model.py`** - 🏋️ Script de entrenamiento del modelo
- **`AnalizadorCartas.py`** - 📸 Capturador de pantallas de PokerStars
- **`requirements.txt`** - 📦 Dependencias del proyecto

## 🚀 Instalación

### 1. Instalar Python

Para Ubuntu/Debian:
```bash
sudo apt update
sudo apt install python3 python3-pip python3-venv
```

Para otras distribuciones, usa tu gestor de paquetes correspondiente.

### 2. Crear entorno virtual (recomendado)

```bash
cd /home/drlk/GitHub/PokerHelper/services/python
python3 -m venv venv
source venv/bin/activate  # En Linux/Mac
# o venv\Scripts\activate  # En Windows
```

### 3. Instalar dependencias

```bash
pip install -r requirements.txt
```

## 🧪 Pruebas

### Prueba Completa del Sistema
```bash
python main_test.py
```
Este script:
- Verifica todas las dependencias
- Prueba cada componente individualmente
- Ejecuta pruebas del sistema integrado
- Muestra un resumen completo

### Pruebas Individuales de Componentes

```bash
# Probar solo detección de cartas
python card_detector.py

# Probar solo preprocesamiento 
python image_preprocessor.py

# Probar carga de modelos
python model_loader.py

# Probar visualización
python visualizer.py
```

### Análisis Rápido de Imágenes
```bash
# Usando el predictor integrado
python card_predictor.py

# O usando el archivo de compatibilidad
python Predictor.py
```

## 💼 Uso en Código

### Uso Básico
```python
from card_predictor import CardPredictor

# Crear predictor
predictor = CardPredictor()

# Analizar imagen
results = predictor.analyze_image("ruta/imagen.jpg")

print(f"Cartas detectadas: {results['total_cards_detected']}")
```

### Uso Modular
```python
from card_detector import CardDetector
from model_loader import ModelLoader
from visualizer import ResultVisualizer

# Usar componentes por separado
detector = CardDetector()
loader = ModelLoader()
visualizer = ResultVisualizer()

# Detectar cartas
regions = detector.detect_card_regions(image)
crops = detector.extract_card_crops(image, regions)

# Cargar modelo y predecir
loader.load_model()
predictions = []
for crop in crops:
    pred = loader.predict(crop['image'])
    predictions.append(pred)

# Visualizar resultados
visualizer.visualize_crops(crops, predictions)
```

## 📂 Datos

### Estructura de Carpetas Necesarias

```
├── test/          # Imágenes para pruebas
├── train/         # Imágenes de entrenamiento  
├── valid/         # Imágenes de validación
├── anotaciones/   # Archivos CSV con anotaciones
└── modelos/       # Modelos entrenados (.h5)
```

### Formato de Anotaciones

Las anotaciones están en formato CSV con columnas:
- `filename`: Nombre del archivo de imagen
- `width`, `height`: Dimensiones de la imagen
- `cl1`: Clase de la carta (ej: "T 9", "C 1", "P 3")
- `xmin`, `ymin`, `xmax`, `ymax`: Coordenadas de la caja delimitadora

## 🎯 Funcionalidades

### Detección de Cartas
- Detección automática de regiones que contienen cartas
- Filtrado por área y relación de aspecto
- Extracción de recortes individuales

### Preprocesamiento
- Redimensionamiento automático
- Normalización de píxeles
- Mantenimiento de relación de aspecto

### Predicción
- Carga automática del modelo más reciente
- Predicción de clase con nivel de confianza
- Manejo de errores robusto

### Visualización  
- Visualización de detecciones sobre imagen original
- Muestra de recortes individuales
- Guardado de resultados visuales

## 🔧 Configuración

### Parámetros de Detección
```python
detector = CardDetector(
    min_area=5000,        # Área mínima para cartas
    max_area=50000,       # Área máxima para cartas
    min_aspect_ratio=0.5, # Ratio mín ancho/alto
    max_aspect_ratio=0.9  # Ratio máx ancho/alto
)
```

### Parámetros del Modelo
```python
predictor = CardPredictor(
    img_height=1200,  # Altura esperada por el modelo
    img_width=1920,   # Ancho esperado por el modelo
    model_path="modelos/model1.0.h5"  # Ruta específica del modelo
)
```

## 🐛 Solución de Problemas

### Error: "ModuleNotFoundError"
```bash
pip install -r requirements.txt
```

### Error: "No se encontró ningún modelo"
1. Entrena un modelo ejecutando `model.py`
2. O especifica una ruta de modelo manualmente

### Error: "No se encontraron imágenes de prueba"
1. Crea la carpeta `test/`
2. Coloca algunas imágenes .jpg o .png en esa carpeta

### Error: OpenCV no funciona
```bash
pip install opencv-python opencv-python-headless
```

## 🔄 Migración desde Predictor.py Original

El archivo `Predictor.py` original se mantiene para compatibilidad, pero ahora usa el sistema modular internamente. Para mejores funcionalidades, usa directamente `card_predictor.py`.

## 📈 Próximos Pasos

1. **Entrenar Modelo**: Ejecuta `model.py` para crear un modelo .h5
2. **Pruebas**: Ejecuta `main_test.py` para verificar todo funcione
3. **Integración**: Usa `AnalizadorCartas.py` para capturas en tiempo real  
4. **Personalización**: Ajusta parámetros de detección según tus necesidades

## 📞 Soporte

Si encuentras problemas:
1. Ejecuta `main_test.py` para diagnóstico completo
2. Verifica que todas las dependencias estén instaladas
3. Asegúrate de tener imágenes de prueba en la carpeta `test/`
