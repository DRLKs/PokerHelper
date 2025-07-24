"""
Script interactivo para visualizar la detección de cartas.
Permite elegir una imagen y ver el resultado de la detección de cartas.
"""
import os
import sys
import matplotlib.pyplot as plt
import matplotlib.patches as patches
from PIL import Image
import numpy as np
from tkinter import filedialog, messagebox
import tkinter as tk
import cv2

# Añadir el directorio padre al path para importar los módulos
sys.path.append(os.path.dirname(os.path.dirname(os.path.abspath(__file__))))

from card_detector import CardDetector


class CardDetectorVisualizer:
    """
    Visualizador interactivo para la detección de cartas.
    """
    
    def __init__(self):
        """Inicializa el visualizador."""
        self.detector = CardDetector()
        self.last_image_path = None
        self.last_detections = None
        self.last_image = None
    
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
    
    def detect_and_visualize(self, image_path, save_result=False):
        """
        Detecta cartas en una imagen y muestra los resultados.
        
        Args:
            image_path: Ruta de la imagen
            save_result: Si guardar la imagen con detecciones
        """
        try:
            print(f"\nDetectando cartas en: {image_path}")
            
            # Cargar imagen original
            original_img = Image.open(image_path)
            img_array = np.array(original_img)
            
            print(f"Dimensiones originales: {original_img.size}")
            print(f"Modo de color: {original_img.mode}")
            
            # Detectar regiones de cartas
            card_regions = self.detector.detect_card_regions(img_array)
            print(f"Cartas detectadas: {len(card_regions)}")
            
            if card_regions:
                for i, (x, y, w, h) in enumerate(card_regions):
                    area = w * h
                    aspect_ratio = w / h if h > 0 else 0
                    print(f"  Carta {i+1}: x={x}, y={y}, w={w}, h={h}, área={area}, ratio={aspect_ratio:.2f}")
            
            # Crear visualización
            fig, axes = plt.subplots(1, 3, figsize=(18, 6))
            
            # Imagen original
            axes[0].imshow(original_img)
            axes[0].set_title(f'Original\n{original_img.size[0]}x{original_img.size[1]} - {len(card_regions)} cartas detectadas')
            axes[0].axis('off')
            
            # Imagen con detecciones marcadas
            axes[1].imshow(original_img)
            ax1 = axes[1]
            for i, (x, y, w, h) in enumerate(card_regions):
                # Crear rectángulo
                rect = patches.Rectangle((x, y), w, h, linewidth=2, edgecolor='red', facecolor='none')
                ax1.add_patch(rect)
                # Añadir número de carta
                ax1.text(x, y-5, f'{i+1}', color='red', fontsize=12, fontweight='bold', 
                        bbox=dict(boxstyle="round,pad=0.3", facecolor='white', edgecolor='red'))
            
            axes[1].set_title(f'Detecciones\nCartas encontradas: {len(card_regions)}')
            axes[1].axis('off')
            
            # Imagen procesada (escala de grises con bordes)
            gray = cv2.cvtColor(img_array, cv2.COLOR_RGB2GRAY)
            blurred = cv2.GaussianBlur(gray, (5, 5), 0)
            edges = cv2.Canny(blurred, 50, 150)
            
            axes[2].imshow(edges, cmap='gray')
            axes[2].set_title('Procesamiento\n(Detección de bordes)')
            axes[2].axis('off')
            
            # Información adicional
            fig.suptitle(f'Detección de Cartas: {os.path.basename(image_path)}', fontsize=14, fontweight='bold')
            
            plt.tight_layout()
            plt.show()
            
            # Guardar resultado si se solicita
            if save_result:
                self.save_detection_image(original_img, card_regions, image_path)
            
            # Guardar para referencia
            self.last_image_path = image_path
            self.last_detections = card_regions
            self.last_image = img_array
            
            return card_regions
            
        except Exception as e:
            print(f"Error detectando cartas: {e}")
            return None
    
    def save_detection_image(self, original_img, card_regions, original_path):
        """
        Guarda la imagen con las detecciones marcadas.
        
        Args:
            original_img: Imagen original PIL
            card_regions: Lista de regiones detectadas
            original_path: Ruta de la imagen original
        """
        try:
            # Crear directorio de imágenes si no existe
            images_dir = os.path.join(os.path.dirname(os.path.dirname(os.path.dirname(__file__))), "images")
            os.makedirs(images_dir, exist_ok=True)
            
            # Crear nombre de archivo de salida
            base_name = os.path.splitext(os.path.basename(original_path))[0]
            output_path = os.path.join(images_dir, f"{base_name}_detections.png")
            
            # Crear imagen con detecciones
            fig, ax = plt.subplots(1, 1, figsize=(12, 8))
            ax.imshow(original_img)
            
            for i, (x, y, w, h) in enumerate(card_regions):
                # Crear rectángulo
                rect = patches.Rectangle((x, y), w, h, linewidth=3, edgecolor='red', facecolor='none')
                ax.add_patch(rect)
                # Añadir número de carta
                ax.text(x, y-5, f'{i+1}', color='red', fontsize=14, fontweight='bold', 
                       bbox=dict(boxstyle="round,pad=0.3", facecolor='white', edgecolor='red'))
            
            ax.set_title(f'Detecciones: {len(card_regions)} cartas encontradas', fontsize=16)
            ax.axis('off')
            
            plt.tight_layout()
            plt.savefig(output_path, dpi=150, bbox_inches='tight')
            plt.close()
            
            print(f"✓ Imagen con detecciones guardada en: {output_path}")
            
        except Exception as e:
            print(f"Error guardando la imagen: {e}")
    
    def show_card_crops(self, image_path):
        """
        Muestra los recortes individuales de las cartas detectadas.
        
        Args:
            image_path: Ruta de la imagen
        """
        if not self.last_detections or not self.last_image is not None:
            print("Primero detecta cartas en una imagen.")
            return
            
        try:
            # Extraer recortes de cartas
            crops = self.detector.extract_card_crops(self.last_image, self.last_detections)
            
            if not crops:
                print("No hay recortes de cartas para mostrar.")
                return
            
            # Calcular disposición de subplots
            n_crops = len(crops)
            cols = min(4, n_crops)
            rows = (n_crops + cols - 1) // cols
            
            fig, axes = plt.subplots(rows, cols, figsize=(15, 4*rows))
            if n_crops == 1:
                axes = [axes]
            elif rows == 1:
                axes = [axes]
            else:
                axes = axes.flatten()
            
            for i, crop_info in enumerate(crops):
                if i < len(axes):
                    crop_img = crop_info['crop']
                    metadata = crop_info['metadata']
                    
                    axes[i].imshow(crop_img)
                    axes[i].set_title(f'Carta {i+1}\n{metadata["width"]}x{metadata["height"]}')
                    axes[i].axis('off')
            
            # Ocultar ejes sobrantes
            for i in range(n_crops, len(axes)):
                axes[i].axis('off')
            
            plt.suptitle(f'Recortes de Cartas: {os.path.basename(self.last_image_path)}')
            plt.tight_layout()
            plt.show()
            
        except Exception as e:
            print(f"Error mostrando recortes: {e}")
    
    def compare_settings(self, image_path):
        """
        Compara diferentes configuraciones de detección.
        
        Args:
            image_path: Ruta de la imagen
        """
        try:
            original_img = Image.open(image_path)
            img_array = np.array(original_img)
            
            # Diferentes configuraciones
            configs = [
                {"min_area": 3000, "max_area": 30000, "title": "Sensible (áreas pequeñas)"},
                {"min_area": 5000, "max_area": 50000, "title": "Normal (config actual)"},
                {"min_area": 8000, "max_area": 70000, "title": "Conservador (áreas grandes)"}
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
                    # Crear detector con nueva configuración
                    temp_detector = CardDetector(
                        min_area=config["min_area"],
                        max_area=config["max_area"],
                        min_aspect_ratio=self.detector.min_aspect_ratio,
                        max_aspect_ratio=self.detector.max_aspect_ratio
                    )
                    
                    # Detectar cartas
                    card_regions = temp_detector.detect_card_regions(img_array)
                    
                    # Mostrar
                    axes[i].imshow(original_img)
                    for j, (x, y, w, h) in enumerate(card_regions):
                        rect = patches.Rectangle((x, y), w, h, linewidth=2, edgecolor='red', facecolor='none')
                        axes[i].add_patch(rect)
                        axes[i].text(x, y-5, f'{j+1}', color='red', fontsize=10, fontweight='bold')
                    
                    axes[i].set_title(f'{config["title"]}\n{len(card_regions)} cartas')
                    axes[i].axis('off')
            
            plt.suptitle(f'Comparación de Configuraciones - {os.path.basename(image_path)}')
            plt.tight_layout()
            plt.show()
            
        except Exception as e:
            print(f"Error en comparación: {e}")
    
    def show_detection_steps(self, image_path):
        """
        Muestra los pasos individuales del proceso de detección.
        
        Args:
            image_path: Ruta de la imagen
        """
        try:
            # Cargar imagen original
            original_img = Image.open(image_path)
            img_array = np.array(original_img)
            
            # Paso 1: Escala de grises
            gray = cv2.cvtColor(img_array, cv2.COLOR_RGB2GRAY)
            
            # Paso 2: Blur gaussiano
            blurred = cv2.GaussianBlur(gray, (5, 5), 0)
            
            # Paso 3: Detección de bordes
            edges = cv2.Canny(blurred, 50, 150)
            
            # Paso 4: Encontrar contornos
            contours, _ = cv2.findContours(edges, cv2.RETR_EXTERNAL, cv2.CHAIN_APPROX_SIMPLE)
            
            # Visualizar pasos
            fig, axes = plt.subplots(2, 3, figsize=(18, 12))
            
            # Original
            axes[0, 0].imshow(original_img)
            axes[0, 0].set_title(f'1. Original\n{original_img.size} - {original_img.mode}')
            axes[0, 0].axis('off')
            
            # Escala de grises
            axes[0, 1].imshow(gray, cmap='gray')
            axes[0, 1].set_title('2. Escala de Grises')
            axes[0, 1].axis('off')
            
            # Blur gaussiano
            axes[0, 2].imshow(blurred, cmap='gray')
            axes[0, 2].set_title('3. Blur Gaussiano')
            axes[0, 2].axis('off')
            
            # Detección de bordes
            axes[1, 0].imshow(edges, cmap='gray')
            axes[1, 0].set_title('4. Detección de Bordes')
            axes[1, 0].axis('off')
            
            # Contornos encontrados
            contour_img = img_array.copy()
            cv2.drawContours(contour_img, contours, -1, (0, 255, 0), 2)
            axes[1, 1].imshow(contour_img)
            axes[1, 1].set_title(f'5. Contornos\n({len(contours)} encontrados)')
            axes[1, 1].axis('off')
            
            # Resultado final
            card_regions = self.detector.detect_card_regions(img_array)
            axes[1, 2].imshow(original_img)
            for i, (x, y, w, h) in enumerate(card_regions):
                rect = patches.Rectangle((x, y), w, h, linewidth=2, edgecolor='red', facecolor='none')
                axes[1, 2].add_patch(rect)
                axes[1, 2].text(x, y-5, f'{i+1}', color='red', fontsize=12, fontweight='bold')
            
            axes[1, 2].set_title(f'6. Cartas Filtradas\n({len(card_regions)} válidas)')
            axes[1, 2].axis('off')
            
            plt.suptitle(f'Pasos de Detección - {os.path.basename(image_path)}')
            plt.tight_layout()
            plt.show()
            
        except Exception as e:
            print(f"Error mostrando pasos: {e}")
    
    def interactive_mode(self):
        """Modo interactivo principal."""
        print("=== VISUALIZADOR DE DETECCIÓN DE CARTAS ===")
        print("Este script te permite visualizar cómo se detectan las cartas")
        print("en imágenes usando técnicas de visión por computadora.\n")
        
        while True:
            print("\n" + "="*50)
            print("MENÚ PRINCIPAL")
            print("="*50)
            print("1. Seleccionar imagen (GUI)")
            print("2. Seleccionar imagen (Consola)")
            print("3. Mostrar recortes de cartas (última imagen)")
            print("4. Comparar configuraciones (última imagen)")
            print("5. Mostrar pasos de detección (última imagen)")
            print("6. Configurar parámetros del detector")
            print("7. Salir")
            
            if self.last_image_path:
                print(f"\nÚltima imagen: {os.path.basename(self.last_image_path)}")
                if self.last_detections:
                    print(f"Cartas detectadas: {len(self.last_detections)}")
            
            choice = input("\nSelecciona una opción (1-7): ").strip()
            
            if choice == '1':
                image_path = self.select_image_gui()
                if image_path:
                    save = input("¿Guardar imagen con detecciones? (s/n): ").lower().startswith('s')
                    self.detect_and_visualize(image_path, save_result=save)
                else:
                    print("No se seleccionó ninguna imagen.")
            
            elif choice == '2':
                image_path = self.select_image_console()
                if image_path:
                    save = input("¿Guardar imagen con detecciones? (s/n): ").lower().startswith('s')
                    self.detect_and_visualize(image_path, save_result=save)
                else:
                    print("No se seleccionó ninguna imagen.")
            
            elif choice == '3':
                if self.last_image_path:
                    self.show_card_crops(self.last_image_path)
                else:
                    print("Primero selecciona una imagen.")
            
            elif choice == '4':
                if self.last_image_path:
                    self.compare_settings(self.last_image_path)
                else:
                    print("Primero selecciona una imagen.")
            
            elif choice == '5':
                if self.last_image_path:
                    self.show_detection_steps(self.last_image_path)
                else:
                    print("Primero selecciona una imagen.")
            
            elif choice == '6':
                self.configure_detector()
            
            elif choice == '7':
                print("¡Hasta luego!")
                break
            
            else:
                print("Opción no válida. Intenta de nuevo.")
    
    def configure_detector(self):
        """Permite configurar los parámetros del detector."""
        print(f"\nConfiguración actual:")
        print(f"  Área mínima: {self.detector.min_area}")
        print(f"  Área máxima: {self.detector.max_area}")
        print(f"  Ratio aspecto mín: {self.detector.min_aspect_ratio}")
        print(f"  Ratio aspecto máx: {self.detector.max_aspect_ratio}")
        
        try:
            new_min_area = input(f"Nueva área mínima (actual: {self.detector.min_area}): ").strip()
            new_max_area = input(f"Nueva área máxima (actual: {self.detector.max_area}): ").strip()
            new_min_ratio = input(f"Nuevo ratio mínimo (actual: {self.detector.min_aspect_ratio}): ").strip()
            new_max_ratio = input(f"Nuevo ratio máximo (actual: {self.detector.max_aspect_ratio}): ").strip()
            
            if new_min_area:
                self.detector.min_area = int(new_min_area)
            if new_max_area:
                self.detector.max_area = int(new_max_area)
            if new_min_ratio:
                self.detector.min_aspect_ratio = float(new_min_ratio)
            if new_max_ratio:
                self.detector.max_aspect_ratio = float(new_max_ratio)
            
            print(f"✓ Nueva configuración aplicada")
            
        except ValueError:
            print("Error: Introduce valores válidos.")


def quick_test(image_path):
    """
    Función rápida para probar con una imagen específica.
    
    Args:
        image_path: Ruta de la imagen a procesar
    """
    if not os.path.exists(image_path):
        print(f"Error: La imagen {image_path} no existe.")
        return
    
    visualizer = CardDetectorVisualizer()
    visualizer.detect_and_visualize(image_path, save_result=True)


def main():
    """Función principal."""
    import sys
    
    # Si se pasa una imagen como argumento, usarla directamente
    if len(sys.argv) > 1:
        image_path = sys.argv[1]
        print(f"Detectando cartas desde argumentos: {image_path}")
        quick_test(image_path)
    else:
        # Modo interactivo
        visualizer = CardDetectorVisualizer()
        visualizer.interactive_mode()


if __name__ == "__main__":
    main()
