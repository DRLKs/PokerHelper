export type Language = 'es' | 'en' | 'fr';

export interface Translations {
  appTitle: string;
  playerHandTitle: string;
  communityCardsTitle: string;
  selectCard: string;
  clear: string;
  suits: {
    hearts: string;
    diamonds: string;
    clubs: string;
    spades: string;
  };
  language: string;
  analysis: {
    title: string;
    loading: string;
    error: string;
    selectCards: string;
    handType: string;
    strength: string;
    winProbability: string;
    recommendations: string;
    updated: string;
  };
}
