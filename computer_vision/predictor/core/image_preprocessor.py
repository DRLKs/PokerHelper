"""
Preprocesador de imágenes para el modelo de predicción de cartas.
"""
import numpy as np
from PIL import Image
from tensorflow.keras.preprocessing.image import load_img, img_to_array


class ImagePreprocessor:
    """
    Clase para preprocesar imágenes antes de pasarlas al modelo.
    """
    
    def __init__(self, img_height=1200, img_width=1920):
        """
        Inicializa el preprocesador.
        
        Args:
            img_height: Altura objetivo de las imágenes
            img_width: Ancho objetivo de las imágenes
        """
        self.img_height = img_height
        self.img_width = img_width
    
    def preprocess_image(self, image):
        """
        Preprocesa la imagen para el modelo.
        
        Args:
            image: Imagen como array numpy, PIL Image o ruta de archivo
            
        Returns:
            numpy array: Imagen preprocesada lista para el modelo
        """
        # Convertir diferentes tipos de entrada a PIL Image
        if isinstance(image, str):  # Si es una ruta de archivo
            pil_image = load_img(image, target_size=(self.img_height, self.img_width))
        elif isinstance(image, np.ndarray):  # Si es array numpy
            pil_image = Image.fromarray(image)
            pil_image = pil_image.resize((self.img_width, self.img_height))
        else:  # Si ya es PIL Image
            pil_image = image.resize((self.img_width, self.img_height))
        
        # Convertir a array y normalizar
        img_array = img_to_array(pil_image) / 255.0
        
        return img_array
    
    def preprocess_batch(self, images):
        """
        Preprocesa un lote de imágenes.
        
        Args:
            images: Lista de imágenes
            
        Returns:
            numpy array: Lote de imágenes preprocesadas
        """
        processed_images = []
        
        for image in images:
            processed_img = self.preprocess_image(image)
            processed_images.append(processed_img)
        
        return np.array(processed_images)
    
    def resize_maintaining_aspect_ratio(self, image, target_size):
        """
        Redimensiona una imagen manteniendo la relación de aspecto.
        
        Args:
            image: PIL Image o numpy array
            target_size: Tuple (width, height) del tamaño objetivo
            
        Returns:
            PIL Image: Imagen redimensionada
        """
        if isinstance(image, np.ndarray):
            pil_image = Image.fromarray(image)
        else:
            pil_image = image
        
        # Calcular el ratio para mantener aspecto
        original_width, original_height = pil_image.size
        target_width, target_height = target_size
        
        # Calcular ratios
        width_ratio = target_width / original_width
        height_ratio = target_height / original_height
        
        # Usar el ratio menor para mantener la imagen completa
        resize_ratio = min(width_ratio, height_ratio)
        
        # Calcular nuevas dimensiones
        new_width = int(original_width * resize_ratio)
        new_height = int(original_height * resize_ratio)
        
        # Redimensionar
        resized_image = pil_image.resize((new_width, new_height), Image.Resampling.LANCZOS)
        
        # Crear imagen con padding si es necesario
        final_image = Image.new('RGB', target_size, (0, 0, 0))
        
        # Calcular posición para centrar la imagen
        paste_x = (target_width - new_width) // 2
        paste_y = (target_height - new_height) // 2
        
        final_image.paste(resized_image, (paste_x, paste_y))
        
        return final_image


def test_preprocessing(image_path=None, image_array=None):
    """
    Función de prueba para el preprocesamiento.
    
    Args:
        image_path: Ruta a la imagen (opcional)
        image_array: Array numpy de la imagen (opcional)
    """
    import matplotlib.pyplot as plt
    
    # Crear preprocesador
    preprocessor = ImagePreprocessor()
    
    # Cargar imagen
    if image_path:
        original_img = Image.open(image_path)
    elif image_array is not None:
        original_img = Image.fromarray(image_array)
    else:
        print("Error: Proporciona image_path o image_array")
        return
    
    print(f"Imagen original: {original_img.size}")
    
    # Preprocesar
    processed = preprocessor.preprocess_image(original_img)
    print(f"Imagen procesada: {processed.shape}")
    
    # Mostrar resultados
    fig, axes = plt.subplots(1, 2, figsize=(15, 5))
    
    # Imagen original
    axes[0].imshow(original_img)
    axes[0].set_title(f'Original ({original_img.size})')
    axes[0].axis('off')
    
    # Imagen procesada
    axes[1].imshow(processed)
    axes[1].set_title(f'Procesada ({processed.shape})')
    axes[1].axis('off')
    
    plt.tight_layout()
    plt.show()
    
    return processed


if __name__ == "__main__":
    import os
    
    print("=== Test del Preprocesador de Imágenes ===")
    
    # Buscar imagen de prueba
    if os.path.exists('test'):
        test_images = [os.path.join('test', f) for f in os.listdir('test') 
                      if f.lower().endswith(('.jpg', '.jpeg', '.png'))]
        
        if test_images:
            print(f"Probando con: {test_images[0]}")
            test_preprocessing(test_images[0])
        else:
            print("No se encontraron imágenes en la carpeta test/")
    else:
        print("Carpeta test/ no existe")
