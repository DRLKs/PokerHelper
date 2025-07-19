import { Card } from '../types/Card';

// Tipo para cartas en el formato del backend
export interface BackendCard {
  suit: string;
  rank: number;
}

export interface PokerHandAnalysis {
  playerProbabilities: {
    pair: number;
    threeOfAKind: number;
    straight: number;
    flush: number;
    fullHouse: number;
    fourOfAKind: number;
    straightFlush: number;
    royalFlush: number;
  };
  opponentProbabilities: {
    pair: number;
    threeOfAKind: number;
    straight: number;
    flush: number;
    fullHouse: number;
    fourOfAKind: number;
    straightFlush: number;
    royalFlush: number;
  };
  decision: {
    action: string;
    betAmount: number;
    description: string;
  };
  timestamp: number;
}

export interface PokerCalculationRequest {
  pocketCards: BackendCard[];
  communityCards: BackendCard[];
  numberOfOpponents: number;
  smallBlind: number;
  accumulatedBet: number;
}

// Configuración del backend
const API_HOST = import.meta.env.VITE_API_JAVA_HOST || 'localhost';
const API_PORT = import.meta.env.VITE_API_JAVA_PORT || '8080';
const API_PATH = import.meta.env.VITE_API_JAVA_PATH || '/api/poker';
const BACKEND_URL = `http://${API_HOST}:${API_PORT}${API_PATH}`;

console.log('PokerAPI Configuration:', {
  API_HOST,
  API_PORT,
  API_PATH,
  BACKEND_URL,
  fullCalculateURL: `${BACKEND_URL}/calculate`,
  fullInfoURL: `${BACKEND_URL}/info`
});

class PokerAPIService {
  private abortController: AbortController | null = null;
  private lastRequestId: number = 0;

  // Convertir suit del frontend al formato del backend
  private convertSuit(suit: string): string {
    const suitMap: { [key: string]: string } = {
      'hearts': 'H',
      'diamonds': 'D',
      'clubs': 'C',
      'spades': 'S'
    };
    return suitMap[suit] || suit;
  }

  // Convertir rank del frontend al formato del backend
  private convertRank(rank: string): number {
    const rankMap: { [key: string]: number } = {
      'A': 14, // Ace high
      '2': 2, '3': 3, '4': 4, '5': 5, '6': 6, '7': 7, '8': 8, '9': 9, '10': 10,
      'J': 11, 'Q': 12, 'K': 13
    };
    return rankMap[rank] || parseInt(rank) || 0;
  }

  // Convertir carta del frontend al formato del backend
  private convertCard(card: Card): BackendCard {
    const converted = {
      suit: this.convertSuit(card.suit),
      rank: this.convertRank(card.rank)
    };
    console.log('Converting card:', { input: card, output: converted });
    return converted;
  }

  // Convertir array de cartas del frontend al backend
  private convertCards(cards: Card[]): BackendCard[] {
    return cards.map(card => this.convertCard(card));
  }

  // Cancelar la petición anterior si existe
  private cancelPreviousRequest(): void {
    if (this.abortController) {
      this.abortController.abort();
    }
    this.abortController = new AbortController();
  }

  // Filtrar cartas válidas (que tengan suit y rank)
  private filterValidCards(cards: Card[]): Card[] {
    const filtered = cards.filter(card => card.suit && card.rank && card.suit.trim() !== '' && card.rank.trim() !== '');
    console.log('Filtering cards:', { input: cards, filtered });
    return filtered;
  }

  // Hacer petición al backend para calcular probabilidades
  async analyzeHand(
    playerHand: Card[], 
    communityCards: Card[], 
    numberOfOpponents: number = 3,
    smallBlind: number = 5,
    accumulatedBet: number = 0
  ): Promise<PokerHandAnalysis | null> {
    // Cancelar petición anterior
    this.cancelPreviousRequest();
    
    const requestId = ++this.lastRequestId;
    const validPocketCards = this.filterValidCards(playerHand);
    const validCommunityCards = this.filterValidCards(communityCards);

    console.log('AnalyzeHand called with:', { 
      playerHand, 
      communityCards, 
      validPocketCards, 
      validCommunityCards,
      numberOfOpponents,
      smallBlind,
      accumulatedBet
    });

    // Validar que tenemos exactamente 2 cartas para la mano del jugador
    if (validPocketCards.length < 2) {
      console.log('Not enough valid pocket cards:', validPocketCards.length, 'required: 2');
      return null;
    }

    // Tomar solo las primeras 2 cartas válidas (el backend requiere exactamente 2)
    const pocketCardsToSend = validPocketCards.slice(0, 2);
    
    // Asegurar valores mínimos válidos
    const safeNumberOfOpponents = Math.max(1, Math.min(9, numberOfOpponents)); // Entre 1 y 9
    const safeSmallBlind = Math.max(1, smallBlind); // Mínimo 1
    const safeAccumulatedBet = Math.max(0, accumulatedBet); // No negativo

    console.log('Validated parameters:', {
      pocketCardsCount: pocketCardsToSend.length,
      communityCardsCount: validCommunityCards.length,
      numberOfOpponents: safeNumberOfOpponents,
      smallBlind: safeSmallBlind,
      accumulatedBet: safeAccumulatedBet
    });

    const requestData: PokerCalculationRequest = {
      pocketCards: this.convertCards(pocketCardsToSend),
      communityCards: this.convertCards(validCommunityCards),
      numberOfOpponents: safeNumberOfOpponents,
      smallBlind: safeSmallBlind,
      accumulatedBet: safeAccumulatedBet
    };

    console.log('Sending request to:', `${BACKEND_URL}/calculate`);
    console.log('Request data:', requestData);

    try {
      const response = await fetch(`${BACKEND_URL}/calculate`, {
        method: 'POST',
        headers: {
          'Content-Type': 'application/json',
        },
        body: JSON.stringify(requestData),
        signal: this.abortController?.signal,
      });

      console.log('Response status:', response.status, response.statusText);

      // Verificar si la petición fue cancelada
      if (this.abortController?.signal.aborted) {
        throw new Error('Request was aborted');
      }

      // Verificar si esta respuesta corresponde a la última petición
      if (requestId !== this.lastRequestId) {
        throw new Error('Response from outdated request');
      }

      if (!response.ok) {
        // Intentar obtener más información del error
        let errorDetails;
        try {
          const errorText = await response.text();
          console.error('Server error response:', errorText);
          errorDetails = errorText;
        } catch (e) {
          console.error('Could not read error response body');
          errorDetails = `HTTP ${response.status} - ${response.statusText}`;
        }
        throw new Error(`HTTP error! status: ${response.status}, details: ${errorDetails}`);
      }

      const data: PokerHandAnalysis = await response.json();
      console.log('Successfully received analysis:', data);
      
      // Agregar timestamp para tracking
      data.timestamp = Date.now();
      
      return data;
    } catch (error) {
      if (error instanceof Error) {
        if (error.name === 'AbortError' || error.message === 'Request was aborted') {
          console.log('Request was cancelled');
          return null;
        }
        if (error.message === 'Response from outdated request') {
          console.log('Ignoring outdated response');
          return null;
        }
      }
      
      console.error('Error analyzing hand:', error);
      throw error;
    }
  }

  // Hacer petición para obtener decisión de poker
  async makeDecision(
    playerHand: Card[], 
    communityCards: Card[], 
    numberOfOpponents: number = 3,
    smallBlind: number = 5,
    accumulatedBet: number = 0
  ): Promise<{action: string; betAmount: number; description: string} | null> {
    // Cancelar petición anterior
    this.cancelPreviousRequest();
    
    const requestId = ++this.lastRequestId;
    const validPocketCards = this.filterValidCards(playerHand);
    const validCommunityCards = this.filterValidCards(communityCards);

    console.log('MakeDecision called with:', { 
      playerHand, 
      communityCards, 
      validPocketCards, 
      validCommunityCards,
      numberOfOpponents,
      smallBlind,
      accumulatedBet
    });

    // Validar que tenemos exactamente 2 cartas para la mano del jugador
    if (validPocketCards.length < 2) {
      console.log('Not enough valid pocket cards for decision:', validPocketCards.length, 'required: 2');
      return null;
    }

    // Tomar solo las primeras 2 cartas válidas
    const pocketCardsToSend = validPocketCards.slice(0, 2);
    
    // Asegurar valores mínimos válidos
    const safeNumberOfOpponents = Math.max(1, Math.min(9, numberOfOpponents));
    const safeSmallBlind = Math.max(1, smallBlind);
    const safeAccumulatedBet = Math.max(0, accumulatedBet);

    const requestData: PokerCalculationRequest = {
      pocketCards: this.convertCards(pocketCardsToSend),
      communityCards: this.convertCards(validCommunityCards),
      numberOfOpponents: safeNumberOfOpponents,
      smallBlind: safeSmallBlind,
      accumulatedBet: safeAccumulatedBet
    };

    try {
      const response = await fetch(`${BACKEND_URL}/decision`, {
        method: 'POST',
        headers: {
          'Content-Type': 'application/json',
        },
        body: JSON.stringify(requestData),
        signal: this.abortController?.signal,
      });

      // Verificar si la petición fue cancelada
      if (this.abortController?.signal.aborted) {
        throw new Error('Request was aborted');
      }

      // Verificar si esta respuesta corresponde a la última petición
      if (requestId !== this.lastRequestId) {
        throw new Error('Response from outdated request');
      }

      if (!response.ok) {
        throw new Error(`HTTP error! status: ${response.status}`);
      }

      const data = await response.json();
      return data;
    } catch (error) {
      if (error instanceof Error) {
        if (error.name === 'AbortError' || error.message === 'Request was aborted') {
          console.log('Decision request was cancelled');
          return null;
        }
        if (error.message === 'Response from outdated request') {
          console.log('Ignoring outdated decision response');
          return null;
        }
      }
      
      console.error('Error making poker decision:', error);
      throw error;
    }
  }

  // Obtener información de la API
  async getApiInfo(): Promise<{name: string; version: string; description: string; endpoints: string[]} | null> {
    try {
      const response = await fetch(`${BACKEND_URL}/info`, {
        method: 'GET',
        headers: {
          'Content-Type': 'application/json',
        },
      });

      if (!response.ok) {
        throw new Error(`HTTP error! status: ${response.status}`);
      }

      const data = await response.json();
      return data;
    } catch (error) {
      console.error('Error getting API info:', error);
      throw error;
    }
  }

  // Método para cancelar manualmente todas las peticiones
  cancelAllRequests(): void {
    if (this.abortController) {
      this.abortController.abort();
      this.abortController = null;
    }
  }

  // Método de debug para testear la conexión
  async testConnection(): Promise<{ success: boolean; message: string; details?: any }> {
    console.log('Testing connection to:', BACKEND_URL);
    
    try {
      // Primero probar el health endpoint
      const healthResponse = await fetch('http://localhost:8080/health', {
        method: 'GET',
        headers: { 'Content-Type': 'application/json' },
      });

      if (!healthResponse.ok) {
        return {
          success: false,
          message: `Health check failed with status: ${healthResponse.status}`,
          details: { status: healthResponse.status, statusText: healthResponse.statusText }
        };
      }

      const healthData = await healthResponse.json();
      console.log('Health check passed:', healthData);

      // Luego probar el info endpoint
      const infoResponse = await fetch(`${BACKEND_URL}/info`, {
        method: 'GET',
        headers: { 'Content-Type': 'application/json' },
      });

      if (!infoResponse.ok) {
        return {
          success: false,
          message: `API info failed with status: ${infoResponse.status}`,
          details: { status: infoResponse.status, statusText: infoResponse.statusText }
        };
      }

      const infoData = await infoResponse.json();
      console.log('API info retrieved:', infoData);

      // Finalmente, probar el endpoint de cálculo con datos de ejemplo
      const testCalculationData: PokerCalculationRequest = {
        pocketCards: [
          { suit: 'H', rank: 14 }, // Ace of Hearts
          { suit: 'S', rank: 13 }  // King of Spades
        ],
        communityCards: [],
        numberOfOpponents: 3,
        smallBlind: 5,
        accumulatedBet: 0
      };

      console.log('Testing calculation endpoint with:', testCalculationData);
      
      const calcResponse = await fetch(`${BACKEND_URL}/calculate`, {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify(testCalculationData),
      });

      if (!calcResponse.ok) {
        let errorDetails;
        try {
          const errorText = await calcResponse.text();
          errorDetails = errorText;
        } catch (e) {
          errorDetails = `HTTP ${calcResponse.status} - ${calcResponse.statusText}`;
        }
        
        return {
          success: false,
          message: `Calculation test failed with status: ${calcResponse.status}`,
          details: { 
            health: healthData, 
            info: infoData,
            calculationError: errorDetails
          }
        };
      }

      const calcData = await calcResponse.json();
      console.log('Calculation test passed:', calcData);

      return {
        success: true,
        message: 'All connection tests successful',
        details: { 
          health: healthData, 
          info: infoData,
          calculationTest: calcData
        }
      };

    } catch (error) {
      const errorMessage = error instanceof Error ? error.message : 'Unknown error';
      console.error('Connection test failed:', error);
      
      return {
        success: false,
        message: `Connection failed: ${errorMessage}`,
        details: { error }
      };
    }
  }
}

// Instancia singleton del servicio
export const pokerAPIService = new PokerAPIService();
