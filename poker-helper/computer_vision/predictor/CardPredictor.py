"""
Predictor de cartas usando YOLO v8 y EasyOCR.
Optimizado para CPU.
"""
import time
import numpy as np
from PIL import Image
from typing import Dict, Any, List, Optional, Tuple, Union
from dataclasses import dataclass, field

# Importar detectores
try:
    from .core.yolo_detector import YOLOCardDetector, get_detector as get_yolo_detector
    YOLO_AVAILABLE = True
except ImportError:
    YOLO_AVAILABLE = False
    YOLOCardDetector = None
    get_yolo_detector = None

try:
    from .core.number_extractor import NumberExtractor, get_extractor as get_number_extractor
    OCR_AVAILABLE = True
except ImportError:
    OCR_AVAILABLE = False
    NumberExtractor = None
    get_number_extractor = None


# =============================================================================
# Data Classes
# =============================================================================

@dataclass
class CardResult:
    """Carta detectada."""
    rank: str  # A, 2-10, J, Q, K
    suit: str  # hearts, diamonds, clubs, spades
    confidence: float
    bbox: Tuple[int, int, int, int]  # x, y, width, height
    
    def to_dict(self) -> Dict[str, Any]:
        return {
            'rank': self.rank,
            'suit': self.suit,
            'confidence': self.confidence,
            'bbox': list(self.bbox)
        }
    
    def __str__(self) -> str:
        symbols = {'hearts': '♥', 'diamonds': '♦', 'clubs': '♣', 'spades': '♠'}
        return f"{self.rank}{symbols.get(self.suit, '?')}"


@dataclass
class NumberResult:
    """Número extraído (pot, bet, stack)."""
    value: float
    text_raw: str
    confidence: float
    region: str
    
    def to_dict(self) -> Dict[str, Any]:
        return {
            'value': self.value,
            'text_raw': self.text_raw,
            'confidence': self.confidence,
            'region': self.region
        }


@dataclass
class AnalysisResult:
    """Resultado del análisis de imagen."""
    cards: List[CardResult] = field(default_factory=list)
    numbers: List[NumberResult] = field(default_factory=list)
    pot_size: Optional[float] = None
    player_stack: Optional[float] = None
    opponent_bets: List[float] = field(default_factory=list)
    image_shape: Optional[Tuple[int, int, int]] = None
    yolo_available: bool = False
    ocr_available: bool = False
    processing_time_ms: float = 0.0
    
    def to_dict(self) -> Dict[str, Any]:
        return {
            'cards': [c.to_dict() for c in self.cards],
            'numbers': [n.to_dict() for n in self.numbers],
            'pot_size': self.pot_size,
            'player_stack': self.player_stack,
            'opponent_bets': self.opponent_bets,
            'image_shape': list(self.image_shape) if self.image_shape else None,
            'yolo_available': self.yolo_available,
            'ocr_available': self.ocr_available,
            'processing_time_ms': self.processing_time_ms
        }


# =============================================================================
# CardPredictor
# =============================================================================

class CardPredictor:
    """
    Predictor de cartas de poker usando YOLO v8 y EasyOCR.
    
    Ejemplo:
        predictor = CardPredictor()
        result = predictor.analyze_image("screenshot.png")
        for card in result.cards:
            print(card)
    """
    
    def __init__(
        self,
        yolo_model_path: Optional[str] = None,
        confidence_threshold: float = 0.5,
        verbose: bool = False
    ):
        self.confidence_threshold = confidence_threshold
        self.verbose = verbose
        self.yolo_detector: Optional[YOLOCardDetector] = None
        self.number_extractor: Optional[NumberExtractor] = None
        
        # Inicializar YOLO
        if YOLO_AVAILABLE and get_yolo_detector:
            try:
                self.yolo_detector = get_yolo_detector()
                if yolo_model_path:
                    self.yolo_detector.load_model(yolo_model_path)
                self._log("YOLO inicializado")
            except Exception as e:
                self._log(f"Error YOLO: {e}")
        
        # Inicializar OCR
        if OCR_AVAILABLE and get_number_extractor:
            try:
                self.number_extractor = get_number_extractor()
                self._log("OCR inicializado")
            except Exception as e:
                self._log(f"Error OCR: {e}")
    
    def _log(self, msg: str):
        if self.verbose:
            print(f"[CardPredictor] {msg}")
    
    def is_ready(self) -> bool:
        """Verifica si el predictor puede analizar imágenes."""
        return self.yolo_detector is not None
    
    def load_yolo_model(self, model_path: str) -> bool:
        """Carga un modelo YOLO entrenado."""
        if not YOLO_AVAILABLE:
            return False
        try:
            if self.yolo_detector is None and get_yolo_detector:
                self.yolo_detector = get_yolo_detector()
            if self.yolo_detector:
                self.yolo_detector.load_model(model_path)
                return True
        except Exception as e:
            self._log(f"Error cargando modelo: {e}")
        return False
    
    def analyze_image(
        self,
        image: Union[str, np.ndarray, Image.Image],
        detect_cards: bool = True,
        extract_numbers: bool = True,
        number_regions: Optional[List[Dict]] = None
    ) -> AnalysisResult:
        """
        Analiza una imagen para detectar cartas y números.
        
        Args:
            image: Ruta, numpy array, o PIL Image
            detect_cards: Detectar cartas
            extract_numbers: Extraer números (pot, bets)
            number_regions: Regiones específicas para OCR
        """
        start = time.time()
        img = self._to_numpy(image)
        
        result = AnalysisResult(
            image_shape=img.shape,
            yolo_available=self.yolo_detector is not None,
            ocr_available=self.number_extractor is not None
        )
        
        if detect_cards:
            result.cards = self._detect_cards(img)
        
        if extract_numbers and self.number_extractor:
            result.numbers = self._extract_numbers(img, number_regions)
            for num in result.numbers:
                if num.region == 'pot':
                    result.pot_size = num.value
                elif num.region == 'player_stack':
                    result.player_stack = num.value
                elif 'opponent' in num.region:
                    result.opponent_bets.append(num.value)
        
        result.processing_time_ms = (time.time() - start) * 1000
        return result
    
    def _to_numpy(self, image: Union[str, np.ndarray, Image.Image]) -> np.ndarray:
        if isinstance(image, str):
            return np.array(Image.open(image))
        elif isinstance(image, Image.Image):
            return np.array(image)
        return image
    
    def _detect_cards(self, img: np.ndarray) -> List[CardResult]:
        if self.yolo_detector is None:
            return []
        
        try:
            detections = self.yolo_detector.detect_and_classify(
                img, conf_threshold=self.confidence_threshold
            )
            cards = []
            for det in detections:
                rank, suit = self._parse_label(det.get('class_name', ''))
                cards.append(CardResult(
                    rank=rank,
                    suit=suit,
                    confidence=det.get('confidence', 0.0),
                    bbox=tuple(det.get('bbox', [0, 0, 0, 0]))
                ))
            return cards
        except Exception as e:
            self._log(f"Error detección: {e}")
            return []
    
    def _extract_numbers(
        self, img: np.ndarray, regions: Optional[List[Dict]] = None
    ) -> List[NumberResult]:
        if self.number_extractor is None:
            return []
        
        try:
            numbers = []
            if regions:
                for r in regions:
                    roi = r.get('bbox')
                    name = r.get('name', 'unknown')
                    if roi:
                        for val, text, conf in self.number_extractor.extract_from_region(img, roi):
                            numbers.append(NumberResult(val, text, conf, name))
            else:
                for val, text, conf in self.number_extractor.extract_numbers(img):
                    numbers.append(NumberResult(val, text, conf, 'unknown'))
            return numbers
        except Exception as e:
            self._log(f"Error OCR: {e}")
            return []
    
    def _parse_label(self, label: str) -> Tuple[str, str]:
        """Parsea etiqueta YOLO: 'Ah' -> ('A', 'hearts')"""
        suits = {'h': 'hearts', 'd': 'diamonds', 'c': 'clubs', 's': 'spades'}
        if not label:
            return ('?', 'unknown')
        suit = suits.get(label[-1].lower(), 'unknown')
        rank = label[:-1].upper()
        valid = ['A', '2', '3', '4', '5', '6', '7', '8', '9', '10', 'J', 'Q', 'K']
        return (rank if rank in valid else '?', suit)
    
    def get_status(self) -> Dict[str, Any]:
        return {
            'ready': self.is_ready(),
            'yolo_available': YOLO_AVAILABLE,
            'yolo_loaded': self.yolo_detector is not None,
            'ocr_available': OCR_AVAILABLE,
            'ocr_loaded': self.number_extractor is not None,
            'confidence_threshold': self.confidence_threshold
        }


# =============================================================================
# Singleton
# =============================================================================

_instance: Optional[CardPredictor] = None

def get_predictor(**kwargs) -> CardPredictor:
    """Obtiene la instancia singleton del predictor."""
    global _instance
    if _instance is None:
        _instance = CardPredictor(**kwargs)
    return _instance


def analyze_image_file(path: str, model_path: Optional[str] = None) -> AnalysisResult:
    """Analiza una imagen desde archivo."""
    p = get_predictor()
    if model_path:
        p.load_yolo_model(model_path)
    return p.analyze_image(path)


def analyze_screenshot(img: np.ndarray, regions: Optional[List[Dict]] = None) -> AnalysisResult:
    """Analiza una captura de pantalla."""
    return get_predictor().analyze_image(img, number_regions=regions)
