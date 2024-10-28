import sys
import tensorflow as tf
from tensorflow.keras import layers 

## Modelo DQN
## Modelo de Refuerzo Profundo ->
##      Cuando falle -> Recibirá una respuesta negativa en función DineroPerdido / CiegaGrande
##      Cuando acierte -> Recivirá una respuesta positiva en función DineroGanado / CiegaGrande



## Variables globales que usaremos
probEscalera
probColor
probFullHouse
probPoker
çprobEscaleraColor
probEscaleraReal

if __name__ == "__main__":
    # Toma los argumentos y conviértelos a float
    probEscalera = float(sys.argv[1]);
	probColor = float(sys.argv[2]);
	probFullHouse = float(sys.argv[3]);
	probPoker = float(sys.argv[4]);
	probEscaleraColor = float(sys.argv[5]);
	probEscaleraReal = float(sys.argv[6]);
    
