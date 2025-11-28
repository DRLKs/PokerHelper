"""
Detector de cartas usando YOLO v8 (Ultralytics).
Optimizado para CPU.
"""
import os
import numpy as np
from typing import List, Dict, Any, Optional, Tuple
from pathlib import Path


class YOLOCardDetector:
    """
    Detector de cartas de poker usando YOLO v8.
    Diseñado para funcionar sin GPU.
    """
    
    # Mapeo de clases del dataset a formato interno
    # Formato dataset: "{PALO} {VALOR}" -> Formato interno: (suit, rank)
    SUIT_MAP = {
        'T': 'c',  # Tréboles -> clubs
        'P': 's',  # Picas -> spades
        'C': 'h',  # Corazones -> hearts
        'D': 'd',  # Diamantes -> diamonds
    }
    
    VALUE_MAP = {
        '1': 14,   # As
        '2': 2, '3': 3, '4': 4, '5': 5, '6': 6,
        '7': 7, '8': 8, '9': 9, '10': 10,
        '11': 11,  # J
        '12': 12,  # Q
        '13': 13,  # K
    }
    
    def __init__(self, model_path: Optional[str] = None, confidence_threshold: float = 0.5):
        """
        Inicializa el detector YOLO.
        
        Args:
            model_path: Ruta al modelo .pt entrenado. Si es None, busca en rutas por defecto.
            confidence_threshold: Umbral de confianza para detecciones (0-1)
        """
        self.confidence_threshold = confidence_threshold
        self.model = None
        self.model_path = model_path
        
        # Intentar cargar el modelo
        self._load_model()
    
    def _find_model(self) -> Optional[str]:
        """Busca el modelo en ubicaciones comunes."""
        possible_paths = [
            self.model_path,
            "models/poker_cards.pt",
            "models/best.pt",
            "../models/poker_cards.pt",
            "../models/best.pt",
            os.path.join(os.path.dirname(__file__), "../../models/poker_cards.pt"),
            os.path.join(os.path.dirname(__file__), "../../models/best.pt"),
        ]
        
        for path in possible_paths:
            if path and os.path.exists(path):
                return path
        
        return None
    
    def _load_model(self) -> bool:
        """Carga el modelo YOLO."""
        try:
            from ultralytics import YOLO
            
            model_path = self._find_model()
            
            if model_path:
                print(f"Cargando modelo YOLO desde: {model_path}")
                self.model = YOLO(model_path)
                # Forzar modo CPU
                self.model.to('cpu')
                return True
            else:
                print("Advertencia: No se encontró modelo YOLO entrenado.")
                print("Para entrenar un modelo, usa: yolo train data=dataset.yaml model=yolov8n.pt")
                return False
                
        except ImportError:
            print("Error: ultralytics no está instalado. Ejecuta: pip install ultralytics")
            return False
        except Exception as e:
            print(f"Error cargando modelo YOLO: {e}")
            return False
    
    def is_loaded(self) -> bool:
        """Verifica si el modelo está cargado."""
        return self.model is not None
    
    def detect(self, image: np.ndarray) -> List[Dict[str, Any]]:
        """
        Detecta cartas en una imagen.
        
        Args:
            image: Imagen como array numpy (RGB)
            
        Returns:
            Lista de detecciones con formato:
            [
                {
                    'suit': 'h',
                    'rank': 14,
                    'confidence': 0.95,
                    'bbox': (x, y, w, h),
                    'class_name': 'C 1'  # Formato original del dataset
                },
                ...
            ]
        """
        if not self.is_loaded():
            return []
        
        try:
            # Ejecutar detección
            results = self.model.predict(
                source=image,
                conf=self.confidence_threshold,
                verbose=False,
                device='cpu'
            )
            
            detections = []
            
            for result in results:
                boxes = result.boxes
                
                if boxes is None:
                    continue
                
                for i in range(len(boxes)):
                    # Obtener bounding box
                    box = boxes.xyxy[i].cpu().numpy()
                    x1, y1, x2, y2 = box
                    
                    # Convertir a formato (x, y, w, h)
                    x, y = int(x1), int(y1)
                    w, h = int(x2 - x1), int(y2 - y1)
                    
                    # Obtener clase y confianza
                    class_id = int(boxes.cls[i].cpu().numpy())
                    confidence = float(boxes.conf[i].cpu().numpy())
                    
                    # Obtener nombre de clase (ej: "C 1", "P 13")
                    class_name = self.model.names.get(class_id, str(class_id))
                    
                    # Parsear a formato interno
                    suit, rank = self._parse_class_name(class_name)
                    
                    if suit and rank:
                        detections.append({
                            'suit': suit,
                            'rank': rank,
                            'confidence': confidence,
                            'bbox': (x, y, w, h),
                            'class_name': class_name
                        })
            
            # Ordenar por confianza descendente
            detections.sort(key=lambda x: x['confidence'], reverse=True)
            
            return detections
            
        except Exception as e:
            print(f"Error en detección YOLO: {e}")
            return []
    
    def _parse_class_name(self, class_name: str) -> Tuple[Optional[str], Optional[int]]:
        """
        Parsea el nombre de clase del dataset al formato interno.
        
        Args:
            class_name: Nombre de clase (ej: "C 1", "P 13", "T 7")
            
        Returns:
            Tuple de (suit, rank) o (None, None) si no se puede parsear
        """
        try:
            parts = class_name.strip().split(' ')
            if len(parts) != 2:
                return None, None
            
            suit_code, value_str = parts
            
            suit = self.SUIT_MAP.get(suit_code)
            rank = self.VALUE_MAP.get(value_str)
            
            if suit and rank:
                return suit, rank
            
            return None, None
            
        except Exception:
            return None, None
    
    def detect_in_region(self, image: np.ndarray, region: Tuple[float, float, float, float]) -> List[Dict[str, Any]]:
        """
        Detecta cartas en una región específica de la imagen.
        
        Args:
            image: Imagen completa como array numpy
            region: Región como (x1, y1, x2, y2) en coordenadas relativas (0-1)
            
        Returns:
            Lista de detecciones
        """
        height, width = image.shape[:2]
        
        x1 = int(region[0] * width)
        y1 = int(region[1] * height)
        x2 = int(region[2] * width)
        y2 = int(region[3] * height)
        
        # Extraer región
        region_image = image[y1:y2, x1:x2]
        
        if region_image.size == 0:
            return []
        
        # Detectar en la región
        detections = self.detect(region_image)
        
        # Ajustar coordenadas al sistema de la imagen completa
        for det in detections:
            bx, by, bw, bh = det['bbox']
            det['bbox'] = (bx + x1, by + y1, bw, bh)
        
        return detections


# Singleton para reutilizar el detector
_detector_instance: Optional[YOLOCardDetector] = None


def get_detector(model_path: Optional[str] = None) -> YOLOCardDetector:
    """
    Obtiene una instancia singleton del detector.
    
    Args:
        model_path: Ruta al modelo (solo se usa en la primera llamada)
        
    Returns:
        Instancia del detector YOLO
    """
    global _detector_instance
    
    if _detector_instance is None:
        _detector_instance = YOLOCardDetector(model_path=model_path)
    
    return _detector_instance


if __name__ == "__main__":
    import sys
    from PIL import Image
    
    print("=== Test del Detector YOLO ===")
    
    detector = YOLOCardDetector()
    
    if not detector.is_loaded():
        print("\nNo hay modelo cargado. Para usar YOLO necesitas:")
        print("1. Entrenar un modelo con tu dataset")
        print("2. Colocar el archivo .pt en models/poker_cards.pt")
        sys.exit(1)
    
    # Buscar imagen de prueba
    test_dir = "../../../tests/images"
    if os.path.exists(test_dir):
        images = [f for f in os.listdir(test_dir) if f.endswith(('.jpg', '.png'))]
        if images:
            img_path = os.path.join(test_dir, images[0])
            print(f"\nProbando con: {img_path}")
            
            img = np.array(Image.open(img_path))
            detections = detector.detect(img)
            
            print(f"Detectadas {len(detections)} cartas:")
            for det in detections:
                print(f"  - {det['class_name']}: {det['suit']}{det['rank']} "
                      f"(conf: {det['confidence']:.2f})")
