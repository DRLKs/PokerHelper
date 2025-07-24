"""
Punto de entrada principal para la API FastAPI del servicio de predicción de cartas.
"""
from fastapi import FastAPI, Request
from fastapi.middleware.cors import CORSMiddleware
from fastapi.staticfiles import StaticFiles
from fastapi.responses import JSONResponse
import uvicorn
import os
import sys
from pathlib import Path

# Añadir el directorio raíz al path para importar módulos
root_dir = Path(__file__).parent.parent
sys.path.append(str(root_dir))

from api.routers import cards, health, upload
from api.config.settings import get_settings
from api.utils.logger import setup_logger

# Configurar logger
logger = setup_logger(__name__)

# Configuración
settings = get_settings()

# Crear aplicación FastAPI
app = FastAPI(
    title="PokerHelper Card Prediction API",
    description="API para la detección y predicción de cartas de poker usando visión por computadora",
    version="1.0.0",
    docs_url="/docs",
    redoc_url="/redoc"
)

# Configurar CORS para permitir peticiones solo desde el mismo dispositivo
app.add_middleware(
    CORSMiddleware,
    allow_origins=settings.cors_origins if settings.cors_origins != ["*"] else [
        "http://localhost:3000",      # React/Vue development server
        "http://localhost:8080",      # Vue development server
        "http://localhost:5173",      # Vite development server
        "http://127.0.0.1:3000",     # Alternative localhost
        "http://127.0.0.1:8080",     # Alternative localhost
        "http://127.0.0.1:5173",     # Alternative localhost
        "file://",                   # Electron apps
        "capacitor://localhost",     # Capacitor apps
        "tauri://localhost",         # Tauri apps
    ],
    allow_credentials=True,
    allow_methods=["GET", "POST", "PUT", "DELETE", "OPTIONS"],
    allow_headers=["*"],
)

# Incluir routers
app.include_router(
    health.router,
    prefix="/api/health",
    tags=["health"]
)

app.include_router(
    upload.router,
    prefix="/api/upload",
    tags=["upload"]
)

app.include_router(
    cards.router,
    prefix="/api/cards",
    tags=["cards"]
)

# Servir archivos estáticos (uploads, etc.)
uploads_dir = os.path.join(os.path.dirname(__file__), "..", "uploads")
os.makedirs(uploads_dir, exist_ok=True)
app.mount("/uploads", StaticFiles(directory=uploads_dir), name="uploads")

# Manejador de errores global
@app.exception_handler(Exception)
async def global_exception_handler(request: Request, exc: Exception):
    logger.error(f"Error no manejado: {exc}", exc_info=True)
    return JSONResponse(
        status_code=500,
        content={
            "error": "Error interno del servidor",
            "detail": str(exc) if settings.debug else "Ha ocurrido un error inesperado"
        }
    )

# Eventos de inicio y cierre
@app.on_event("startup")
async def startup_event():
    logger.info("Iniciando PokerHelper Card Prediction API")
    logger.info(f"Modo debug: {settings.debug}")
    logger.info(f"Versión: {app.version}")

@app.on_event("shutdown")
async def shutdown_event():
    logger.info("🛑 Cerrando PokerHelper Card Prediction API")

# Endpoint raíz
@app.get("/")
async def root():
    return {
        "message": "PokerHelper Card Prediction API",
        "version": app.version,
        "docs": "/docs",
        "status": "running"
    }

# Función principal para ejecutar el servidor
def main():
    uvicorn.run(
        "api.main:app",
        host=settings.host,
        port=settings.port,
        reload=settings.debug,
        log_level="info"
    )

if __name__ == "__main__":
    main()
