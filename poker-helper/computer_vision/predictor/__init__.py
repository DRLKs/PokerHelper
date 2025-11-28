"""
Predictor de cartas de poker.

Uso:
    from predictor import get_predictor
    
    predictor = get_predictor()
    result = predictor.analyze_image("screenshot.png")
    
    for card in result.cards:
        print(card)
"""

from .CardPredictor import (
    CardPredictor,
    get_predictor,
    analyze_image_file,
    analyze_screenshot,
    CardResult,
    NumberResult,
    AnalysisResult,
    YOLO_AVAILABLE,
    OCR_AVAILABLE,
)

__all__ = [
    'CardPredictor',
    'get_predictor',
    'analyze_image_file',
    'analyze_screenshot',
    'CardResult',
    'NumberResult',
    'AnalysisResult',
    'YOLO_AVAILABLE',
    'OCR_AVAILABLE',
]
