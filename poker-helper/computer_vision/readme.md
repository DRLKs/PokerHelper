# Servicio de Detección de Estado del Juego - Python

## Funcionalidades
- Número de jugadores
- Cartas del usuario
- Cartas en la mesa

## Posibles implementaciones futuras
- Apuestas
- Ciegas pequeñas y grandes

## Configuración del Entorno

Las dependencias están instaladas localmente en `poker_env`

### Crear el entorno virtual

#### En Linux/macOS:
```bash
# Crear entorno virtual
python3 -m venv poker_env

# Activar el entorno virtual
source poker_env/bin/activate

# Actualizar pip y setuptools (IMPORTANTE para Python 3.13)
pip install --upgrade pip setuptools wheel

# Instalar dependencias
pip install -r requirements.txt 
```

#### En Arch Linux (instrucciones específicas):
```bash
# Instalar dependencias del sistema primero
sudo pacman -S python python-pip python-virtualenv tk

# Para compilar algunos paquetes (si es necesario)
sudo pacman -S base-devel gcc

# Crear entorno virtual
python -m venv poker_env

# Activar el entorno virtual
source poker_env/bin/activate

# Actualizar herramientas de build
pip install --upgrade pip setuptools wheel

# Opción 1: Instalar desde requirements.txt
pip install -r requirements.txt

# Opción 2: Si falla, instalar una por una para evitar conflictos
pip install numpy==1.24.3
pip install pillow>=10.3.0
pip install matplotlib==3.8.2
pip install pandas==2.2.3
pip install opencv-python>=4.8.0
pip install tensorflow==2.14.0
pip install pygetwindow==0.0.9
```

#### En Windows (Command Prompt):
```cmd
# Crear entorno virtual
python -m venv poker_env

# Activar el entorno virtual
poker_env\Scripts\activate

# Actualizar pip y setuptools (IMPORTANTE para Python 3.13)
pip install --upgrade pip setuptools wheel

# Instalar dependencias
pip install -r requirements.txt
```

#### En Windows (PowerShell):
```powershell
# Crear entorno virtual
python -m venv poker_env

# Activar el entorno virtual
poker_env\Scripts\Activate.ps1

# Actualizar pip y setuptools (IMPORTANTE para Python 3.13)
pip install --upgrade pip setuptools wheel

# Instalar dependencias
pip install -r requirements.txt
```

### Desactivar el entorno virtual
```bash
# En cualquier sistema operativo
deactivate
```

### Solución de problemas comunes

#### Error con setuptools en Python 3.13:
```bash
# Si aparece error de 'setuptools.build_meta'
pip install --upgrade pip setuptools wheel
pip install --force-reinstall setuptools
```

#### Errores específicos de Arch Linux:

**Error con numpy/tensorflow:**
```bash
# Si falla la compilación de numpy o tensorflow
sudo pacman -S python-numpy python-tensorflow
# Luego instalar en el entorno virtual
pip install --no-deps tensorflow==2.14.0
```

**Error con OpenCV:**
```bash
# Si falla opencv-python
sudo pacman -S opencv python-opencv
pip install --no-deps opencv-python>=4.8.0
```

**Error con tkinter:**
```bash
# tkinter ya viene con Python en Arch, pero si falta:
sudo pacman -S tk
```

**Problemas generales de compilación:**
```bash
# Instalar todas las herramientas de desarrollo
sudo pacman -S base-devel python-devel

# Si tienes problemas con wheels
pip install --only-binary=all -r requirements.txt
```

#### Alternativa con versiones de sistema (Arch Linux):
```bash
# Opción alternativa: usar paquetes del sistema
sudo pacman -S python-numpy python-pillow python-matplotlib python-pandas python-tensorflow python-opencv

# Crear entorno virtual sin estas dependencias
python -m venv --system-site-packages poker_env
source poker_env/bin/activate

# Solo instalar lo que no está en el sistema
pip install pygetwindow==0.0.9
```

## Estructura del Proyecto
```
services/python/
├── predictor/
│   ├── image_preprocessor.py
│   └── visualice_preprocessor/
│       └── visualice_preprocessor.py
├── poker_env/          # Entorno virtual
└── readme.md
```

## Uso

### Visualizador de Preprocesamiento
```bash
# Activar entorno virtual primero
python predictor/visualice_preprocessor/visualice_preprocessor.py
```

## Notas
- Asegúrate de tener Python 3.7+ instalado
- En Windows, si tienes problemas con PowerShell, usa Command Prompt (cmd)
- El entorno virtual debe activarse cada vez que trabajes en el proyecto
