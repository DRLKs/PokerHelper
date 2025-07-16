import { Card } from '../types/Card';

export interface PokerHandAnalysis {
  playerHand: Card[];
  communityCards: Card[];
  handStrength: number;
  handType: string;
  winProbability: number;
  recommendations: string[];
  timestamp: number;
}

export interface PokerRequest {
  playerHand: Card[];
  communityCards: Card[];
}

// Configuración del backend
const BACKEND_URL = 'http://localhost:8080/api/poker';

class PokerAPIService {
  private abortController: AbortController | null = null;
  private lastRequestId: number = 0;

  // Cancelar la petición anterior si existe
  private cancelPreviousRequest(): void {
    if (this.abortController) {
      this.abortController.abort();
    }
    this.abortController = new AbortController();
  }

  // Filtrar cartas válidas (que tengan suit y rank)
  private filterValidCards(cards: Card[]): Card[] {
    return cards.filter(card => card.suit && card.rank);
  }

  // Hacer petición al backend
  async analyzeHand(playerHand: Card[], communityCards: Card[]): Promise<PokerHandAnalysis | null> {
    // Cancelar petición anterior
    this.cancelPreviousRequest();
    
    const requestId = ++this.lastRequestId;
    const validPlayerHand = this.filterValidCards(playerHand);
    const validCommunityCards = this.filterValidCards(communityCards);

    // Si no hay cartas válidas, no hacer petición
    if (validPlayerHand.length === 0) {
      return null;
    }

    const requestData: PokerRequest = {
      playerHand: validPlayerHand,
      communityCards: validCommunityCards
    };

    try {
      const response = await fetch(`${BACKEND_URL}/analyze`, {
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

      const data: PokerHandAnalysis = await response.json();
      
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

  // Método para cancelar manualmente todas las peticiones
  cancelAllRequests(): void {
    if (this.abortController) {
      this.abortController.abort();
      this.abortController = null;
    }
  }
}

// Instancia singleton del servicio
export const pokerAPIService = new PokerAPIService();
