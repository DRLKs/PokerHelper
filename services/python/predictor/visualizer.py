"""
Visualizador de resultados para mostrar detecciones y predicciones.
"""
import matplotlib.pyplot as plt
import matplotlib.patches as patches
import numpy as np
from PIL import Image


class ResultVisualizer:
    """
    Clase para visualizar los resultados de detección y predicción de cartas.
    """
    
    def __init__(self):
        """
        Inicializa el visualizador.
        """
        # Configuración de colores para diferentes tipos de resultados
        self.colors = {
            'detected': 'red',
            'predicted': 'green',
            'error': 'orange'
        }
    
    def visualize_results(self, image, results, figsize=(15, 10)):
        """
        Visualiza los resultados sobre la imagen original.
        
        Args:
            image: Imagen original como numpy array
            results: Lista de resultados de detección/predicción
            figsize: Tamaño de la figura
        """
        fig, ax = plt.subplots(1, 1, figsize=figsize)
        ax.imshow(image)
        
        for result in results:
            x, y, w, h = result['bbox']
            
            # Determinar color según si hay predicción o error
            prediction = result['prediction']
            if 'error' in prediction:
                color = self.colors['error']
                status = "Error"
            else:
                color = self.colors['predicted']
                status = f"Clase {prediction['class_index']}"
            
            # Dibujar rectángulo
            rect = patches.Rectangle((x, y), w, h, linewidth=2, 
                                   edgecolor=color, facecolor='none')
            ax.add_patch(rect)
            
            # Preparar etiqueta
            if 'error' not in prediction:
                confidence = prediction.get('confidence', 0)
                label = f"{status}\nConf: {confidence:.2f}"
            else:
                label = f"{status}"
            
            # Añadir etiqueta
            ax.text(x, y-10, label, color=color, fontsize=8, 
                   bbox=dict(boxstyle="round,pad=0.3", facecolor='white', alpha=0.8))
        
        ax.set_title(f'Cartas detectadas: {len(results)}')
        ax.axis('off')
        plt.tight_layout()
        plt.show()
    
    def visualize_crops(self, crops, predictions=None, max_cols=5):
        """
        Visualiza los recortes de cartas individuales.
        
        Args:
            crops: Lista de diccionarios con recortes de cartas
            predictions: Lista opcional de predicciones correspondientes
            max_cols: Número máximo de columnas en la visualización
        """
        n_crops = len(crops)
        if n_crops == 0:
            print("No hay recortes para mostrar")
            return
        
        # Calcular dimensiones de la grilla
        n_cols = min(max_cols, n_crops)
        n_rows = (n_crops + n_cols - 1) // n_cols
        
        fig, axes = plt.subplots(n_rows, n_cols, figsize=(3*n_cols, 3*n_rows))
        
        # Asegurar que axes sea un array 2D
        if n_rows == 1:
            axes = axes.reshape(1, -1)
        if n_cols == 1:
            axes = axes.reshape(-1, 1)
        
        for i in range(n_crops):
            row = i // n_cols
            col = i % n_cols
            
            ax = axes[row, col]
            
            # Mostrar el recorte
            crop_image = crops[i]['image']
            ax.imshow(crop_image)
            
            # Preparar título
            title = f"Carta {crops[i]['id']}"
            if predictions and i < len(predictions):
                pred = predictions[i]
                if 'error' not in pred:
                    title += f"\nClase: {pred['class_index']}\nConf: {pred['confidence']:.2f}"
                else:
                    title += f"\n{pred['error']}"
            
            ax.set_title(title, fontsize=8)
            ax.axis('off')
        
        # Ocultar ejes vacíos
        for i in range(n_crops, n_rows * n_cols):
            row = i // n_cols
            col = i % n_cols
            axes[row, col].axis('off')
        
        plt.tight_layout()
        plt.show()
    
    def visualize_detection_steps(self, original_image, processed_steps):
        """
        Visualiza los pasos del proceso de detección.
        
        Args:
            original_image: Imagen original
            processed_steps: Dict con los pasos del procesamiento
        """
        # Número de pasos a mostrar
        n_steps = len(processed_steps) + 1  # +1 por la imagen original
        
        fig, axes = plt.subplots(1, n_steps, figsize=(5*n_steps, 5))
        
        if n_steps == 1:
            axes = [axes]
        
        # Mostrar imagen original
        axes[0].imshow(original_image)
        axes[0].set_title('Original')
        axes[0].axis('off')
        
        # Mostrar pasos de procesamiento
        for i, (step_name, step_image) in enumerate(processed_steps.items(), 1):
            if len(step_image.shape) == 2:  # Imagen en escala de grises
                axes[i].imshow(step_image, cmap='gray')
            else:
                axes[i].imshow(step_image)
            
            axes[i].set_title(step_name)
            axes[i].axis('off')
        
        plt.tight_layout()
        plt.show()
    
    def save_visualization(self, image, results, output_path, figsize=(15, 10)):
        """
        Guarda la visualización en un archivo.
        
        Args:
            image: Imagen original
            results: Resultados de detección/predicción
            output_path: Ruta donde guardar la imagen
            figsize: Tamaño de la figura
        """
        fig, ax = plt.subplots(1, 1, figsize=figsize)
        ax.imshow(image)
        
        for result in results:
            x, y, w, h = result['bbox']
            
            # Determinar color según si hay predicción o error
            prediction = result['prediction']
            if 'error' in prediction:
                color = self.colors['error']
                status = "Error"
            else:
                color = self.colors['predicted']
                status = f"Clase {prediction['class_index']}"
            
            # Dibujar rectángulo
            rect = patches.Rectangle((x, y), w, h, linewidth=2, 
                                   edgecolor=color, facecolor='none')
            ax.add_patch(rect)
            
            # Preparar etiqueta
            if 'error' not in prediction:
                confidence = prediction.get('confidence', 0)
                label = f"{status}\nConf: {confidence:.2f}"
            else:
                label = f"{status}"
            
            # Añadir etiqueta
            ax.text(x, y-10, label, color=color, fontsize=8, 
                   bbox=dict(boxstyle="round,pad=0.3", facecolor='white', alpha=0.8))
        
        ax.set_title(f'Cartas detectadas: {len(results)}')
        ax.axis('off')
        plt.tight_layout()
        plt.savefig(output_path, dpi=150, bbox_inches='tight')
        plt.close()
        
        print(f"Visualización guardada en: {output_path}")


def test_visualization():
    """
    Función de prueba para el visualizador.
    """
    import os
    from card_detector import CardDetector
    
    print("=== Test del Visualizador ===")
    
    # Buscar imagen de prueba
    if os.path.exists('test'):
        test_images = [os.path.join('test', f) for f in os.listdir('test') 
                      if f.lower().endswith(('.jpg', '.jpeg', '.png'))]
        
        if test_images:
            # Cargar imagen
            image_path = test_images[0]
            print(f"Usando imagen: {image_path}")
            
            pil_img = Image.open(image_path)
            img_array = np.array(pil_img)
            
            # Detectar cartas
            detector = CardDetector()
            regions = detector.detect_card_regions(img_array)
            crops = detector.extract_card_crops(img_array, regions)
            
            # Crear resultados falsos para prueba
            results = []
            for crop_data in crops:
                result = {
                    'crop_id': crop_data['id'],
                    'bbox': crop_data['bbox'],
                    'area': crop_data['area'],
                    'aspect_ratio': crop_data['aspect_ratio'],
                    'prediction': {
                        'class_index': np.random.randint(0, 10),
                        'confidence': np.random.random()
                    }
                }
                results.append(result)
            
            # Visualizar
            visualizer = ResultVisualizer()
            visualizer.visualize_results(img_array, results)
            
            # Mostrar recortes si hay algunos
            if crops[:5]:  # Solo primeros 5
                predictions = [r['prediction'] for r in results[:5]]
                visualizer.visualize_crops(crops[:5], predictions)
            
        else:
            print("No se encontraron imágenes en la carpeta test/")
    else:
        print("Carpeta test/ no existe")


if __name__ == "__main__":
    test_visualization()
