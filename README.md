# Aplicación Para Futura IA Poker

## Funcionalidad
Poker Helper pretende ser un software que juegue al Poker Texas Holdem de una forma lo más perfecta posible.
Esto lo hará devolviendo al jugador la decisión que más le convenga en cada mano. Para ello, Poker Helper necesita:
- Las cartas de tu mano.
- Las cartas que haya en la mesa.
- El número de jugadores que estén jugando en ese momento.

Toda esta información la obtendremos mediante el análisis por visión por computador de una ventana donde el jugador se encuentre en una partida de Poker Online (Solo se implementará para Poker Stars), en el caso de que el jugador lo decida también podrá introducir los datos a mano.
El dataSet utilizado para el apredizaje del modelo ha sido obtenido de [roboflow](https://universe.roboflow.com/), exactamente [dataSet](https://universe.roboflow.com/poker-nnnrc/poker-cards-nesyh/dataset/6), también lo podreis ver en el apartado de análisis de cartas.

## Posibles Usos
- Aprender a jugar a este famoso juego de cartas, memorizando cuales son las manos con las que será más probable ganar rondas.
- Introducirlo a una máquina de auto-aprendizaje para que reconozca cuando posse buenas cartas y mejore su calidad de juego.

## Motivación
Soy un apasionado del Poker, de la programación y de las matemáticas, por tanto mezclarlo todo en un aplicación de escritorio es algo que me ilusiona.


## Instalar dependencias
En el apartado de análisis de cartas, te encontrarás un archivo de texto, donde podrás ver todas las dependencias del programa

```cmd
pip install -r requirements.txt
