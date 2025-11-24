"""
Configuración de la aplicación usando Pydantic Settings.
Usa variables de entorno y archivo .env para configuración flexible.
"""
from pydantic import BaseSettings, Field, validator
from typing import Optional, List
import os
from pathlib import Path

class Settings(BaseSettings):
    
    # API Configuration
    api_host: str = Field(default="127.0.0.1", env="API_HOST")
    api_port: int = Field(default=8000, env="API_PORT")
    debug: bool = Field(default=True, env="DEBUG")
    
    # Model Configuration
    model_path: Optional[str] = Field(default=None, env="MODEL_PATH")
    img_height: int = Field(default=1200, env="IMG_HEIGHT")
    img_width: int = Field(default=1920, env="IMG_WIDTH")
    
    # File Upload Configuration
    max_file_size: int = Field(default=10 * 1024 * 1024, env="MAX_FILE_SIZE")  # 10MB
    allowed_extensions: List[str] = [".jpg", ".jpeg", ".png", ".bmp", ".tiff"]
    upload_dir: str = Field(default="uploads", env="UPLOAD_DIR")
    
    # Logging Configuration
    log_level: str = Field(default="INFO", env="LOG_LEVEL")
    log_file: str = Field(default="logs/api.log", env="LOG_FILE")
    
    # Security
    secret_key: str = Field(default="dev-secret-key-change-in-production", env="SECRET_KEY")
    
    # CORS Configuration
    allowed_origins: str = Field(default="*", env="ALLOWED_ORIGINS")
    
    # Cache Configuration (Redis)
    redis_url: Optional[str] = Field(default=None, env="REDIS_URL")
    cache_ttl: int = Field(default=3600, env="CACHE_TTL")  # 1 hora
    
    # Performance Configuration
    max_workers: int = Field(default=4, env="MAX_WORKERS")
    keep_alive: int = Field(default=2, env="KEEP_ALIVE")
    
    # Database Configuration (opcional para futuro)
    database_url: Optional[str] = Field(default=None, env="DATABASE_URL")
    
    # Validadores
    @validator('log_level')
    def validate_log_level(cls, v):
        """Valida que el nivel de log sea válido."""
        valid_levels = ['DEBUG', 'INFO', 'WARNING', 'ERROR', 'CRITICAL']
        if v.upper() not in valid_levels:
            raise ValueError(f'log_level debe ser uno de: {valid_levels}')
        return v.upper()
    
    @validator('max_file_size')
    def validate_file_size(cls, v):
        """Valida que el tamaño máximo de archivo sea razonable."""
        if v < 1024:  # Mínimo 1KB
            raise ValueError('max_file_size debe ser al menos 1024 bytes')
        if v > 100 * 1024 * 1024:  # Máximo 100MB
            raise ValueError('max_file_size no puede ser mayor a 100MB')
        return v
    
    @validator('upload_dir')
    def validate_upload_dir(cls, v):
        """Crea el directorio de uploads si no existe."""
        Path(v).mkdir(parents=True, exist_ok=True)
        return v
    
    @validator('allowed_origins')
    def parse_allowed_origins(cls, v):
        """Convierte string de origins separados por coma en lista."""
        if v == "*":
            return ["*"]
        return [origin.strip() for origin in v.split(",") if origin.strip()]
    
    @property
    def host(self) -> str:
        """Alias para compatibilidad hacia atrás."""
        return self.api_host
    
    @property
    def port(self) -> int:
        """Alias para compatibilidad hacia atrás."""
        return self.api_port
    
    @property
    def cors_origins(self) -> List[str]:
        """Lista de orígenes permitidos para CORS."""
        return self.allowed_origins if isinstance(self.allowed_origins, list) else ["*"]
    
    def get_log_file_path(self) -> Path:
        """Obtiene la ruta completa del archivo de log."""
        log_path = Path(self.log_file)
        log_path.parent.mkdir(parents=True, exist_ok=True)
        return log_path
    
    class Config:
        env_file = ".env"
        env_file_encoding = "utf-8"
        case_sensitive = False
        
        # Permitir validación de campos extra en desarrollo
        extra = "ignore"

# Singleton para configuración
_settings = None

def get_settings() -> Settings:
    """Obtener instancia singleton de configuración."""
    global _settings
    if _settings is None:
        _settings = Settings()
    return _settings
