"""
Modelos Pydantic para las respuestas de la API.
"""
from pydantic import BaseModel
from typing import List, Dict, Any, Optional
from datetime import datetime

class CardPrediction(BaseModel):
    """Modelo para la predicción de una carta individual."""
    class_index: Optional[int] = None
    confidence: Optional[float] = None
    predicted_value: Optional[str] = None
    predicted_suit: Optional[str] = None
    raw_prediction: Optional[List[float]] = None
    error: Optional[str] = None

class CardDetection(BaseModel):
    """Modelo para la detección de una carta."""
    crop_id: str
    bbox: List[int]  # [x, y, width, height]
    area: int
    aspect_ratio: float
    prediction: CardPrediction

class ImageInfo(BaseModel):
    """Información sobre la imagen analizada."""
    filename: str
    width: int
    height: int
    channels: int
    format: Optional[str] = None
    size_bytes: Optional[int] = None
    error: Optional[str] = None

class CardAnalysisResponse(BaseModel):
    """Respuesta del análisis de cartas."""
    total_cards_detected: int
    cards: List[CardDetection]
    image_info: ImageInfo
    model_loaded: bool
    processing_time: float
    timestamp: datetime = datetime.utcnow()
    
    class Config:
        json_encoders = {
            datetime: lambda v: v.isoformat()
        }

class ModelStatus(BaseModel):
    """Estado del modelo de ML."""
    loaded: bool
    model_path: Optional[str] = None
    model_name: Optional[str] = None
    input_shape: Optional[List[int]] = None
    num_classes: Optional[int] = None
    last_loaded: Optional[datetime] = None
    error: Optional[str] = None
    
    class Config:
        json_encoders = {
            datetime: lambda v: v.isoformat()
        }

class BatchAnalysisResponse(BaseModel):
    """Respuesta del análisis por lotes."""
    total_images: int
    successful_analyses: int
    failed_analyses: int
    results: List[CardAnalysisResponse]
    total_processing_time: float
    timestamp: datetime = datetime.utcnow()
    
    class Config:
        json_encoders = {
            datetime: lambda v: v.isoformat()
        }
