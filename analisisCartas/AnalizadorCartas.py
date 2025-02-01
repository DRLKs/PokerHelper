import tkinter as tk
from tkinter import messagebox
import pygetwindow as gw
from PIL import ImageGrab
import os
import time
import numpy as np

# Función para seleccionar una ventana
def seleccionar_ventana():
    # Obtener todas las ventanas abiertas
    ventanas = gw.getAllTitles()

    # Filtrar ventanas con título (ignorar ventanas sin título)
    ventanas = [ventana for ventana in ventanas if ventana]
    # Filtramos las ventanas, con las ventanas de PokerStars
    mesas_poker = [v for v in ventanas if "PokerStars" in v]

    if not mesas_poker:
        messagebox.showerror("Error", "No se encontraron ventanas abiertas.")
        return None

    # Crear una ventana de tkinter
    root = tk.Tk()
    root.title("Seleccionar ventana")

    # Variable para almacenar la ventana seleccionada
    ventana_seleccionada = tk.StringVar(root)
    ventana_seleccionada.set(mesas_poker[0])  # Valor por defecto

    # Crear un menú desplegable con las ventanas
    label = tk.Label(root, text="Seleccione la ventana que desea analizar:")
    label.pack(pady=10)

    opciones = tk.OptionMenu(root, ventana_seleccionada, *mesas_poker)
    opciones.pack(pady=10)

    # Función para confirmar la selección
    def confirmar():
        root.quit()

    boton_confirmar = tk.Button(root, text="Confirmar", command=confirmar)
    boton_confirmar.pack(pady=10)

    # Mostrar la ventana y esperar a que el usuario seleccione una opción
    root.mainloop()

    # Obtener la ventana seleccionada
    seleccion = ventana_seleccionada.get()
    root.destroy()  # Cerrar la ventana de tkinter

    # Obtener el objeto de la ventana seleccionada
    ventana = gw.getWindowsWithTitle(seleccion)[0]
    return ventana

# Función para capturar la ventana seleccionada
def capturar_ventana(ventana):
    if ventana:
        try:
            # Activar la ventana (traerla al frente y restaurarla si está minimizada)
            ventana.activate()
            time.sleep(0.5)  # Esperar un momento para que la ventana se active

            # Obtener las coordenadas de la ventana
            left = ventana.left
            top = ventana.top
            right = ventana.right
            bottom = ventana.bottom

            # Capturar la región de la ventana
            captura = ImageGrab.grab(bbox=(left, top, right, bottom))

            ### ESTO LO USAMOS AHORA PARA PROBAR EL PROGRAMITA 

            # Crear la carpeta si no existe
            carpeta = "capturas"
            if not os.path.exists(carpeta):
                os.makedirs(carpeta)
            # Guardar la captura (Lo borraremos en un futuro)
            # Nombre del archivo con marca de tiempo
            nombre_archivo = f"captura_{int(time.time())}.png"
            # Ruta completa del archivo
            ruta_completa = os.path.join(carpeta, nombre_archivo)
            captura.save(ruta_completa)

            ### PRUEBAS PROGRAMITA

            ################# DEBEMOS ANALIZAR LA CAPTURA CON EL MODELO AQUÍ


            # Convertir la imagen a un array de numpy para procesarla
            captura_array = np.array(captura)

            return captura_array
        except Exception as e:
            print(f"Error al capturar la ventana: {e}")
    else:
        print("No se ha seleccionado ninguna ventana.")
    return None

# Función principal
def main(intervalo=5):
    # Seleccionar la ventana
    ventana = seleccionar_ventana()

    if ventana:
        print(f"Iniciando capturas de la ventana '{ventana.title}' cada {intervalo} segundos...")
        try:
            while True:
                # Capturar la ventana
                captura = capturar_ventana(ventana)

                if captura is not None:
                    # Aquí puedes procesar la captura con tu modelo
                    print("Captura realizada. Procesando con el modelo...")
                    # TODO: Integrar tu modelo aquí

                # Esperar el intervalo de tiempo
                time.sleep(intervalo)
        except KeyboardInterrupt:
            print("Capturas detenidas.")
    else:
        print("No se pudo iniciar la captura.")

if __name__ == "__main__":
    # Ejecutar la aplicación con un intervalo de 5 segundos
    main(intervalo=5)