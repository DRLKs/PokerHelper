package com.pokerhelper.domain.model;

/**
 * Domain model for poker probabilities
 */
public class PokerProbabilities {
    
    private final PlayerProbabilities playerProbabilities;
    private final PlayerProbabilities opponentProbabilities;
    private final Decision decision;

    public PokerProbabilities(PlayerProbabilities playerProbabilities, 
                            PlayerProbabilities opponentProbabilities, 
                            Decision decision) {
        this.playerProbabilities = playerProbabilities;
        this.opponentProbabilities = opponentProbabilities;
        this.decision = decision;
    }

    public PlayerProbabilities getPlayerProbabilities() {
        return playerProbabilities;
    }

    public PlayerProbabilities getOpponentProbabilities() {
        return opponentProbabilities;
    }

    public Decision getDecision() {
        return decision;
    }

    public static class PlayerProbabilities {
        private final double pair;
        private final double threeOfAKind;
        private final double straight;
        private final double flush;
        private final double fullHouse;
        private final double fourOfAKind;
        private final double straightFlush;
        private final double royalFlush;

        public PlayerProbabilities(double pair, double threeOfAKind, double straight, 
                                 double flush, double fullHouse, double fourOfAKind, 
                                 double straightFlush, double royalFlush) {
            this.pair = pair;
            this.threeOfAKind = threeOfAKind;
            this.straight = straight;
            this.flush = flush;
            this.fullHouse = fullHouse;
            this.fourOfAKind = fourOfAKind;
            this.straightFlush = straightFlush;
            this.royalFlush = royalFlush;
        }

        public double getPair() { return pair; }
        public double getThreeOfAKind() { return threeOfAKind; }
        public double getStraight() { return straight; }
        public double getFlush() { return flush; }
        public double getFullHouse() { return fullHouse; }
        public double getFourOfAKind() { return fourOfAKind; }
        public double getStraightFlush() { return straightFlush; }
        public double getRoyalFlush() { return royalFlush; }
    }
}
