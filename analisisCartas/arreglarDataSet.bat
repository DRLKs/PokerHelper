@echo off
setlocal enabledelayedexpansion

REM Verifica si se proporcionó un archivo de texto como argumento
if "%~1"=="" (
    echo Error: Debes proporcionar un archivo de texto como argumento.
    exit /b 1
)

REM Define las palabras a reemplazar y sus sustitutos
set "picas_aleman=Piek"
set "picas_letra=P"

set "treboles_aleman=Kreuz"
set "treboles_letra=T"

set "diamantes_aleman=Karo"
set "diamantes_letra=T"

set "corazones_aleman=Herz"
set "corazones_letra=C"

set "as_aleman=Ass"
set "as_numero=1"

set "rey_aleman=Konig"
set "rey_numero=13"

set "dama_aleman=Dame"
set "dama_numero=12"

set "sota_aleman=Bube"
set "sota_numero=11"

REM Archivo de texto de entrada
set "archivo_entrada=%~1"

REM Archivo de texto de salida (temporal)
set "archivo_salida=%archivo_entrada%.tmp"

REM Reemplaza las palabras en el archivo
(
    for /f "tokens=*" %%a in (%archivo_entrada%) do (
        set "linea=%%a"
        REM PICAS
        set "linea=!linea:%picas_aleman%=%picas_letra%!"  
        REM TREBOLES          
        set "linea=!linea:%treboles_aleman%=%treboles_letra%!"
        REM DIAMANTES
        set "linea=!linea:%diamantes_aleman%=%diamantes_letra%!"
        REM CORAZONES
        set "linea=!linea:%corazones_aleman%=%corazones_letra%!"


        REM ASES
        set "linea=!linea:%as_aleman%=%as_numero%!"
        REM REYES
        set "linea=!linea:%rey_aleman%=%rey_numero%!"
        REM DAMAS
        set "linea=!linea:%dama_aleman%=%dama_numero%!"
        REM SOTAS
        set "linea=!linea:%sota_aleman%=%sota_numero%!"
        echo !linea!
    )
) > %archivo_salida%

REM Reemplaza el archivo original con el archivo modificado
move /y %archivo_salida% %archivo_entrada% >nul

echo Reemplazo completado. Las palabras han sido reemplazadas en el archivo.