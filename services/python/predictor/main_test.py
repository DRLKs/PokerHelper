"""
Script principal para probar toda la funcionalidad del predictor de cartas.
"""
import os
import sys
from PIL import Image
import numpy as np

# Importar nuestros módulos
from card_detector import CardDetector, test_card_detection
from image_preprocessor import ImagePreprocessor, test_preprocessing  
from model_loader import ModelLoader, test_model_loading
from visualizer import ResultVisualizer, test_visualization
from card_predictor import CardPredictor, analyze_image_file


def print_header(title):
    """Imprime un encabezado con formato."""
    print("\n" + "="*50)
    print(f"  {title}")
    print("="*50)


def test_individual_components():
    """
    Prueba cada componente individualmente.
    """
    print_header("PRUEBAS DE COMPONENTES INDIVIDUALES")
    
    # 1. Probar detector de cartas
    print("\n1. Probando CardDetector...")
    try:
        test_card_detection()
        print("✓ CardDetector funcionando correctamente")
    except Exception as e:
        print(f"✗ Error en CardDetector: {e}")
    
    # 2. Probar preprocesador
    print("\n2. Probando ImagePreprocessor...")
    try:
        test_preprocessing()
        print("✓ ImagePreprocessor funcionando correctamente")
    except Exception as e:
        print(f"✗ Error en ImagePreprocessor: {e}")
    
    # 3. Probar cargador de modelos
    print("\n3. Probando ModelLoader...")
    try:
        test_model_loading()
        print("✓ ModelLoader funcionando correctamente")
    except Exception as e:
        print(f"✗ Error en ModelLoader: {e}")
    
    # 4. Probar visualizador
    print("\n4. Probando ResultVisualizer...")
    try:
        test_visualization()
        print("✓ ResultVisualizer funcionando correctamente")
    except Exception as e:
        print(f"✗ Error en ResultVisualizer: {e}")


def test_integrated_system():
    """
    Prueba el sistema integrado completo.
    """
    print_header("PRUEBA DEL SISTEMA INTEGRADO")
    
    # Buscar imágenes de prueba
    test_images = []
    if os.path.exists('test'):
        test_images = [os.path.join('test', f) for f in os.listdir('test') 
                      if f.lower().endswith(('.jpg', '.jpeg', '.png'))]
    
    if not test_images:
        print("No se encontraron imágenes de prueba en la carpeta 'test'")
        print("Por favor, coloca algunas imágenes .jpg o .png en la carpeta 'test'")
        return
    
    print(f"Encontradas {len(test_images)} imágenes de prueba")
    
    # Probar con las primeras 3 imágenes
    for i, image_path in enumerate(test_images[:3]):
        print(f"\n--- Procesando imagen {i+1}: {os.path.basename(image_path)} ---")
        
        try:
            # Análizar imagen completa
            results = analyze_image_file(image_path, show_results=False)
            
            # Mostrar resumen
            print(f"✓ Imagen procesada exitosamente")
            print(f"  - Dimensiones: {results['image_shape']}")
            print(f"  - Cartas detectadas: {results['total_cards_detected']}")
            print(f"  - Modelo cargado: {results['model_loaded']}")
            
            # Mostrar detalles de las primeras cartas
            for j, result in enumerate(results['results'][:3]):
                x, y, w, h = result['bbox']
                pred = result['prediction']
                
                print(f"  Carta {j+1}:")
                print(f"    Posición: ({x}, {y}), Tamaño: {w}x{h}")
                print(f"    Área: {result['area']}")
                
                if 'error' not in pred:
                    print(f"    Predicción: Clase {pred['class_index']} (confianza: {pred['confidence']:.3f})")
                else:
                    print(f"    Error: {pred['error']}")
            
        except Exception as e:
            print(f"✗ Error procesando {image_path}: {e}")


def check_dependencies():
    """
    Verifica que todas las dependencias estén disponibles.
    """
    print_header("VERIFICACIÓN DE DEPENDENCIAS")
    
    dependencies = {
        'numpy': 'numpy',
        'PIL': 'Pillow',
        'matplotlib': 'matplotlib',
        'pandas': 'pandas',
        'cv2': 'opencv-python',
        'tensorflow': 'tensorflow'
    }
    
    missing_deps = []
    
    for module, package in dependencies.items():
        try:
            __import__(module)
            print(f"✓ {module} ({package}) - Disponible")
        except ImportError:
            print(f"✗ {module} ({package}) - FALTANTE")
            missing_deps.append(package)
    
    if missing_deps:
        print(f"\nDependencias faltantes: {', '.join(missing_deps)}")
        print("Instálalas con: python -m pip install " + " ".join(missing_deps))
        return False
    else:
        print("\n✓ Todas las dependencias están disponibles")
        return True


def show_project_structure():
    """
    Muestra la estructura del proyecto.
    """
    print_header("ESTRUCTURA DEL PROYECTO")
    
    files_info = {
        'card_detector.py': 'Detecta regiones de cartas en imágenes',
        'image_preprocessor.py': 'Preprocesa imágenes para el modelo',
        'model_loader.py': 'Carga y maneja modelos de TensorFlow',
        'visualizer.py': 'Visualiza resultados de detección/predicción',
        'card_predictor.py': 'Predictor principal que integra todo',
        'main_test.py': 'Script principal de pruebas (este archivo)',
        'model.py': 'Script original de entrenamiento',
        'AnalizadorCartas.py': 'Capturador de pantallas de PokerStars',
        'requirements.txt': 'Lista de dependencias del proyecto'
    }
    
    print("Archivos del proyecto:")
    for filename, description in files_info.items():
        if os.path.exists(filename):
            print(f"  ✓ {filename:<25} - {description}")
        else:
            print(f"  ✗ {filename:<25} - {description} (FALTANTE)")
    
    # Mostrar carpetas importantes
    folders = ['test', 'train', 'valid', 'anotaciones', 'modelos']
    print("\nCarpetas importantes:")
    for folder in folders:
        if os.path.exists(folder):
            file_count = len([f for f in os.listdir(folder) if os.path.isfile(os.path.join(folder, f))])
            print(f"  ✓ {folder:<15} - {file_count} archivos")
        else:
            print(f"  ✗ {folder:<15} - No existe")


def main():
    """
    Función principal que ejecuta todas las pruebas.
    """
    print_header("PROBADOR DEL SISTEMA DE PREDICCIÓN DE CARTAS")
    print("Este script probará todos los componentes del sistema paso a paso.")
    
    # 1. Mostrar estructura del proyecto
    show_project_structure()
    
    # 2. Verificar dependencias
    deps_ok = check_dependencies()
    
    if not deps_ok:
        print("\n⚠️  Algunas dependencias faltan. El sistema puede no funcionar correctamente.")
        response = input("¿Continuar con las pruebas? (s/n): ")
        if response.lower() not in ['s', 'si', 'y', 'yes']:
            return
    
    # 3. Probar componentes individuales
    try:
        test_individual_components()
    except KeyboardInterrupt:
        print("\nPruebas interrumpidas por el usuario.")
        return
    except Exception as e:
        print(f"\nError en pruebas de componentes: {e}")
    
    # 4. Probar sistema integrado
    try:
        test_integrated_system()
    except KeyboardInterrupt:
        print("\nPruebas interrumpidas por el usuario.")
        return
    except Exception as e:
        print(f"\nError en pruebas del sistema integrado: {e}")
    
    # 5. Resumen final
    print_header("PRUEBAS COMPLETADAS")
    print("Si llegaste hasta aquí, el sistema básico está funcionando.")
    print("\nPróximos pasos:")
    print("1. Entrena un modelo usando model.py")
    print("2. Coloca más imágenes de prueba en la carpeta 'test'")
    print("3. Usa card_predictor.py para análisis de producción")
    print("4. Integra con AnalizadorCartas.py para capturas en tiempo real")


if __name__ == "__main__":
    main()
