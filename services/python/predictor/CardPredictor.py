"""
Predictor principal que combina detección, preprocesamiento y predicción.
"""
import numpy as np
from PIL import Image
import os

from card_detector import CardDetector
from image_preprocessor import ImagePreprocessor
from model_loader import ModelLoader
from visualizer import ResultVisualizer


class CardPredictor:
    """
    Predictor principal de cartas que integra todos los componentes.
    """
    
    def __init__(self, model_path=None, img_height=1200, img_width=1920):
        """
        Inicializa el predictor de cartas.
        
        Args:
            model_path: Ruta al modelo entrenado (.h5). Si es None, busca el último modelo
            img_height: Altura de imagen para el modelo
            img_width: Ancho de imagen para el modelo
        """
        # Inicializar componentes
        self.detector = CardDetector()
        self.preprocessor = ImagePreprocessor(img_height, img_width)
        self.model_loader = ModelLoader()
        self.visualizer = ResultVisualizer()
        
        # Cargar modelo
        self.load_model(model_path)
    
    def load_model(self, model_path=None):
        """
        Carga el modelo predictivo.
        
        Args:
            model_path: Ruta específica del modelo (opcional)
        """
        success = self.model_loader.load_model(model_path)
        if not success:
            print("Advertencia: No se pudo cargar el modelo.")
            print("El predictor funcionará solo para detección sin predicción.")
    
    def predict_single_card(self, card_crop):
        """
        Predice el valor y palo de una carta individual.
        
        Args:
            card_crop: Imagen recortada de la carta
            
        Returns:
            dict: Diccionario con la predicción y confianza
        """
        if not self.model_loader.is_loaded():
            return {'error': 'Modelo no cargado'}
        
        try:
            # Preprocesar el recorte
            processed_crop = self.preprocessor.preprocess_image(card_crop)
            
            # Añadir dimensión de batch
            batch_crop = np.expand_dims(processed_crop, axis=0)
            
            # Hacer predicción
            prediction = self.model_loader.predict(batch_crop)
            
            # Obtener la clase con mayor probabilidad
            predicted_class_idx = np.argmax(prediction[0])
            confidence = prediction[0][predicted_class_idx]
            
            return {
                'class_index': int(predicted_class_idx),
                'confidence': float(confidence),
                'raw_prediction': prediction[0].tolist()
            }
            
        except Exception as e:
            return {'error': f'Error en predicción: {str(e)}'}
    
    def analyze_image(self, image, show_results=True, save_results=False):
        """
        Función principal que analiza una imagen completa buscando cartas.
        
        Args:
            image: Imagen a analizar (array numpy, PIL Image o ruta)
            show_results: Si mostrar los resultados visualmente
            save_results: Si guardar los resultados
            
        Returns:
            dict: Resultados del análisis
        """
        # Convertir imagen a numpy array si es necesario
        if isinstance(image, str):
            pil_img = Image.open(image)
            img_array = np.array(pil_img)
        elif isinstance(image, Image.Image):
            img_array = np.array(image)
        else:
            img_array = image
        
        print("Iniciando análisis de imagen...")
        print(f"Dimensiones de la imagen: {img_array.shape}")
        
        # 1. Detectar regiones de cartas
        print("Detectando regiones de cartas...")
        card_regions = self.detector.detect_card_regions(img_array)
        print(f"Se detectaron {len(card_regions)} posibles regiones de cartas")
        
        # 2. Extraer recortes
        print("Extrayendo recortes de cartas...")
        card_crops = self.detector.extract_card_crops(img_array, card_regions)
        print(f"Se extrajeron {len(card_crops)} recortes válidos")
        
        # 3. Predecir cada carta (solo si el modelo está cargado)
        results = []
        
        if self.model_loader.is_loaded():
            print("Analizando cada carta con el modelo...")
            
            for i, crop_data in enumerate(card_crops):
                print(f"Analizando carta {i+1}/{len(card_crops)}")
                
                prediction = self.predict_single_card(crop_data['image'])
                
                result = {
                    'crop_id': crop_data['id'],
                    'bbox': crop_data['bbox'],
                    'area': crop_data['area'],
                    'aspect_ratio': crop_data['aspect_ratio'],
                    'prediction': prediction
                }
                results.append(result)
        else:
            print("Modelo no disponible. Solo se mostrarán las detecciones.")
            
            for crop_data in card_crops:
                result = {
                    'crop_id': crop_data['id'],
                    'bbox': crop_data['bbox'],
                    'area': crop_data['area'],
                    'aspect_ratio': crop_data['aspect_ratio'],
                    'prediction': {'error': 'Modelo no cargado'}
                }
                results.append(result)
        
        # 4. Mostrar resultados si se solicita
        if show_results and results:
            self.visualizer.visualize_results(img_array, results)
        
        # 5. Guardar resultados si se solicita
        if save_results:
            self.save_results(results)
        
        analysis_result = {
            'total_cards_detected': len(results),
            'results': results,
            'image_shape': img_array.shape,
            'model_loaded': self.model_loader.is_loaded()
        }
        
        return analysis_result
    
    def save_results(self, results, output_file='prediction_results.txt'):
        """
        Guarda los resultados en un archivo.
        """
        with open(output_file, 'w') as f:
            f.write(f"Resultados de predicción de cartas\n")
            f.write(f"=====================================\n\n")
            
            for i, result in enumerate(results):
                f.write(f"Carta {i+1}:\n")
                f.write(f"  ID: {result['crop_id']}\n")
                f.write(f"  Bbox (x,y,w,h): {result['bbox']}\n")
                f.write(f"  Área: {result['area']}\n")
                f.write(f"  Ratio aspecto: {result['aspect_ratio']:.2f}\n")
                f.write(f"  Predicción: {result['prediction']}\n")
                f.write(f"\n")
        
        print(f"Resultados guardados en: {output_file}")


# Funciones de utilidad
def analyze_image_file(image_path, model_path=None, show_results=True):
    """
    Función de conveniencia para analizar una imagen desde archivo.
    """
    predictor = CardPredictor(model_path=model_path)
    return predictor.analyze_image(image_path, show_results=show_results)


def analyze_numpy_image(image_array, model_path=None, show_results=True):
    """
    Función de conveniencia para analizar una imagen como array numpy.
    """
    predictor = CardPredictor(model_path=model_path)
    return predictor.analyze_image(image_array, show_results=show_results)


if __name__ == "__main__":
    print("=== Test del Predictor Principal ===")
    
    # Crear instancia del predictor
    predictor = CardPredictor()
    
    # Buscar imágenes de prueba
    if os.path.exists('test'):
        test_images = [os.path.join('test', f) for f in os.listdir('test') 
                      if f.lower().endswith(('.jpg', '.jpeg', '.png'))][:3]
        
        if test_images:
            print(f"\nAnalizando {len(test_images)} imágenes de prueba...")
            
            for image_path in test_images:
                print(f"\n--- Analizando: {os.path.basename(image_path)} ---")
                try:
                    results = predictor.analyze_image(image_path, show_results=False, save_results=True)
                    print(f"✓ Cartas detectadas: {results['total_cards_detected']}")
                    print(f"✓ Modelo cargado: {results['model_loaded']}")
                    
                    # Mostrar detalles de cada carta
                    for i, result in enumerate(results['results'][:5]):  # Solo primeras 5
                        bbox = result['bbox']
                        pred = result['prediction']
                        print(f"  Carta {i+1}: Bbox{bbox}, Área: {result['area']}")
                        
                        if 'error' not in pred:
                            print(f"    -> Predicción: Clase {pred['class_index']} (conf: {pred['confidence']:.3f})")
                        else:
                            print(f"    -> {pred['error']}")
                            
                except Exception as e:
                    print(f"✗ Error procesando {image_path}: {e}")
        else:
            print("No se encontraron imágenes de prueba en la carpeta 'test'")
    else:
        print("Carpeta 'test' no existe")
        print("Crea la carpeta 'test' y coloca algunas imágenes para probar")
