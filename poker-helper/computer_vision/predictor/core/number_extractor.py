"""
Numeric text extractor using EasyOCR.
Tuned to read pot, bet and blind amounts on poker screens.
"""
import numpy as np
from typing import Optional, Tuple, List
import re

MIN_OCR_HEIGHT = 50
AMOUNT_SUFFIX_MULTIPLIERS = {"K": 1_000, "M": 1_000_000, "B": 1_000_000_000}


class NumberExtractor:
    """
    Extracts numbers from images using EasyOCR.
    Optimized for CPU and for poker amounts (digits, $, K, M).
    """
    
    def __init__(self, languages: List[str] = ['en'], gpu: bool = False):
        """
        Initializes the number extractor.
        
        Args:
            languages: Languages used by the OCR
            gpu: Whether to use the GPU (False by default for compatibility)
        """
        self.reader = None
        self.gpu = gpu
        self.languages = languages
        self._initialize_reader()
    
    def _initialize_reader(self) -> bool:
        """Initializes the EasyOCR reader."""
        try:
            import easyocr
            
            print("Inicializando EasyOCR (esto puede tardar la primera vez)...")
            self.reader = easyocr.Reader(
                self.languages,
                gpu=self.gpu,
                verbose=False
            )
            print("EasyOCR inicializado correctamente")
            return True
            
        except ImportError:
            print("Error: easyocr no está instalado. Ejecuta: pip install easyocr")
            return False
        except Exception as e:
            print(f"Error inicializando EasyOCR: {e}")
            return False
    
    def is_loaded(self) -> bool:
        """Returns whether the reader is initialized."""
        return self.reader is not None
    
    def extract_number(self, image: np.ndarray, preprocess: bool = True) -> Tuple[float, float]:
        """
        Extracts a number from an image.
        
        Args:
            image: Image as a numpy array (RGB or grayscale)
            preprocess: Whether to preprocess the image first
            
        Returns:
            Tuple of (extracted_value, confidence)
            Returns (0.0, 0.0) when no number is found
        """
        if not self.is_loaded():
            return 0.0, 0.0
        
        try:
            if preprocess:
                image = self._preprocess_for_ocr(image)
            
            results = self.reader.readtext(
                image,
                allowlist='0123456789.$,kKmMbB ',
                paragraph=False
            )
            
            if not results:
                return 0.0, 0.0
            
            for bbox, text, confidence in results:
                parsed_value = self._parse_poker_number(text)
                if parsed_value > 0:
                    return parsed_value, confidence
            
            return 0.0, 0.0
            
        except Exception as e:
            print(f"Error en OCR: {e}")
            return 0.0, 0.0
    
    def extract_all_numbers(self, image: np.ndarray) -> List[Tuple[float, float, Tuple]]:
        """
        Extracts every number found in an image.
        
        Args:
            image: Image as a numpy array
            
        Returns:
            List of (value, confidence, bbox)
        """
        if not self.is_loaded():
            return []
        
        try:
            image = self._preprocess_for_ocr(image)
            
            results = self.reader.readtext(
                image,
                allowlist='0123456789.$,kKmMbB ',
                paragraph=False
            )
            
            numbers = []
            for bbox, text, confidence in results:
                parsed_value = self._parse_poker_number(text)
                if parsed_value > 0:
                    numbers.append((parsed_value, confidence, bbox))
            
            return numbers
            
        except Exception as e:
            print(f"Error en OCR: {e}")
            return []
    
    def _preprocess_for_ocr(self, image: np.ndarray) -> np.ndarray:
        """
        Preprocesses the image to improve OCR accuracy.
        
        Args:
            image: Original image
            
        Returns:
            Preprocessed image
        """
        import cv2
        
        if len(image.shape) == 3:
            gray = cv2.cvtColor(image, cv2.COLOR_RGB2GRAY)
        else:
            gray = image.copy()
        
        height, width = gray.shape[:2]
        if height < MIN_OCR_HEIGHT:
            scale = MIN_OCR_HEIGHT / height
            new_width = int(width * scale)
            gray = cv2.resize(gray, (new_width, MIN_OCR_HEIGHT), interpolation=cv2.INTER_CUBIC)
        
        binary = cv2.adaptiveThreshold(
            gray, 255,
            cv2.ADAPTIVE_THRESH_GAUSSIAN_C,
            cv2.THRESH_BINARY,
            11, 2
        )
        
        background_is_dark = np.mean(binary) < 127
        if background_is_dark:
            binary = cv2.bitwise_not(binary)
        
        return binary
    
    def _parse_poker_number(self, text: str) -> float:
        """
        Parses text into a poker amount.
        
        Supported formats:
        - "1,234" -> 1234.0
        - "$1.5K" -> 1500.0
        - "2.5M" -> 2500000.0
        - "500" -> 500.0
        - "1.5B" -> 1500000000.0
        
        Args:
            text: Text to parse
            
        Returns:
            Numeric value, or 0.0 if the text cannot be parsed
        """
        if not text:
            return 0.0
        
        text = text.strip().upper()
        text = text.replace('$', '').replace(' ', '').replace(',', '')
        
        multiplier = 1.0
        suffix = text[-1:]
        if suffix in AMOUNT_SUFFIX_MULTIPLIERS:
            multiplier = AMOUNT_SUFFIX_MULTIPLIERS[suffix]
            text = text[:-1]
        
        try:
            value = float(text)
            return value * multiplier
        except ValueError:
            pass
        
        digits = re.sub(r'[^\d.]', '', text)
        if digits:
            try:
                return float(digits) * multiplier
            except ValueError:
                pass
        
        return 0.0


_extractor_instance: Optional[NumberExtractor] = None


def get_extractor() -> NumberExtractor:
    """
    Returns the singleton extractor instance.
    
    Returns:
        The number extractor instance
    """
    global _extractor_instance
    
    if _extractor_instance is None:
        _extractor_instance = NumberExtractor(gpu=False)
    
    return _extractor_instance


def extract_pot_value(image: np.ndarray) -> Tuple[float, float]:
    """
    Convenience function that extracts the pot value.
    
    Args:
        image: Image of the pot region
        
    Returns:
        Tuple of (value, confidence)
    """
    extractor = get_extractor()
    return extractor.extract_number(image)


if __name__ == "__main__":
    import sys
    from PIL import Image
    import os
    
    print("=== Test del Extractor de Números ===")
    
    extractor = NumberExtractor(gpu=False)
    
    if not extractor.is_loaded():
        print("No se pudo inicializar EasyOCR")
        sys.exit(1)
    
    sample_amounts = [
        "1,234",
        "$1.5K",
        "2.5M",
        "500",
        "$100",
        "10.5k",
        "1B",
    ]
    
    print("\nTest de parsing de números:")
    for text in sample_amounts:
        value = extractor._parse_poker_number(text)
        print(f"  '{text}' -> {value}")
