"""
Router para endpoints de salud del servicio.
"""
from fastapi import APIRouter
from datetime import datetime
import psutil
import os

router = APIRouter()

@router.get("/")
async def health_check():
    """
    Endpoint básico de health check.
    
    Returns:
        dict: Estado del servicio
    """
    return {
        "status": "healthy",
        "timestamp": datetime.utcnow(),
        "service": "PokerHelper Card Prediction API"
    }

@router.get("/detailed")
async def detailed_health():
    """
    Health check detallado con métricas del sistema.
    
    Returns:
        dict: Estado detallado del servicio
    """
    # Obtener métricas del sistema
    cpu_percent = psutil.cpu_percent(interval=1)
    memory = psutil.virtual_memory()
    disk = psutil.disk_usage('/')
    
    return {
        "status": "healthy",
        "timestamp": datetime.utcnow(),
        "service": "PokerHelper Card Prediction API",
        "system": {
            "cpu_usage_percent": cpu_percent,
            "memory": {
                "total_gb": round(memory.total / (1024**3), 2),
                "available_gb": round(memory.available / (1024**3), 2),
                "used_percent": memory.percent
            },
            "disk": {
                "total_gb": round(disk.total / (1024**3), 2),
                "free_gb": round(disk.free / (1024**3), 2),
                "used_percent": round((disk.used / disk.total) * 100, 2)
            }
        },
        "process": {
            "pid": os.getpid(),
            "python_version": f"{psutil.version_info.major}.{psutil.version_info.minor}"
        }
    }
