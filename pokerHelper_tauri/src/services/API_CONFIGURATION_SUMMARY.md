# Configuración de PokerAPI.ts - Resumen de Cambios

## Cambios Realizados

### 1. Interfaces Actualizadas

#### `PokerHandAnalysis`
- Incluye:
  - `playerProbabilities`: Probabilidades del jugador (pair, threeOfAKind, straight, flush, fullHouse, fourOfAKind, straightFlush, royalFlush)
  - `opponentProbabilities`: Probabilidades de los oponentes (misma estructura)
  - `decision`: Objeto con acción recomendada, cantidad de apuesta y descripción
  - `timestamp`: Para tracking

#### `PokerCalculationRequest`
- Campos requeridos:
  - `numberOfOpponents`: número de oponentes (0-8)
  - `smallBlind`: apuesta ciega pequeña (mínimo 1)
  - `accumulatedBet`: apuesta acumulada (mínimo 0)

#### `BackendCard`
- Interfaz para cartas en formato backend:
  - `suit`: string ("S", "H", "D", "C")
  - `rank`: number (1-14, donde 1=Ace bajo, 11=Jack, 12=Queen, 13=King, 14=Ace alto)

### 2. Endpoints

- `/calculate` 

### 3. Métodos

#### `analyzeHand()`
- Parámetros con valores por defecto:
  - `numberOfOpponents = 3`
  - `smallBlind = 5`
  - `accumulatedBet = 0`
- Validació: requiere al menos 2 cartas en la mano del jugador

#### `makeDecision()`
- Nuevo método para obtener decisiones de poker
- Usa el endpoint `/decision`
- Retorna objeto con acción, cantidad y descripción

#### `getApiInfo()`
- Nuevo método para obtener información de la API
- Usa el endpoint `/info`

### 4. Conversión de Formatos

#### Métodos de Conversión:
- `convertSuit()`: Convierte suits del frontend ("hearts") al backend ("H")
- `convertRank()`: Convierte ranks del frontend ("A") al backend (14)
- `convertCard()`: Convierte carta individual
- `convertCards()`: Convierte array de cartas

#### Mapeo de Suits:
- "hearts" → "H"
- "diamonds" → "D"
- "clubs" → "C"
- "spades" → "S"

#### Mapeo de Ranks:
- "A" → 14 (Ace alto)
- "2"-"10" → 2-10
- "J" → 11
- "Q" → 12
- "K" → 13

### 5. Variables de Entorno

#### Frontend (.env en pokerHelper_tauri):
```
VITE_API_JAVA_HOST=localhost
VITE_API_JAVA_PORT=8080
VITE_API_JAVA_PATH=/api/poker
```

#### Backend (archivo .env raíz):
```
JAVA_API_PATH=/api/poker
API_JAVA_HOST=localhost
API_JAVA_PORT=8080
API_JAVA_PATH=/api/poker
```

## Compatibilidad

✅ **Frontend → Backend**: Completamente compatible
- Estructura de request coincide con `PokerCalculationRequest.java`
- Formato de cartas convertido automáticamente
- Endpoints correctos

✅ **Backend → Frontend**: Completamente compatible
- Estructura de response coincide con `PokerCalculationResponse.java`
- Tipos TypeScript reflejan exactamente la estructura Java

## URLs de Conexión

- **Cálculo de Probabilidades**: `POST http://localhost:8080/api/poker/calculate`
- **Decisión de Poker**: `POST http://localhost:8080/api/poker/decision`
- **Información de API**: `GET http://localhost:8080/api/poker/info`

## Ejemplo de Uso

```typescript
// Análisis de mano
const analysis = await pokerAPIService.analyzeHand(
  [{ suit: 'hearts', rank: 'A' }, { suit: 'spades', rank: 'K' }], // pocket cards
  [{ suit: 'hearts', rank: '10' }, { suit: 'hearts', rank: 'J' }], // community cards
  3, // número de oponentes
  5, // small blind
  10 // apuesta acumulada
);

// Decisión de poker
const decision = await pokerAPIService.makeDecision(
  pocketCards,
  communityCards,
  3, 5, 10
);

console.log(decision.action); // "FOLD", "CALL", "RAISE", etc.
console.log(decision.betAmount); // cantidad recomendada
console.log(decision.description); // descripción de la decisión
```
