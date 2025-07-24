"""
Script interactivo para visualizar el preprocesamiento de imágenes.
Permite elegir una imagen y ver el resultado del preprocesamiento.
"""
import os
import sys
import matplotlib.pyplot as plt
from PIL import Image
import numpy as np
from tkinter import filedialog, messagebox
import tkinter as tk

# Añadir el directorio padre al path para importar los módulos
sys.path.append(os.path.dirname(os.path.dirname(os.path.abspath(__file__))))

from image_preprocessor import ImagePreprocessor

class ImagePreprocessorVisualizer:
    """
    Visualizador interactivo para el preprocesamiento de imágenes.
    """
    
    def __init__(self):
        """Inicializa el visualizador."""
        self.preprocessor = ImagePreprocessor()
        self.last_image_path = None
        self.last_processed = None
    
    def select_image_gui(self):
        """
        Abre un diálogo para seleccionar una imagen usando GUI.
        
        Returns:
            str: Ruta de la imagen seleccionada o None si se cancela
        """
        # Crear ventana raíz temporal (se oculta)
        root = tk.Tk()
        root.withdraw()
        
        # Tipos de archivo permitidos
        filetypes = [
            ("Imágenes", "*.jpg *.jpeg *.png *.bmp *.tiff *.gif"),
            ("JPEG", "*.jpg *.jpeg"),
            ("PNG", "*.png"),
            ("Todos los archivos", "*.*")
        ]
        
        # Abrir diálogo de selección
        image_path = filedialog.askopenfilename(
            title="Selecciona una imagen",
            filetypes=filetypes,
            initialdir=os.getcwd()
        )
        
        root.destroy()
        return image_path if image_path else None
    
    def select_image_console(self):
        """
        Permite seleccionar una imagen desde la consola.
        
        Returns:
            str: Ruta de la imagen seleccionada o None si se cancela
        """
        print("\n=== SELECTOR DE IMAGEN ===")
        
        # Mostrar imágenes disponibles en carpetas comunes
        common_dirs = ['test', 'train', 'valid', '.', '..']
        available_images = []
        
        for directory in common_dirs:
            if os.path.exists(directory):
                for file in os.listdir(directory):
                    if file.lower().endswith(('.jpg', '.jpeg', '.png', '.bmp', '.tiff', '.gif')):
                        full_path = os.path.join(directory, file)
                        available_images.append(full_path)
        
        if available_images:
            print(f"\nImágenes disponibles ({len(available_images)} encontradas):")
            for i, img_path in enumerate(available_images[:10], 1):  # Mostrar solo las primeras 10
                print(f"  {i}. {img_path}")
            
            if len(available_images) > 10:
                print(f"  ... y {len(available_images) - 10} más")
            
            print(f"\nOpciones:")
            print(f"  1-{min(10, len(available_images))}: Seleccionar imagen de la lista")
            print(f"  0: Introducir ruta manualmente")
            print(f"  q: Salir")
            
            choice = input("\nTu elección: ").strip().lower()
            
            if choice == 'q':
                return None
            elif choice == '0':
                manual_path = input("Introduce la ruta de la imagen: ").strip()
                return manual_path if os.path.exists(manual_path) else None
            else:
                try:
                    index = int(choice) - 1
                    if 0 <= index < min(10, len(available_images)):
                        return available_images[index]
                except ValueError:
                    pass
        
        # Si no hay imágenes disponibles o selección inválida
        manual_path = input("Introduce la ruta completa de la imagen: ").strip()
        return manual_path if os.path.exists(manual_path) else None
    
    def preprocess_and_visualize(self, image_path, save_result=False):
        """
        Preprocesa una imagen y muestra los resultados.
        
        Args:
            image_path: Ruta de la imagen
            save_result: Si guardar la imagen procesada
        """
        try:
            print(f"\nProcesando imagen: {image_path}")
            
            # Cargar imagen original
            original_img = Image.open(image_path)
            print(f"Dimensiones originales: {original_img.size}")
            print(f"Modo de color: {original_img.mode}")
            
            # Preprocesar imagen
            processed_array = self.preprocessor.preprocess_image(image_path)
            print(f"Dimensiones procesadas: {processed_array.shape}")
            print(f"Rango de valores: [{processed_array.min():.3f}, {processed_array.max():.3f}]")
            
            # Crear visualización
            fig, axes = plt.subplots(1, 3, figsize=(18, 6))
            
            # Imagen original
            axes[0].imshow(original_img)
            axes[0].set_title(f'Original\n{original_img.size[0]}x{original_img.size[1]} - {original_img.mode}')
            axes[0].axis('off')
            
            # Imagen redimensionada (antes de normalizar)
            resized_img = original_img.resize((self.preprocessor.img_width, self.preprocessor.img_height))
            axes[1].imshow(resized_img)
            axes[1].set_title(f'Redimensionada\n{resized_img.size[0]}x{resized_img.size[1]}')
            axes[1].axis('off')
            
            # Imagen procesada (normalizada)
            axes[2].imshow(processed_array)
            axes[2].set_title(f'Procesada (Normalizada)\n{processed_array.shape[1]}x{processed_array.shape[0]} - Rango [0,1]')
            axes[2].axis('off')
            
            # Información adicional
            fig.suptitle(f'Preprocesamiento de: {os.path.basename(image_path)}', fontsize=14, fontweight='bold')
            
            plt.tight_layout()
            plt.show()
            
            # Guardar resultado si se solicita
            if save_result:
                self.save_processed_image(processed_array, image_path)
            
            # Guardar para referencia
            self.last_image_path = image_path
            self.last_processed = processed_array
            
            return processed_array
            
        except Exception as e:
            print(f"Error procesando la imagen: {e}")
            return None
    
    def save_processed_image(self, processed_array, original_path):
        """
        Guarda la imagen procesada.
        
        Args:
            processed_array: Array de la imagen procesada
            original_path: Ruta de la imagen original
        """
        try:
            # Crear directorio de imágenes si no existe
            images_dir = os.path.join(os.path.dirname(os.path.dirname(os.path.dirname(__file__))), "images")
            os.makedirs(images_dir, exist_ok=True)
            
            # Crear nombre de archivo de salida
            base_name = os.path.splitext(os.path.basename(original_path))[0]
            output_path = os.path.join(images_dir, f"{base_name}_processed.png")
            
            # Convertir array normalizado de vuelta a 0-255
            processed_img = (processed_array * 255).astype(np.uint8)
            
            # Guardar
            Image.fromarray(processed_img).save(output_path)
            print(f"✓ Imagen procesada guardada en: {output_path}")
            
        except Exception as e:
            print(f"Error guardando la imagen: {e}")
    
    def compare_settings(self, image_path):
        """
        Compara diferentes configuraciones de preprocesamiento.
        
        Args:
            image_path: Ruta de la imagen
        """
        try:
            original_img = Image.open(image_path)
            
            # Diferentes configuraciones
            configs = [
                {"size": (1920, 1200), "title": "Config Original (1920x1200)"},
                {"size": (960, 600), "title": "Config Pequeña (960x600)"},
                {"size": (640, 400), "title": "Config Mini (640x400)"}
            ]
            
            fig, axes = plt.subplots(2, 2, figsize=(15, 12))
            axes = axes.flatten()
            
            # Imagen original
            axes[0].imshow(original_img)
            axes[0].set_title(f'Original\n{original_img.size[0]}x{original_img.size[1]}')
            axes[0].axis('off')
            
            # Diferentes configuraciones
            for i, config in enumerate(configs, 1):
                if i < len(axes):
                    # Crear preprocesador con nueva configuración
                    temp_preprocessor = ImagePreprocessor(
                        img_height=config["size"][1], 
                        img_width=config["size"][0]
                    )
                    
                    # Procesar imagen
                    processed = temp_preprocessor.preprocess_image(image_path)
                    
                    # Mostrar
                    axes[i].imshow(processed)
                    axes[i].set_title(config["title"])
                    axes[i].axis('off')
            
            plt.suptitle(f'Comparación de Configuraciones - {os.path.basename(image_path)}')
            plt.tight_layout()
            plt.show()
            
        except Exception as e:
            print(f"Error en comparación: {e}")
    
    def show_processing_steps(self, image_path):
        """
        Muestra los pasos individuales del preprocesamiento.
        
        Args:
            image_path: Ruta de la imagen
        """
        try:
            # Cargar imagen original
            original_img = Image.open(image_path)
            
            # Paso 1: Redimensionar
            resized_img = original_img.resize((self.preprocessor.img_width, self.preprocessor.img_height))
            
            # Paso 2: Convertir a array
            img_array = np.array(resized_img)
            
            # Paso 3: Normalizar
            normalized_array = img_array / 255.0
            
            # Visualizar pasos
            fig, axes = plt.subplots(2, 2, figsize=(15, 12))
            
            # Original
            axes[0, 0].imshow(original_img)
            axes[0, 0].set_title(f'1. Original\n{original_img.size} - {original_img.mode}')
            axes[0, 0].axis('off')
            
            # Redimensionada
            axes[0, 1].imshow(resized_img)
            axes[0, 1].set_title(f'2. Redimensionada\n{resized_img.size}')
            axes[0, 1].axis('off')
            
            # Como array (0-255)
            axes[1, 0].imshow(img_array)
            axes[1, 0].set_title(f'3. Como Array\n{img_array.shape} - Rango [{img_array.min()}, {img_array.max()}]')
            axes[1, 0].axis('off')
            
            # Normalizada (0-1)
            axes[1, 1].imshow(normalized_array)
            axes[1, 1].set_title(f'4. Normalizada\n{normalized_array.shape} - Rango [{normalized_array.min():.3f}, {normalized_array.max():.3f}]')
            axes[1, 1].axis('off')
            
            plt.suptitle(f'Pasos del Preprocesamiento - {os.path.basename(image_path)}')
            plt.tight_layout()
            plt.show()
            
        except Exception as e:
            print(f"Error mostrando pasos: {e}")
    
    def interactive_mode(self):
        """Modo interactivo principal."""
        print("=== VISUALIZADOR DE PREPROCESAMIENTO DE IMÁGENES ===")
        print("Este script te permite visualizar cómo se preprocesan las imágenes")
        print("para el modelo de predicción de cartas.\n")
        
        while True:
            print("\n" + "="*50)
            print("MENÚ PRINCIPAL")
            print("="*50)
            print("1. Seleccionar imagen (GUI)")
            print("2. Seleccionar imagen (Consola)")
            print("3. Comparar configuraciones (última imagen)")
            print("4. Mostrar pasos de procesamiento (última imagen)")
            print("5. Configurar dimensiones del preprocesador")
            print("6. Salir")
            
            if self.last_image_path:
                print(f"\nÚltima imagen: {os.path.basename(self.last_image_path)}")
            
            choice = input("\nSelecciona una opción (1-6): ").strip()
            
            if choice == '1':
                image_path = self.select_image_gui()
                if image_path:
                    save = input("¿Guardar imagen procesada? (s/n): ").lower().startswith('s')
                    self.preprocess_and_visualize(image_path, save_result=save)
                else:
                    print("No se seleccionó ninguna imagen.")
            
            elif choice == '2':
                image_path = self.select_image_console()
                if image_path:
                    save = input("¿Guardar imagen procesada? (s/n): ").lower().startswith('s')
                    self.preprocess_and_visualize(image_path, save_result=save)
                else:
                    print("No se seleccionó ninguna imagen.")
            
            elif choice == '3':
                if self.last_image_path:
                    self.compare_settings(self.last_image_path)
                else:
                    print("Primero selecciona una imagen.")
            
            elif choice == '4':
                if self.last_image_path:
                    self.show_processing_steps(self.last_image_path)
                else:
                    print("Primero selecciona una imagen.")
            
            elif choice == '5':
                self.configure_preprocessor()
            
            elif choice == '6':
                break
            
            else:
                print("Opción no válida. Intenta de nuevo.")
    
    def configure_preprocessor(self):
        """Permite configurar las dimensiones del preprocesador."""
        print(f"\nConfiguración actual:")
        print(f"  Ancho: {self.preprocessor.img_width}")
        print(f"  Alto: {self.preprocessor.img_height}")
        
        try:
            new_width = input(f"Nuevo ancho (actual: {self.preprocessor.img_width}): ").strip()
            new_height = input(f"Nuevo alto (actual: {self.preprocessor.img_height}): ").strip()
            
            if new_width:
                self.preprocessor.img_width = int(new_width)
            if new_height:
                self.preprocessor.img_height = int(new_height)
            
            print(f"✓ Nueva configuración: {self.preprocessor.img_width}x{self.preprocessor.img_height}")
            
        except ValueError:
            print("Error: Introduce números válidos.")


def quick_test(image_path):
    """
    Función rápida para probar con una imagen específica.
    
    Args:
        image_path: Ruta de la imagen a procesar
    """
    if not os.path.exists(image_path):
        print(f"Error: La imagen {image_path} no existe.")
        return
    
    visualizer = ImagePreprocessorVisualizer()
    visualizer.preprocess_and_visualize(image_path, save_result=True)


def main():
    """Función principal."""
    import sys
    
    # Si se pasa una imagen como argumento, usarla directamente
    if len(sys.argv) > 1:
        image_path = sys.argv[1]
        print(f"Procesando imagen desde argumentos: {image_path}")
        quick_test(image_path)
    else:
        # Modo interactivo
        visualizer = ImagePreprocessorVisualizer()
        visualizer.interactive_mode()


if __name__ == "__main__":
    main()