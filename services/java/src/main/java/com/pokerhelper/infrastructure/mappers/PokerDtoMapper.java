package com.pokerhelper.infrastructure.mappers;

import com.pokerhelper.domain.model.Card;
import com.pokerhelper.domain.model.Decision;
import com.pokerhelper.domain.model.PokerProbabilities;
import com.pokerhelper.infrastructure.dto.CardDto;
import com.pokerhelper.infrastructure.dto.PokerCalculationResponse;

import java.util.List;
import java.util.stream.Collectors;

/**
 * Mapper to convert between DTOs and domain models                                                                     
 */
public class PokerDtoMapper {

    /**
     * Convert CardDto list to domain Card list
     */
    public List<Card> toDomainCards(List<CardDto> cardDtos) {
        if (cardDtos == null) {
            return List.of();
        }
        
        return cardDtos.stream()
                .map(this::toDomainCard)
                .collect(Collectors.toList());
    }

    /**
     * Convert CardDto to domain Card
     */
    public Card toDomainCard(CardDto cardDto) {
        if (cardDto == null) {
            throw new IllegalArgumentException("CardDto cannot be null");
        }
        
        return new Card(cardDto.getSuitAsChar(), cardDto.getRank());
    }

    /**
     * Convert domain PokerProbabilities to response DTO
     */
    public PokerCalculationResponse toResponseDto(PokerProbabilities probabilities) {
        if (probabilities == null) {
            throw new IllegalArgumentException("PokerProbabilities cannot be null");
        }
        
        var playerProbs = toProbabilitiesDto(probabilities.getPlayerProbabilities());
        var opponentProbs = toProbabilitiesDto(probabilities.getOpponentProbabilities());
        var decision = toDecisionDto(probabilities.getDecision());
        
        return new PokerCalculationResponse(playerProbs, opponentProbs, decision);
    }

    /**
     * Convert domain Decision to DecisionDto
     */
    public PokerCalculationResponse.DecisionDto toDecisionDto(Decision decision) {
        if (decision == null) {
            return new PokerCalculationResponse.DecisionDto("FOLD", 0, "No decision made");
        }
        
        return new PokerCalculationResponse.DecisionDto(
                decision.getActionName(),
                decision.getBetAmount(),
                decision.getDescription()
        );
    }

    /**
     * Convert domain PlayerProbabilities to ProbabilitiesDto
     */
    private PokerCalculationResponse.ProbabilitiesDto toProbabilitiesDto(
            PokerProbabilities.PlayerProbabilities probabilities) {
        
        if (probabilities == null) {
            return new PokerCalculationResponse.ProbabilitiesDto(0, 0, 0, 0, 0, 0, 0, 0);
        }
        
        return new PokerCalculationResponse.ProbabilitiesDto(
                probabilities.getPair(),
                probabilities.getThreeOfAKind(),
                probabilities.getStraight(),
                probabilities.getFlush(),
                probabilities.getFullHouse(),
                probabilities.getFourOfAKind(),
                probabilities.getStraightFlush(),
                probabilities.getRoyalFlush()
        );
    }
}
