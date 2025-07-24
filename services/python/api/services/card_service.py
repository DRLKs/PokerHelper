"""
Servicio para el manejo de la lógica de negocio relacionada con cartas.
"""
import time
import io
from PIL import Image
import numpy as np
from datetime import datetime
from typing import Optional
import sys
from pathlib import Path

# Añadir el directorio raíz al path
root_dir = Path(__file__).parent.parent.parent
sys.path.append(str(root_dir))

from predictor.CardPredictor import CardPredictor
from api.models.response import (
    CardAnalysisResponse, 
    CardDetection, 
    CardPrediction, 
    ImageInfo,
    ModelStatus
)
from api.config.settings import Settings
from api.utils.logger import setup_logger

logger = setup_logger(__name__)

class CardService:
    """Servicio para la predicción y análisis de cartas."""
    
    def __init__(self, settings: Settings):
        """
        Inicializa el servicio de cartas.
        
        Args:
            settings: Configuración de la aplicación
        """
        self.settings = settings
        self._predictor: Optional[CardPredictor] = None
        self._initialize_predictor()
    
    def _initialize_predictor(self):
        """Inicializa el predictor de cartas."""
        try:
            logger.info("Inicializando predictor de cartas...")
            self._predictor = CardPredictor(
                model_path=self.settings.model_path,
                img_height=self.settings.img_height,
                img_width=self.settings.img_width
            )
            logger.info("✓ Predictor inicializado correctamente")
        except Exception as e:
            logger.error(f"Error inicializando predictor: {e}", exc_info=True)
            self._predictor = None
    
    async def analyze_image(
        self, 
        image_content: bytes, 
        filename: str,
        show_results: bool = False,
        save_results: bool = False
    ) -> CardAnalysisResponse:
        """
        Analiza una imagen para detectar y predecir cartas.
        
        Args:
            image_content: Contenido binario de la imagen
            filename: Nombre del archivo
            show_results: Si mostrar resultados visual
            save_results: Si guardar los resultados
            
        Returns:
            CardAnalysisResponse: Resultados del análisis
        """
        start_time = time.time()
        
        try:
            # Verificar que el predictor esté disponible
            if not self._predictor:
                raise Exception("Predictor no disponible")
            
            # Cargar imagen desde bytes
            image = Image.open(io.BytesIO(image_content))
            image_array = np.array(image)
            
            # Información de la imagen
            image_info = ImageInfo(
                filename=filename,
                width=image.size[0],
                height=image.size[1],
                channels=len(image_array.shape) if len(image_array.shape) > 2 else 1,
                format=image.format,
                size_bytes=len(image_content)
            )
            
            logger.info(f"Analizando imagen: {filename} ({image.size})")
            
            # Realizar análisis
            analysis_result = self._predictor.analyze_image(
                image_array,
                show_results=show_results,
                save_results=save_results
            )
            
            # Convertir resultados al formato de la API
            cards = []
            for result in analysis_result.get('results', []):
                # Convertir predicción
                pred_data = result.get('prediction', {})
                prediction = CardPrediction(
                    class_index=pred_data.get('class_index'),
                    confidence=pred_data.get('confidence'),
                    raw_prediction=pred_data.get('raw_prediction'),
                    error=pred_data.get('error')
                )
                
                # Crear detección
                detection = CardDetection(
                    crop_id=result.get('crop_id', 'unknown'),
                    bbox=result.get('bbox', [0, 0, 0, 0]),
                    area=result.get('area', 0),
                    aspect_ratio=result.get('aspect_ratio', 0.0),
                    prediction=prediction
                )
                cards.append(detection)
            
            processing_time = time.time() - start_time
            
            response = CardAnalysisResponse(
                total_cards_detected=len(cards),
                cards=cards,
                image_info=image_info,
                model_loaded=analysis_result.get('model_loaded', False),
                processing_time=processing_time
            )
            
            logger.info(f"Análisis completado en {processing_time:.2f}s: {len(cards)} cartas")
            return response
            
        except Exception as e:
            processing_time = time.time() - start_time
            logger.error(f"Error en análisis: {e}", exc_info=True)
            
            # Retornar respuesta con error
            return CardAnalysisResponse(
                total_cards_detected=0,
                cards=[],
                image_info=ImageInfo(
                    filename=filename,
                    width=0,
                    height=0,
                    channels=0,
                    error=str(e)
                ),
                model_loaded=bool(self._predictor),
                processing_time=processing_time
            )
    
    def get_model_status(self) -> ModelStatus:
        """
        Obtiene el estado actual del modelo.
        
        Returns:
            ModelStatus: Estado del modelo
        """
        try:
            if not self._predictor:
                return ModelStatus(
                    loaded=False,
                    error="Predictor no inicializado"
                )
            
            model_loaded = self._predictor.model_loader.is_loaded() if self._predictor.model_loader else False
            
            return ModelStatus(
                loaded=model_loaded,
                model_path=self.settings.model_path,
                model_name="CardPredictor",
                input_shape=[self.settings.img_height, self.settings.img_width, 3],
                last_loaded=datetime.utcnow() if model_loaded else None
            )
            
        except Exception as e:
            logger.error(f"Error obteniendo estado del modelo: {e}", exc_info=True)
            return ModelStatus(
                loaded=False,
                error=str(e)
            )
    
    async def reload_model(self, model_path: Optional[str] = None) -> dict:
        """
        Recarga el modelo de predicción.
        
        Args:
            model_path: Ruta opcional del nuevo modelo
            
        Returns:
            dict: Resultado de la recarga
        """
        try:
            logger.info(f"Recargando modelo: {model_path or 'default'}")
            
            # Actualizar configuración si se proporciona nueva ruta
            if model_path:
                self.settings.model_path = model_path
            
            # Reinicializar predictor
            self._initialize_predictor()
            
            if self._predictor:
                status = self.get_model_status()
                return {
                    "success": True,
                    "message": "Modelo recargado exitosamente",
                    "model_status": status.dict(),
                    "reloaded_at": datetime.utcnow()
                }
            else:
                return {
                    "success": False,
                    "message": "Error recargando modelo",
                    "reloaded_at": datetime.utcnow()
                }
                
        except Exception as e:
            logger.error(f"Error recargando modelo: {e}", exc_info=True)
            return {
                "success": False,
                "message": f"Error recargando modelo: {str(e)}",
                "reloaded_at": datetime.utcnow()
            }
