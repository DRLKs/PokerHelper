import { Translations, Language } from '../types/Language';

export const translations: Record<Language, Translations> = {
  es: {
    appTitle: 'Poker Helper',
    playerHandTitle: 'Tu mano (2 cartas)',
    communityCardsTitle: 'Selecciona las cartas comunitarias (hasta 5)',
    selectCard: 'Seleccionar Carta',
    clear: 'Limpiar',
    suits: {
      hearts: 'Corazones',
      diamonds: 'Diamantes',
      clubs: 'Tréboles',
      spades: 'Picas'
    },
    analysis: {
      title: 'Análisis de Mano',
      loading: 'Calculando probabilidades...',
      error: 'Error en el análisis',
      selectCards: 'Selecciona al menos tus 2 cartas para ver el análisis',
      handType: 'Tipo de Mano',
      strength: 'Fuerza',
      winProbability: 'Probabilidad de Victoria',
      recommendations: 'Recomendaciones',
      updated: 'Actualizado'
    }
  },
  en: {
    appTitle: 'Poker Helper',
    playerHandTitle: 'Your hand (2 cards)',
    communityCardsTitle: 'Select community cards (up to 5)',
    selectCard: 'Select Card',
    clear: 'Clear',
    suits: {
      hearts: 'Hearts',
      diamonds: 'Diamonds',
      clubs: 'Clubs',
      spades: 'Spades'
    },
    analysis: {
      title: 'Hand Analysis',
      loading: 'Calculating probabilities...',
      error: 'Analysis error',
      selectCards: 'Select at least your 2 cards to see analysis',
      handType: 'Hand Type',
      strength: 'Strength',
      winProbability: 'Win Probability',
      recommendations: 'Recommendations',
      updated: 'Updated'
    }
  },
  fr: {
    appTitle: 'Aide Poker',
    playerHandTitle: 'Votre main (2 cartes)',
    communityCardsTitle: 'Sélectionner les cartes communes (jusqu\'à 5)',
    selectCard: 'Sélectionner une Carte',
    clear: 'Effacer',
    suits: {
      hearts: 'Cœurs',
      diamonds: 'Carreaux',
      clubs: 'Trèfles',
      spades: 'Piques'
    },
    analysis: {
      title: 'Analyse de Main',
      loading: 'Calcul des probabilités...',
      error: 'Erreur d\'analyse',
      selectCards: 'Sélectionnez au moins vos 2 cartes pour voir l\'analyse',
      handType: 'Type de Main',
      strength: 'Force',
      winProbability: 'Probabilité de Victoire',
      recommendations: 'Recommandations',
      updated: 'Mis à jour'
    }
  }
};
