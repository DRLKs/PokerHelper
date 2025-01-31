import tensorflow as tf
from tensorflow.keras import layers, models
from tensorflow.keras.preprocessing.image import ImageDataGenerator
import matplotlib.pyplot as plt

# Configuración de rutas de los datasets
train_dir = 'train'
validation_dir = 'valid'
test_dir = 'test'

# Parámetros del modelo
img_height, img_width = 150, 150  # Tamaño de las imágenes
batch_size = 32
epochs = 10

# Preprocesamiento de datos
train_datagen = ImageDataGenerator(
    rescale=1.0 / 255,  # Normalizar los valores de píxeles a [0, 1]
    rotation_range=20,  # Aumento de datos: rotación aleatoria
    width_shift_range=0.2,  # Aumento de datos: desplazamiento horizontal
    height_shift_range=0.2,  # Aumento de datos: desplazamiento vertical
    shear_range=0.2,  # Aumento de datos: inclinación
    zoom_range=0.2,  # Aumento de datos: zoom
    horizontal_flip=True,  # Aumento de datos: volteo horizontal
    fill_mode='nearest'  # Rellenar píxeles vacíos
)

validation_datagen = ImageDataGenerator(rescale=1.0 / 255)  # Solo normalización para validación y testeo
test_datagen = ImageDataGenerator(rescale=1.0 / 255)

# Generadores de datos
train_generator = train_datagen.flow_from_directory(
    train_dir,
    target_size=(img_height, img_width),
    batch_size=batch_size,
    class_mode='categorical'  # Cambia a 'binary' si es un problema de clasificación binaria
)

validation_generator = validation_datagen.flow_from_directory(
    validation_dir,
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

# Guardar el modelo entrenado
model.save('modelo_entrenado.h5')
print("Modelo guardado como 'modelo_entrenado.h5'")

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