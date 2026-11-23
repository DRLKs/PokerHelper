"""
Card detector using YOLO v8 (Ultralytics).
Optimized for CPU.
"""
import os
import numpy as np
from typing import List, Dict, Any, Optional, Tuple
from pathlib import Path


DATASET_CLUBS = 'T'
DATASET_SPADES = 'P'
DATASET_HEARTS = 'C'
DATASET_DIAMONDS = 'D'
DATASET_ACE = '1'
ACE_HIGH = 14


class YOLOCardDetector:
    """
    Poker card detector using YOLO v8.
    Designed to run without a GPU.
    """
    
    SUIT_MAP = {
        DATASET_CLUBS: 'c',
        DATASET_SPADES: 's',
        DATASET_HEARTS: 'h',
        DATASET_DIAMONDS: 'd',
    }

    VALUE_MAP = {
        DATASET_ACE: ACE_HIGH,
        '2': 2, '3': 3, '4': 4, '5': 5, '6': 6,
        '7': 7, '8': 8, '9': 9, '10': 10,
        '11': 11, '12': 12, '13': 13,
    }
    
    def __init__(self, model_path: Optional[str] = None, confidence_threshold: float = 0.5):
        """
        Initializes the YOLO detector.
        
        Args:
            model_path: Path to the trained .pt model. If None, the default paths are searched.
            confidence_threshold: Minimum confidence for a detection (0-1)
        """
        self.confidence_threshold = confidence_threshold
        self.model = None
        self.model_path = model_path
        
        self._load_model()
    
    def _find_model(self) -> Optional[str]:
        """Looks for the model in the usual locations."""
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
        """Loads the YOLO model."""
        try:
            from ultralytics import YOLO
            
            model_path = self._find_model()
            
            if model_path:
                print(f"Cargando modelo YOLO desde: {model_path}")
                self.model = YOLO(model_path)
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
        """Returns whether the model is loaded."""
        return self.model is not None
    
    def detect(self, image: np.ndarray) -> List[Dict[str, Any]]:
        """
        Detects cards in an image.
        
        Args:
            image: Image as a numpy array (RGB)
            
        Returns:
            List of detections with this format:
            [
                {
                    'suit': 'h',
                    'rank': 14,
                    'confidence': 0.95,
                    'bbox': (x, y, w, h),
                    'class_name': 'C 1'
                },
                ...
            ]
        """
        if not self.is_loaded():
            return []
        
        try:
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
                    box = boxes.xyxy[i].cpu().numpy()
                    left, top, right, bottom = box
                    x, y = int(left), int(top)
                    w, h = int(right - left), int(bottom - top)
                    
                    class_id = int(boxes.cls[i].cpu().numpy())
                    confidence = float(boxes.conf[i].cpu().numpy())
                    
                    class_name = self.model.names.get(class_id, str(class_id))
                    
                    suit, rank = self._parse_class_name(class_name)
                    
                    if suit and rank:
                        detections.append({
                            'suit': suit,
                            'rank': rank,
                            'confidence': confidence,
                            'bbox': (x, y, w, h),
                            'class_name': class_name
                        })
            
            detections.sort(key=lambda x: x['confidence'], reverse=True)
            
            return detections
            
        except Exception as e:
            print(f"Error en detección YOLO: {e}")
            return []
    
    def _parse_class_name(self, class_name: str) -> Tuple[Optional[str], Optional[int]]:
        """
        Parses a dataset class name into the internal format.
        
        Args:
            class_name: Dataset class name (e.g. "C 1", "P 13", "T 7")
            
        Returns:
            Tuple of (suit, rank), or (None, None) if it cannot be parsed
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
        Detects cards in a specific region of the image.
        
        Args:
            image: Full image as a numpy array
            region: Region as (x1, y1, x2, y2) in relative coordinates (0-1)
            
        Returns:
            List of detections
        """
        height, width = image.shape[:2]
        
        x1 = int(region[0] * width)
        y1 = int(region[1] * height)
        x2 = int(region[2] * width)
        y2 = int(region[3] * height)
        
        region_image = image[y1:y2, x1:x2]
        
        if region_image.size == 0:
            return []
        
        detections = self.detect(region_image)
        
        for detection in detections:
            x_in_region, y_in_region, box_width, box_height = detection['bbox']
            detection['bbox'] = (x_in_region + x1, y_in_region + y1, box_width, box_height)
        
        return detections


_detector_instance: Optional[YOLOCardDetector] = None


def get_detector(model_path: Optional[str] = None) -> YOLOCardDetector:
    """
    Returns the singleton detector instance.
    
    Args:
        model_path: Path to the model (only used on the first call)
        
    Returns:
        The YOLO detector instance
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
