"""
Utilidades para cargar y manejar modelos de TensorFlow/Keras.
"""
import os
import re
import tensorflow as tf
from tensorflow.keras.models import load_model


class ModelLoader:
    """
    Clase para cargar y manejar modelos de predicción.
    """
    
    def __init__(self, model_dir="modelos"):
        """
        Inicializa el cargador de modelos.
        
        Args:
            model_dir: Directorio donde se almacenan los modelos
        """
        self.model_dir = model_dir
        self.model = None
        self.model_path = None
    
    def find_latest_model(self):
        """
        Busca el modelo más reciente en la carpeta de modelos.
        
        Returns:
            str: Ruta del modelo más reciente o None si no hay modelos
        """
        if not os.path.exists(self.model_dir):
            print(f"Directorio {self.model_dir} no existe")
            return None
            
        modelos = [f for f in os.listdir(self.model_dir) if f.endswith('.h5')]
        if not modelos:
            print(f"No se encontraron modelos .h5 en {self.model_dir}")
            return None
        
        # Buscar modelos con patrón modelX.Y.h5
        modelos_versionados = []
        for modelo in modelos:
            match = re.match(r'model(\d+)\.(\d+)\.h5', modelo)
            if match:
                version_major = int(match.group(1))
                version_minor = int(match.group(2))
                modelos_versionados.append((modelo, version_major, version_minor))
        
        if modelos_versionados:
            # Ordenar por versión y tomar el más reciente
            modelo_mas_reciente = max(modelos_versionados, key=lambda x: (x[1], x[2]))[0]
        else:
            # Si no hay modelos versionados, tomar por fecha de modificación
            modelos_con_fecha = [(f, os.path.getmtime(os.path.join(self.model_dir, f))) for f in modelos]
            modelo_mas_reciente = max(modelos_con_fecha, key=lambda x: x[1])[0]
        
        return os.path.join(self.model_dir, modelo_mas_reciente)
    
    def load_model(self, model_path=None):
        """
        Carga un modelo de TensorFlow/Keras.
        
        Args:
            model_path: Ruta específica del modelo. Si es None, busca el más reciente.
            
        Returns:
            bool: True si el modelo se cargó correctamente, False en caso contrario
        """
        try:
            if model_path is None:
                model_path = self.find_latest_model()
            
            if model_path is None or not os.path.exists(model_path):
                print(f"Error: Modelo no encontrado en {model_path}")
                return False
            
            print(f"Cargando modelo desde: {model_path}")
            self.model = load_model(model_path)
            self.model_path = model_path
            
            print("✓ Modelo cargado correctamente")
            self.print_model_info()
            
            return True
            
        except Exception as e:
            print(f"Error cargando el modelo: {e}")
            return False
    
    def print_model_info(self):
        """
        Muestra información del modelo cargado.
        """
        if self.model is None:
            print("No hay modelo cargado")
            return
        
        print("\n=== Información del Modelo ===")
        print(f"Ruta: {self.model_path}")
        print(f"Input shape: {self.model.input_shape}")
        print(f"Output shape: {self.model.output_shape}")
        print(f"Número de parámetros: {self.model.count_params():,}")
        print("==============================\n")
    
    def is_loaded(self):
        """
        Verifica si hay un modelo cargado.
        
        Returns:
            bool: True si hay un modelo cargado
        """
        return self.model is not None
    
    def predict(self, input_data):
        """
        Realiza una predicción con el modelo cargado.
        
        Args:
            input_data: Datos de entrada para la predicción
            
        Returns:
            numpy array: Predicción del modelo
        """
        if not self.is_loaded():
            raise ValueError("No hay modelo cargado. Usa load_model() primero.")
        
        return self.model.predict(input_data, verbose=0)
    
    def get_model(self):
        """
        Retorna el modelo cargado.
        
        Returns:
            tensorflow.keras.Model: El modelo cargado o None
        """
        return self.model


def test_model_loading():
    """
    Función de prueba para cargar modelos.
    """
    print("=== Test de Carga de Modelos ===")
    
    loader = ModelLoader()
    
    # Intentar encontrar y cargar el modelo más reciente
    latest_model = loader.find_latest_model()
    
    if latest_model:
        print(f"Modelo más reciente encontrado: {latest_model}")
        success = loader.load_model()
        
        if success:
            print("✓ Modelo cargado exitosamente")
            
            # Probar una predicción dummy
            try:
                import numpy as np
                dummy_input = np.random.random((1, 1200, 1920, 3))  # Batch de 1 imagen
                prediction = loader.predict(dummy_input)
                print(f"✓ Predicción de prueba exitosa. Shape: {prediction.shape}")
                
            except Exception as e:
                print(f"Error en predicción de prueba: {e}")
        else:
            print("✗ Error cargando el modelo")
    else:
        print("No se encontraron modelos para cargar")
        print("Ejecuta model.py primero para entrenar un modelo")


if __name__ == "__main__":
    test_model_loading()
