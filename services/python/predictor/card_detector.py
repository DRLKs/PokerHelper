"""
Detector de regiones de cartas usando técnicas de visión por computadora.
"""
import cv2
import numpy as np


class CardDetector:
    """
    Clase para detectar regiones donde pueden estar las cartas en una imagen.
    """
    
    def __init__(self, min_area=5000, max_area=50000, min_aspect_ratio=0.5, max_aspect_ratio=0.9):
        """
        Inicializa el detector de cartas.
        
        Args:
            min_area: Área mínima para considerar como carta
            max_area: Área máxima para considerar como carta  
            min_aspect_ratio: Ratio de aspecto mínimo (ancho/alto)
            max_aspect_ratio: Ratio de aspecto máximo (ancho/alto)
        """
        self.min_area = min_area
        self.max_area = max_area
        self.min_aspect_ratio = min_aspect_ratio
        self.max_aspect_ratio = max_aspect_ratio
    
    def detect_card_regions(self, image):
        """
        Detecta regiones donde pueden estar las cartas usando técnicas de visión por computadora.
        
        Args:
            image: Imagen como array numpy (RGB)
            
        Returns:
            list: Lista de coordenadas (x, y, w, h) de las regiones detectadas
        """
        # Convertir a escala de grises
        gray = cv2.cvtColor(image, cv2.COLOR_RGB2GRAY)
        
        # Aplicar filtro Gaussian blur
        blurred = cv2.GaussianBlur(gray, (5, 5), 0)
        
        # Detectar bordes
        edges = cv2.Canny(blurred, 50, 150)
        
        # Encontrar contornos
        contours, _ = cv2.findContours(edges, cv2.RETR_EXTERNAL, cv2.CHAIN_APPROX_SIMPLE)
        
        # Filtrar contornos que podrían ser cartas
        card_regions = []
        for contour in contours:
            # Calcular área
            area = cv2.contourArea(contour)
            
            # Filtrar por área
            if self.min_area < area < self.max_area:
                x, y, w, h = cv2.boundingRect(contour)
                
                # Filtrar por ratio de aspecto (las cartas tienen ratio aproximado de 0.7)
                aspect_ratio = w / h if h > 0 else 0
                if self.min_aspect_ratio < aspect_ratio < self.max_aspect_ratio:
                    card_regions.append((x, y, w, h))
        
        return card_regions
    
    def extract_card_crops(self, image, card_regions):
        """
        Extrae los recortes de las cartas de la imagen original.
        
        Args:
            image: Imagen original como array numpy
            card_regions: Lista de regiones de cartas (x, y, w, h)
            
        Returns:
            list: Lista de diccionarios con imágenes recortadas y metadatos
        """
        crops = []
        
        for i, (x, y, w, h) in enumerate(card_regions):
            # Asegurar que las coordenadas están dentro de los límites de la imagen
            x = max(0, x)
            y = max(0, y)
            w = min(w, image.shape[1] - x)
            h = min(h, image.shape[0] - y)
            
            # Extraer el recorte
            crop = image[y:y+h, x:x+w]
            
            if crop.size > 0:  # Verificar que el recorte no está vacío
                crops.append({
                    'image': crop,
                    'bbox': (x, y, w, h),
                    'id': i,
                    'area': w * h,
                    'aspect_ratio': w / h if h > 0 else 0
                })
        
        return crops


# Funciones de utilidad
def test_card_detection(image_path=None, image_array=None):
    """
    Función de prueba para detectar cartas en una imagen.
    
    Args:
        image_path: Ruta a la imagen (opcional)
        image_array: Array numpy de la imagen (opcional)
    """
    import matplotlib.pyplot as plt
    import matplotlib.patches as patches
    from PIL import Image
    
    # Cargar imagen
    if image_path:
        pil_img = Image.open(image_path)
        img_array = np.array(pil_img)
    elif image_array is not None:
        img_array = image_array
    else:
        print("Error: Proporciona image_path o image_array")
        return
    
    # Crear detector
    detector = CardDetector()
    
    # Detectar regiones
    regions = detector.detect_card_regions(img_array)
    
    print(f"Detectadas {len(regions)} posibles cartas")
    
    # Visualizar resultados
    fig, ax = plt.subplots(1, 1, figsize=(15, 10))
    ax.imshow(img_array)
    
    for i, (x, y, w, h) in enumerate(regions):
        # Dibujar rectángulo
        rect = patches.Rectangle((x, y), w, h, linewidth=2, 
                               edgecolor='red', facecolor='none')
        ax.add_patch(rect)
        
        # Añadir etiqueta
        ax.text(x, y-10, f"Carta {i+1}", color='red', fontsize=8,
               bbox=dict(boxstyle="round,pad=0.3", facecolor='white', alpha=0.8))
    
    ax.set_title(f'Regiones detectadas: {len(regions)}')
    ax.axis('off')
    plt.tight_layout()
    plt.show()
    
    return regions


if __name__ == "__main__":
    import os
    
    print("=== Test del Detector de Cartas ===")
    
    # Buscar imagen de prueba
    if os.path.exists('test'):
        test_images = [os.path.join('test', f) for f in os.listdir('test') 
                      if f.lower().endswith(('.jpg', '.jpeg', '.png'))]
        
        if test_images:
            print(f"Probando con: {test_images[0]}")
            test_card_detection(test_images[0])
        else:
            print("No se encontraron imágenes en la carpeta test/")
    else:
        print("Carpeta test/ no existe")
