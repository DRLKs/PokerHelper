package com.pokerhelper.infrastructure.dto;

import com.fasterxml.jackson.annotation.JsonCreator;
import com.fasterxml.jackson.annotation.JsonProperty;

/**
 * DTO for poker calculation response
 */
public class PokerCalculationResponse {

    private final ProbabilitiesDto playerProbabilities;
    private final ProbabilitiesDto opponentProbabilities;
    private final DecisionDto decision;

    @JsonCreator
    public PokerCalculationResponse(
            @JsonProperty("playerProbabilities") ProbabilitiesDto playerProbabilities,
            @JsonProperty("opponentProbabilities") ProbabilitiesDto opponentProbabilities,
            @JsonProperty("decision") DecisionDto decision) {
        this.playerProbabilities = playerProbabilities;
        this.opponentProbabilities = opponentProbabilities;
        this.decision = decision;
    }

    public ProbabilitiesDto getPlayerProbabilities() {
        return playerProbabilities;
    }

    public ProbabilitiesDto getOpponentProbabilities() {
        return opponentProbabilities;
    }

    public DecisionDto getDecision() {
        return decision;
    }

    public static class ProbabilitiesDto {
        private final double pair;
        private final double threeOfAKind;
        private final double straight;
        private final double flush;
        private final double fullHouse;
        private final double fourOfAKind;
        private final double straightFlush;
        private final double royalFlush;

        @JsonCreator
        public ProbabilitiesDto(
                @JsonProperty("pair") double pair,
                @JsonProperty("threeOfAKind") double threeOfAKind,
                @JsonProperty("straight") double straight,
                @JsonProperty("flush") double flush,
                @JsonProperty("fullHouse") double fullHouse,
                @JsonProperty("fourOfAKind") double fourOfAKind,
                @JsonProperty("straightFlush") double straightFlush,
                @JsonProperty("royalFlush") double royalFlush) {
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

    public static class DecisionDto {
        private final String action;
        private final int betAmount;
        private final String description;

        @JsonCreator
        public DecisionDto(
                @JsonProperty("action") String action,
                @JsonProperty("betAmount") int betAmount,
                @JsonProperty("description") String description) {
            this.action = action;
            this.betAmount = betAmount;
            this.description = description;
        }

        public String getAction() { return action; }
        public int getBetAmount() { return betAmount; }
        public String getDescription() { return description; }
    }
}
