package com.pokerhelper.infrastructure.adapters.legacy;

import com.pokerhelper.application.services.CalculoDeProbabilidades;
import com.pokerhelper.domain.model.Card;
import com.pokerhelper.domain.model.Decision;
import com.pokerhelper.domain.model.PokerProbabilities;
import org.slf4j.Logger;
import org.slf4j.LoggerFactory;

import java.util.List;
import java.util.stream.Collectors;

/**
 * Adapter to integrate with legacy poker calculation code
 */
public class LegacyPokerCalculatorAdapter {

    private static final Logger logger = LoggerFactory.getLogger(LegacyPokerCalculatorAdapter.class);
    
    private final CalculoDeProbabilidades legacyCalculator;

    public LegacyPokerCalculatorAdapter() {
        this.legacyCalculator = new CalculoDeProbabilidades();
    }

    /**
     * Calculate poker probabilities using legacy code
     */
    public PokerProbabilities calculateProbabilities(
            List<Card> cards,
            int numberOfOpponents,
            int smallBlind,
            int accumulatedBet) {
        
        logger.debug("Converting cards to legacy format");
        
        // Convert domain cards to legacy cards
        List<Card> legacyCards = cards.stream()
                .map(this::convertToLegacyCard)
                .collect(Collectors.toList());
        
        logger.debug("Calling legacy calculator with {} cards", legacyCards.size());
        
        // Call legacy calculator
        legacyCalculator.reiniciarDatos(legacyCards, numberOfOpponents, smallBlind, accumulatedBet);
        
        // Extract probabilities from legacy calculator
        var playerProbs = new PokerProbabilities.PlayerProbabilities(
                legacyCalculator.getProbPareja(),
                legacyCalculator.getProbTrio(),
                legacyCalculator.getProbEscalera(),
                legacyCalculator.getProbColor(),
                legacyCalculator.getProbFullHouse(),
                legacyCalculator.getProbPoker(),
                legacyCalculator.getProbEscaleraColor(),
                legacyCalculator.getProbEscaleraReal()
        );
        
        var opponentProbs = new PokerProbabilities.PlayerProbabilities(
                legacyCalculator.getProbParejaCont(),
                legacyCalculator.getProbTrioCont(),
                legacyCalculator.getProbEscaleraCont(),
                legacyCalculator.getProbColorCont(),
                legacyCalculator.getProbFullHouseCont(),
                legacyCalculator.getProbPokerCont(),
                legacyCalculator.getProbEscaleraColorCont(),
                legacyCalculator.getProbEscaleraRealCont()
        );
        
        // Convert legacy decision to domain decision
        var legacyDecision = legacyCalculator.getDecision();
        var decision = convertToDomainDecision(legacyDecision);
        
        logger.debug("Legacy calculation completed successfully");
        
        return new PokerProbabilities(playerProbs, opponentProbs, decision);
    }

    /**
     * Make poker decision using legacy code
     */
    public Decision makeDecision(
            List<Card> cards,
            int numberOfOpponents,
            int smallBlind,
            int accumulatedBet) {
        
        // Calculate probabilities first (legacy code calculates decision as part of this)
        var probabilities = calculateProbabilities(cards, numberOfOpponents, smallBlind, accumulatedBet);
        
        return probabilities.getDecision();
    }

    /**
     * Convert domain Card to legacy Carta
     */
    private Card convertToLegacyCard(Card card) {
        return new Carta(card.getSuit(), card.getRank());
    }

    /**
     * Convert legacy Decision to domain Decision
     */
    private Decision convertToDomainDecision(com.app.service.applications.service.Decision legacyDecision) {
        if (legacyDecision == null) {
            return new Decision(Decision.FOLD, 0);
        }
        
        return new Decision(legacyDecision.getDecision(), legacyDecision.getAumentoApuesta());
    }
}
