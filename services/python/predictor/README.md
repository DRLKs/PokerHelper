
## Pasos que sigue el método de predicción de imágenes - CardPredictor.analyze_image

# Encontramos las cartas 'card_detector.py'
Mediante técnicas de procesamiento de imágenes para la detección de bordes, encontramos los contornos que delimitan las cartas. 
Con las coordenadas de estos bordes podemos recortar las cartas en imágenes separadas.

# Normalización de las cartas 'image_preprocess.py'
Las imágenes de cada carta es normalizada, la resolución de estas se iguala.

# Predecimos el valor de las cartas
Con el modelo anteriormente cargado, predecimos el palo y el valor de cada carta detectada.