"""
Extractor de texto numérico usando EasyOCR.
Optimizado para leer valores de pot, bets y blinds en pantallas de poker.
"""
import numpy as np
from typing import Optional, Tuple, List
import re


class NumberExtractor:
    """
    Extractor de números de imágenes usando EasyOCR.
    Optimizado para CPU y para el contexto de poker (números, $, K, M).
    """
    
    def __init__(self, languages: List[str] = ['en'], gpu: bool = False):
        """
        Inicializa el extractor de números.
        
        Args:
            languages: Lista de idiomas para OCR
            gpu: Si usar GPU (False por defecto para compatibilidad)
        """
        self.reader = None
        self.gpu = gpu
        self.languages = languages
        self._initialize_reader()
    
    def _initialize_reader(self) -> bool:
        """Inicializa el lector EasyOCR."""
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
        """Verifica si el lector está inicializado."""
        return self.reader is not None
    
    def extract_number(self, image: np.ndarray, preprocess: bool = True) -> Tuple[float, float]:
        """
        Extrae un número de una imagen.
        
        Args:
            image: Imagen como array numpy (RGB o grayscale)
            preprocess: Si aplicar preprocesamiento
            
        Returns:
            Tuple de (valor_extraído, confianza)
            Devuelve (0.0, 0.0) si no se encuentra número
        """
        if not self.is_loaded():
            return 0.0, 0.0
        
        try:
            # Preprocesar imagen si se solicita
            if preprocess:
                image = self._preprocess_for_ocr(image)
            
            # Ejecutar OCR
            results = self.reader.readtext(
                image,
                allowlist='0123456789.$,kKmMbB ',
                paragraph=False
            )
            
            if not results:
                return 0.0, 0.0
            
            # Procesar resultados
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
        Extrae todos los números encontrados en una imagen.
        
        Args:
            image: Imagen como array numpy
            
        Returns:
            Lista de (valor, confianza, bbox)
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
        Preprocesa la imagen para mejorar el OCR.
        
        Args:
            image: Imagen original
            
        Returns:
            Imagen preprocesada
        """
        import cv2
        
        # Convertir a escala de grises si es RGB
        if len(image.shape) == 3:
            gray = cv2.cvtColor(image, cv2.COLOR_RGB2GRAY)
        else:
            gray = image.copy()
        
        # Redimensionar si es muy pequeña
        height, width = gray.shape[:2]
        if height < 50:
            scale = 50 / height
            new_width = int(width * scale)
            gray = cv2.resize(gray, (new_width, 50), interpolation=cv2.INTER_CUBIC)
        
        # Aplicar umbral adaptativo para binarizar
        binary = cv2.adaptiveThreshold(
            gray, 255,
            cv2.ADAPTIVE_THRESH_GAUSSIAN_C,
            cv2.THRESH_BINARY,
            11, 2
        )
        
        # Invertir si el fondo es oscuro
        if np.mean(binary) < 127:
            binary = cv2.bitwise_not(binary)
        
        return binary
    
    def _parse_poker_number(self, text: str) -> float:
        """
        Parsea un string de texto a valor numérico de poker.
        
        Soporta formatos:
        - "1,234" -> 1234.0
        - "$1.5K" -> 1500.0
        - "2.5M" -> 2500000.0
        - "500" -> 500.0
        - "1.5B" -> 1500000000.0
        
        Args:
            text: Texto a parsear
            
        Returns:
            Valor numérico o 0.0 si no se puede parsear
        """
        if not text:
            return 0.0
        
        # Limpiar texto
        text = text.strip().upper()
        text = text.replace('$', '').replace(' ', '').replace(',', '')
        
        # Detectar multiplicadores
        multiplier = 1.0
        
        if text.endswith('K'):
            multiplier = 1_000
            text = text[:-1]
        elif text.endswith('M'):
            multiplier = 1_000_000
            text = text[:-1]
        elif text.endswith('B'):
            multiplier = 1_000_000_000
            text = text[:-1]
        
        # Intentar parsear el número
        try:
            # Manejar números con punto decimal
            value = float(text)
            return value * multiplier
        except ValueError:
            pass
        
        # Intentar extraer solo dígitos
        digits = re.sub(r'[^\d.]', '', text)
        if digits:
            try:
                return float(digits) * multiplier
            except ValueError:
                pass
        
        return 0.0


# Singleton para reutilizar el extractor
_extractor_instance: Optional[NumberExtractor] = None


def get_extractor() -> NumberExtractor:
    """
    Obtiene una instancia singleton del extractor.
    
    Returns:
        Instancia del extractor de números
    """
    global _extractor_instance
    
    if _extractor_instance is None:
        _extractor_instance = NumberExtractor(gpu=False)
    
    return _extractor_instance


def extract_pot_value(image: np.ndarray) -> Tuple[float, float]:
    """
    Función de conveniencia para extraer el valor del pot.
    
    Args:
        image: Imagen de la región del pot
        
    Returns:
        Tuple de (valor, confianza)
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
    
    # Tests con strings
    test_cases = [
        "1,234",
        "$1.5K",
        "2.5M",
        "500",
        "$100",
        "10.5k",
        "1B",
    ]
    
    print("\nTest de parsing de números:")
    for text in test_cases:
        value = extractor._parse_poker_number(text)
        print(f"  '{text}' -> {value}")
