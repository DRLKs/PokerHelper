import tensorflow as tf
from tensorflow.keras import layers, models
from tensorflow.keras.preprocessing.image import ImageDataGenerator, load_img, img_to_array

import numpy as np
import pandas as pd

import matplotlib.pyplot as plt
import os
import re

# Configuración de rutas de los datasets
train_dir = 'train'
validation_dir = 'valid'
test_dir = 'test'

anotaciones_dir = 'anotaciones'
anotaciones_fich = '_annotations.csv'    # Debemos añadir el prefijo 'train' ó 'valid'ó 'test' para reconocer las anotaciones

# Parámetros del modelo
img_height = 1200
img_width = 1920
batch_size = 32
epochs = 10

# Preprocesamiento de datos
train_datagen = ImageDataGenerator(
    rescale=1.0 / 255,  # Normalizar los valores de píxeles a [0, 1], facilita el entrenamiento de la red neuronal.
    shear_range=0.2,  # Aumento de datos: inclinación, no creo que sea necesairo en este caso
    zoom_range=0.2,  # Aumento de datos: zoom, no creo que sea necesairo en este caso
    #horizontal_flip=True,  # Aumento de datos: volteo horizontal, no creo que sea necesairo en este caso
    fill_mode='nearest'  # Rellenar píxeles vacíos
)

validation_datagen = ImageDataGenerator(rescale=1.0 / 255)  # Solo normalización para validación y testeo
test_datagen = ImageDataGenerator(rescale=1.0 / 255)

# Generadores de datos
def custom_image_generator(image_dir, anotacionesCSV):
    image_paths = [os.path.join(image_dir, fname) for fname in os.listdir(image_dir)]
    num_images = len(image_paths)
    
    for i in range(0, num_images, batch_size):
        batch_paths = image_paths[i:i + batch_size]
        batch_images = []
        batch_labels = []
        for path in batch_paths:
                
            filename = os.path.basename(path)  # Obtener solo el nombre del archivo
            resultado = anotacionesCSV[ anotacionesCSV['filename'] == filename ]  
            label = resultado['cl1'].toList()
            target_size = ( img_height , img_width )

            img = load_img(path, target_size=target_size)
            img = img_to_array(img) / 255.0  # Normaliza los valores de los píxeles al rango [0, 1].
            batch_images.append(img)
            batch_labels.append(label)
        yield np.array(batch_images), np.array(batch_labels), batch_paths


anotacionesCSV = pd.read_csv(anotaciones_dir + "\train" + anotaciones_fich )           
train_generator = custom_image_generator(train_dir, anotacionesCSV)

anotacionesCSV = pd.read_csv(anotaciones_dir + "\valid" + anotaciones_fich )           
validation_generator = custom_image_generator(validation_dir, anotacionesCSV)

anotacionesCSV = pd.read_csv(anotaciones_dir + "\test" + anotaciones_fich )           
test_generator = custom_image_generator(test_dir, anotacionesCSV)


''' 
train_generator = train_datagen.flow_from_directory(
    train_dir,
    target_size=(img_height, img_width),
    batch_size=batch_size,
    class_mode='categorical'  # Cambia a 'binary' si es un problema de clasificación binaria
)

train_generator = train_datagen.flow_from_directory(
    train_dir,
    target_size=(img_height, img_width),
    batch_size=batch_size,
    class_mode='categorical'  # Cambia a 'binary' si es un problema de clasificación binaria
)

test_generator = test_datagen.flow_from_directory(
    test_dir,
    target_size=(img_height, img_width),
    batch_size=batch_size,
    class_mode='categorical',  # Cambia a 'binary' si es un problema de clasificación binaria
    shuffle=False  # No mezclar para evaluar correctamente
)
'''
# Definir el modelo CNN
model = models.Sequential([
    layers.Conv2D(32, (3, 3), activation='relu', input_shape=(img_height, img_width, 3)),
    layers.MaxPooling2D((2, 2)),
    layers.Conv2D(64, (3, 3), activation='relu'),
    layers.MaxPooling2D((2, 2)),
    layers.Conv2D(128, (3, 3), activation='relu'),
    layers.MaxPooling2D((2, 2)),
    layers.Flatten(),
    layers.Dense(512, activation='relu'),
    layers.Dropout(0.5),  # Regularización para evitar sobreajuste
    layers.Dense(train_generator.num_classes, activation='softmax')  # Capa de salida
])

# Compilar el modelo
model.compile(
    optimizer='adam',
    loss='categorical_crossentropy',  # Cambia a 'binary_crossentropy' si es clasificación binaria
    metrics=['accuracy']
)

# Entrenar el modelo
history = model.fit(
    train_generator,
    steps_per_epoch=train_generator.samples // batch_size,
    validation_data=validation_generator,
    validation_steps=validation_generator.samples // batch_size,
    epochs=epochs
)

# Evaluar el modelo con el dataset de testeo
test_loss, test_accuracy = model.evaluate(test_generator)
print(f"Precisión en el dataset de testeo: {test_accuracy * 100:.2f}%")

## GUARDAR MODELOS CON CONTROL DE VERSIONES

# Directorio donde se guardan los modelos
modelo_dir = "modelos"

# Crear la carpeta si no existe
if not os.path.exists(modelo_dir):
    os.makedirs(modelo_dir)

# Buscar la última versión del modelo
modelos_existentes = [f for f in os.listdir(modelo_dir) if re.match(r'model(\d+)\.(\d+)\.h5', f)]

if modelos_existentes:
    # Extraer versiones y encontrar la última
    versiones = sorted([tuple(map(int, re.findall(r'(\d+)\.(\d+)', m)[0])) for m in modelos_existentes])
    ultima_version = versiones[-1]
    nueva_version = (ultima_version[0], ultima_version[1] + 1)  # Incrementar subversión
else:
    nueva_version = (1, 0)  # Primera versión si no hay modelos

# Nombre del nuevo modelo
nuevo_modelo_nombre = f"model{nueva_version[0]}.{nueva_version[1]}.h5"
ruta_modelo = os.path.join(modelo_dir, nuevo_modelo_nombre)

# Suponiendo que tienes un modelo entrenado llamado 'model'
model.save(ruta_modelo)

print(f"Modelo guardado como: {ruta_modelo}")

# Graficar la precisión y la pérdida durante el entrenamiento
plt.figure(figsize=(12, 4))

# Gráfico de precisión
plt.subplot(1, 2, 1)
plt.plot(history.history['accuracy'], label='Precisión en entrenamiento')
plt.plot(history.history['val_accuracy'], label='Precisión en validación')
plt.title('Precisión durante el entrenamiento')
plt.xlabel('Época')
plt.ylabel('Precisión')
plt.legend()

# Gráfico de pérdida
plt.subplot(1, 2, 2)
plt.plot(history.history['loss'], label='Pérdida en entrenamiento')
plt.plot(history.history['val_loss'], label='Pérdida en validación')
plt.title('Pérdida durante el entrenamiento')
plt.xlabel('Época')
plt.ylabel('Pérdida')
plt.legend()

plt.show()

