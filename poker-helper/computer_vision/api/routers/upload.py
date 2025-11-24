"""
Router para manejo de uploads de archivos.
"""
from fastapi import APIRouter, HTTPException, UploadFile, File, Depends
from fastapi.responses import FileResponse
import os
import uuid
from pathlib import Path
from datetime import datetime

from api.config.settings import get_settings, Settings
from api.utils.logger import setup_logger

logger = setup_logger(__name__)
router = APIRouter()

@router.post("/image")
async def upload_image(
    file: UploadFile = File(...),
    settings: Settings = Depends(get_settings)
):
    """
    Sube una imagen al servidor.
    
    Args:
        file: Archivo de imagen a subir
        
    Returns:
        dict: Información del archivo subido
    """
    try:
        # Validar archivo
        if not file.filename:
            raise HTTPException(status_code=400, detail="No se proporcionó archivo")
        
        # Validar extensión
        ext = os.path.splitext(file.filename)[1].lower()
        if ext not in settings.allowed_extensions:
            raise HTTPException(
                status_code=400, 
                detail=f"Extensión no permitida. Permitidas: {settings.allowed_extensions}"
            )
        
        # Leer contenido y validar tamaño
        content = await file.read()
        if len(content) > settings.max_file_size:
            raise HTTPException(
                status_code=400, 
                detail=f"Archivo demasiado grande. Máximo: {settings.max_file_size} bytes"
            )
        
        # Crear directorio de uploads si no existe
        upload_dir = Path(settings.upload_dir)
        upload_dir.mkdir(exist_ok=True)
        
        # Generar nombre único para el archivo
        unique_filename = f"{uuid.uuid4()}{ext}"
        file_path = upload_dir / unique_filename
        
        # Guardar archivo
        with open(file_path, "wb") as f:
            f.write(content)
        
        # Información del archivo
        file_info = {
            "filename": file.filename,
            "saved_as": unique_filename,
            "size_bytes": len(content),
            "extension": ext,
            "upload_time": datetime.utcnow(),
            "url": f"/uploads/{unique_filename}"
        }
        
        logger.info(f"Archivo subido: {file.filename} -> {unique_filename}")
        return file_info
        
    except HTTPException:
        raise
    except Exception as e:
        logger.error(f"Error subiendo archivo: {e}", exc_info=True)
        raise HTTPException(status_code=500, detail=f"Error subiendo archivo: {str(e)}")

@router.get("/image/{filename}")
async def get_uploaded_image(
    filename: str,
    settings: Settings = Depends(get_settings)
):
    """
    Descarga una imagen previamente subida.
    
    Args:
        filename: Nombre del archivo a descargar
        
    Returns:
        FileResponse: Archivo solicitado
    """
    try:
        file_path = Path(settings.upload_dir) / filename
        
        if not file_path.exists():
            raise HTTPException(status_code=404, detail="Archivo no encontrado")
        
        return FileResponse(
            path=file_path,
            filename=filename,
            media_type="image/*"
        )
        
    except HTTPException:
        raise
    except Exception as e:
        logger.error(f"Error descargando archivo: {e}", exc_info=True)
        raise HTTPException(status_code=500, detail=f"Error descargando archivo: {str(e)}")

@router.delete("/image/{filename}")
async def delete_uploaded_image(
    filename: str,
    settings: Settings = Depends(get_settings)
):
    """
    Elimina una imagen previamente subida.
    
    Args:
        filename: Nombre del archivo a eliminar
        
    Returns:
        dict: Confirmación de eliminación
    """
    try:
        file_path = Path(settings.upload_dir) / filename
        
        if not file_path.exists():
            raise HTTPException(status_code=404, detail="Archivo no encontrado")
        
        file_path.unlink()
        
        logger.info(f"Archivo eliminado: {filename}")
        return {
            "message": "Archivo eliminado exitosamente",
            "filename": filename,
            "deleted_at": datetime.utcnow()
        }
        
    except HTTPException:
        raise
    except Exception as e:
        logger.error(f"Error eliminando archivo: {e}", exc_info=True)
        raise HTTPException(status_code=500, detail=f"Error eliminando archivo: {str(e)}")

@router.get("/list")
async def list_uploaded_images(settings: Settings = Depends(get_settings)):
    """
    Lista todas las imágenes subidas.
    
    Returns:
        dict: Lista de archivos subidos
    """
    try:
        upload_dir = Path(settings.upload_dir)
        
        if not upload_dir.exists():
            return {"files": []}
        
        files = []
        for file_path in upload_dir.iterdir():
            if file_path.is_file() and file_path.suffix.lower() in settings.allowed_extensions:
                stat = file_path.stat()
                files.append({
                    "filename": file_path.name,
                    "size_bytes": stat.st_size,
                    "created_at": datetime.fromtimestamp(stat.st_ctime),
                    "modified_at": datetime.fromtimestamp(stat.st_mtime),
                    "url": f"/uploads/{file_path.name}"
                })
        
        return {
            "files": sorted(files, key=lambda x: x["created_at"], reverse=True),
            "total": len(files)
        }
        
    except Exception as e:
        logger.error(f"Error listando archivos: {e}", exc_info=True)
        raise HTTPException(status_code=500, detail=f"Error listando archivos: {str(e)}")
