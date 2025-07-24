



"""
ARCHIVO DE COMPATIBILIDAD
========================
Este archivo mantiene compatibilidad con el código anterior.
Para el nuevo sistema modular, usa los archivos individuales:

- card_detector.py: Detección de regiones de cartas
- image_preprocessor.py: Preprocesamiento de imágenes  
- model_loader.py: Carga de modelos
- visualizer.py: Visualización de resultados
- card_predictor.py: Predictor principal integrado
- main_test.py: Script de pruebas completas
"""

# Importar la funcionalidad principal desde el nuevo sistema modular
from card_predictor import CardPredictor, analyze_image_file, analyze_numpy_image

# Mantener compatibilidad con código existente
def analyze_image(image_path, model_path=None):
    """Función de compatibilidad."""
    return analyze_image_file(image_path, model_path)

# Crear una instancia por defecto
_default_predictor = None

def get_default_predictor():
    """Obtiene el predictor por defecto."""
    global _default_predictor
    if _default_predictor is None:
        _default_predictor = CardPredictor()
    return _default_predictor

# Para mantener la API anterior
class CardPredictor(CardPredictor):
    """Clase de compatibilidad que extiende la nueva implementación.""" 
    pass

if __name__ == "__main__":
    print("=== Predictor de Cartas (Archivo de Compatibilidad) ===")
    print("Para pruebas completas, ejecuta: python main_test.py")
    print("Para usar el sistema modular, importa desde card_predictor.py")
    
    # Ejecutar una prueba básica
    import os
    if os.path.exists('test'):
        test_images = [os.path.join('test', f) for f in os.listdir('test') 
                      if f.lower().endswith(('.jpg', '.jpeg', '.png'))][:1]
        
        if test_images:
            print(f"\nProbando con: {test_images[0]}")
            predictor = CardPredictor()
            results = predictor.analyze_image(test_images[0], show_results=False)
            print(f"Cartas detectadas: {results['total_cards_detected']}")
        else:
            print("No hay imágenes de prueba disponibles")
    else:
        print("Carpeta test/ no encontrada")