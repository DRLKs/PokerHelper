package com.pokerhelper.application.ports.input;

import com.pokerhelper.domain.model.Card;
import com.pokerhelper.domain.model.Decision;
import com.pokerhelper.domain.model.PokerProbabilities;

import java.util.List;

/**
 * Input port for poker calculations
 */
public interface PokerCalculatorUseCases {

    /**
     * Calculate poker probabilities for the given cards
     *
     * @param pocketCards The cards that only the player sees
     * @param communityCards The cards that everyone sees
     * @param numberOfOpponents Number of active opponents
     * @param smallBlind Value of the small blind
     * @param accumulatedBet Accumulated bet amount
     * @return Poker probabilities result
     */
    PokerProbabilities calculateProbabilities(
            List<Card> pocketCards,
            List<Card> communityCards,
            int numberOfOpponents,
            int smallBlind,
            int accumulatedBet
    );

    /**
     * Make a poker decision based on the current situation
     *
     * @param pocketCards The cards that only the player sees
     * @param communityCards The cards that everyone sees
     * @param numberOfOpponents Number of active opponents
     * @param smallBlind Value of the small blind
     * @param accumulatedBet Accumulated bet amount
     * @return Decision to make
     */
    Decision makeDecision(
            List<Card> pocketCards,
            List<Card> communityCards,
            int numberOfOpponents,
            int smallBlind,
            int accumulatedBet
    );
}
