"""
Core modules for card detection and number extraction.

Components:
- YOLOCardDetector: Card detection using YOLO v8 (CPU optimized)
- NumberExtractor: Number extraction using EasyOCR
"""

# YOLO-based detector
try:
    from .yolo_detector import YOLOCardDetector, get_detector as get_yolo_detector
    YOLO_AVAILABLE = True
except ImportError:
    YOLO_AVAILABLE = False
    YOLOCardDetector = None
    get_yolo_detector = None

# EasyOCR-based number extractor
try:
    from .number_extractor import NumberExtractor, get_extractor as get_number_extractor
    OCR_AVAILABLE = True
except ImportError:
    OCR_AVAILABLE = False
    NumberExtractor = None
    get_number_extractor = None


__all__ = [
    'YOLOCardDetector',
    'get_yolo_detector',
    'NumberExtractor',
    'get_number_extractor',
    'YOLO_AVAILABLE',
    'OCR_AVAILABLE',
]
