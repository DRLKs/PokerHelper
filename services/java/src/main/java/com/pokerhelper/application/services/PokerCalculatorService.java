package com.pokerhelper.application.services;

import com.pokerhelper.application.ports.input.PokerCalculatorUseCases;
import com.pokerhelper.domain.model.Card;
import com.pokerhelper.domain.model.Decision;
import com.pokerhelper.domain.model.PokerProbabilities;
import org.slf4j.Logger;
import org.slf4j.LoggerFactory;

import java.util.List;

/**
 * Application service that implements poker calculation use cases
 */
public class PokerCalculatorService implements PokerCalculatorUseCases {

    private static final Logger logger = LoggerFactory.getLogger(PokerCalculatorService.class);
    private static final CalculoDeProbabilidades calculoDeProbabilidades = new CalculoDeProbabilidades();

    @Override
    public PokerProbabilities calculateProbabilities(
            List<Card> pocketCards,
            List<Card> communityCards,
            int numberOfOpponents,
            int smallBlind,
            int accumulatedBet) {
        
        logger.info("Calculating poker probabilities for {} pocket cards, {} community cards, {} opponents",
                pocketCards.size(), communityCards.size(), numberOfOpponents);
        
        try {
            // Combine all cards
            List<Card> allCards = combineCards(pocketCards, communityCards);
            
            // Use legacy calculator to compute probabilities
            calculoDeProbabilidades.reiniciarDatos (
                    allCards, numberOfOpponents, smallBlind, accumulatedBet);
            
            logger.info("Poker probabilities calculated successfully");
            return result;
            
        } catch (Exception e) {
            logger.error("Error calculating poker probabilities", e);
            throw new PokerCalculationException("Failed to calculate poker probabilities", e);
        }
    }

    @Override
    public Decision makeDecision(
            List<Card> pocketCards,
            List<Card> communityCards,
            int numberOfOpponents,
            int smallBlind,
            int accumulatedBet) {
        
        logger.info("Making poker decision for {} pocket cards, {} community cards, {} opponents",
                pocketCards.size(), communityCards.size(), numberOfOpponents);
        
        try {
            // Combine all cards
            List<Card> allCards = combineCards(pocketCards, communityCards);
            
            // Use legacy calculator to make decision
            var decision = legacyCalculator.makeDecision(
                    allCards, numberOfOpponents, smallBlind, accumulatedBet);
            
            logger.info("Poker decision made: {}", decision.getActionName());
            return decision;
            
        } catch (Exception e) {
            logger.error("Error making poker decision", e);
            throw new PokerCalculationException("Failed to make poker decision", e);
        }
    }

    private List<Card> combineCards(List<Card> pocketCards, List<Card> communityCards) {
        if (pocketCards == null || pocketCards.size() != 2) {
            throw new IllegalArgumentException("Pocket cards must contain exactly 2 cards");
        }
        if (communityCards == null) {
            throw new IllegalArgumentException("Community cards cannot be null");
        }
        if (communityCards.size() > 5) {
            throw new IllegalArgumentException("Community cards cannot exceed 5 cards");
        }
        
        List<Card> allCards = new java.util.ArrayList<>(pocketCards);
        allCards.addAll(communityCards);
        
        return allCards;
    }

    public static class PokerCalculationException extends RuntimeException {
        public PokerCalculationException(String message, Throwable cause) {
            super(message, cause);
        }
    }
}
