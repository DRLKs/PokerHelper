"""
Router para endpoints relacionados con la predicción de cartas.
"""
from fastapi import APIRouter, HTTPException, UploadFile, File, Depends
from typing import List, Dict, Any
import os
import sys
from pathlib import Path

# Añadir el directorio raíz al path
root_dir = Path(__file__).parent.parent.parent
sys.path.append(str(root_dir))

from predictor.CardPredictor import CardPredictor
from api.models.response import CardAnalysisResponse, CardDetection
from api.services.card_service import CardService
from api.config.settings import get_settings, Settings
from api.utils.logger import setup_logger

logger = setup_logger(__name__)
router = APIRouter()

# Dependencia para el servicio de cartas
def get_card_service(settings: Settings = Depends(get_settings)) -> CardService:
    return CardService(settings)

@router.post("/analyze", response_model=CardAnalysisResponse)
async def analyze_cards(
    app: Application,
    card_service: CardService = Depends(get_card_service)
):
    """
    Analiza una imagen que hace el sistema para detectar y predecir cartas.
    
    Args:
        app: Aplicación a la que se le hace la captura de pantalla
        card_service: Servicio de cartas

    Returns:
        CardAnalysisResponse: Resultados del análisis
    """
    try:
        
        # Hacer captura de pantalla
        # file = app.screenshot()

        # Validar archivo
        if not file.filename:
            raise HTTPException(status_code=400, detail="No se proporcionó archivo")
        
        # Validar extensión de la imagen
        ext = os.path.splitext(file.filename)[1].lower()
        settings = get_settings()
        if ext not in settings.allowed_extensions:
            raise HTTPException(
                status_code=400, 
                detail=f"Extensión no permitida. Permitidas: {settings.allowed_extensions}"
            )
        
        # Validar tamaño
        content = await file.read()
        if len(content) > settings.max_file_size:
            raise HTTPException(
                status_code=400, 
                detail=f"Archivo demasiado grande. Máximo: {settings.max_file_size} bytes"
            )
        
        # Analizar imagen
        result = await card_service.analyze_image(
            content, 
            file.filename,      # Quizás deberiamos hacer la captura de pantalla aquí o pasarle la captura
            show_results=show_results,
            save_results=save_results
        )
        
        logger.info(f"Análisis completado: {result.total_cards_detected} cartas detectadas")
        return result
        
    except HTTPException:
        raise
    except Exception as e:
        logger.error(f"Error analizando imagen: {e}", exc_info=True)
        raise HTTPException(status_code=500, detail=f"Error procesando imagen: {str(e)}")


@router.get("/model/status")
async def get_model_status(card_service: CardService = Depends(get_card_service)):
    """
    Obtiene el estado del modelo de predicción.
    
    Returns:
        dict: Estado del modelo
    """
    try:
        status = card_service.get_model_status()
        return status
    except Exception as e:
        logger.error(f"Error obteniendo estado del modelo: {e}", exc_info=True)
        raise HTTPException(status_code=500, detail=f"Error obteniendo estado: {str(e)}")

@router.post("/model/reload")
async def reload_model(
    model_path: str = None,
    card_service: CardService = Depends(get_card_service)
):
    """
    Recarga el modelo de predicción.
    
    Args:
        model_path: Ruta opcional del nuevo modelo
        
    Returns:
        dict: Resultado de la recarga
    """
    try:
        result = await card_service.reload_model(model_path)
        logger.info(f"Modelo recargado: {result}")
        return result
    except Exception as e:
        logger.error(f"Error recargando modelo: {e}", exc_info=True)
        raise HTTPException(status_code=500, detail=f"Error recargando modelo: {str(e)}")
