
import time
import pygetwindow as gw
import pyautogui
import pytesseract
import cv2
import numpy as np
from PIL import Image



def list_windows():
    """Lista las ventanas disponibles en el sistema."""
    windows = gw.getAllTitles()
    return [win for win in windows if win.strip()]  # Filtrar ventanas sin título

def capture_window(window_title):
    """Captura una ventana específica por título."""
    try:
        # Obtener la ventana
        window = gw.getWindowsWithTitle(window_title)[0]
        # Obtener las coordenadas de la ventana
        left, top, right, bottom = window.left, window.top, window.right, window.bottom
        # Capturar la pantalla de la región especificada
        screenshot = pyautogui.screenshot(region=(left, top, right - left, bottom - top))
        return np.array(screenshot)  # Convertir a array de NumPy para OpenCV
    except IndexError:
        print("No se encontró la ventana especificada.")
        return None

def analyze_image(image):
    """Analiza la imagen y extrae texto usando Tesseract OCR."""
    # Convertir a escala de grises
    gray = cv2.cvtColor(image, cv2.COLOR_BGR2GRAY)
    # Aplicar OCR
    text = pytesseract.image_to_string(gray, lang='eng')
    return text

def main():
    # Listar ventanas
    windows = list_windows()
    if not windows:
        print("No se encontraron ventanas disponibles.")
        return

    print("Ventanas disponibles:")
    for i, win in enumerate(windows):
        print(f"{i + 1}. {win}")

    # Seleccionar una ventana
    choice = int(input("Selecciona el número de la ventana: ")) - 1
    if choice < 0 or choice >= len(windows):
        print("Selección inválida.")
        return

    selected_window = windows[choice]
    print(f"Ventana seleccionada: {selected_window}")

    # Capturar y analizar la ventana cada 2 segundos
    try:
        while True:
            image = capture_window(selected_window)
            if image is not None:
                text = analyze_image(image)
                print(f"Texto extraído:\n{text}")
            else:
                print("Error al capturar la ventana.")
            time.sleep(2)
    except KeyboardInterrupt:
        print("Ejecución terminada.")

if __name__ == "__main__":
    main()
