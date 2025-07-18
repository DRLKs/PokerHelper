package com.pokerhelper.infrastructure.dto;

import com.fasterxml.jackson.annotation.JsonCreator;
import com.fasterxml.jackson.annotation.JsonProperty;
import jakarta.validation.constraints.Max;
import jakarta.validation.constraints.Min;
import jakarta.validation.constraints.NotNull;
import jakarta.validation.constraints.Size;

import java.util.List;

/**
 * DTO for poker calculation request
 */
public class PokerCalculationRequest {

    @NotNull(message = "Pocket cards cannot be null")
    @Size(min = 2, max = 2, message = "Pocket cards must contain exactly 2 cards")
    private final List<CardDto> pocketCards;

    @NotNull(message = "Community cards cannot be null")
    @Size(max = 5, message = "Community cards cannot exceed 5 cards")
    private final List<CardDto> communityCards;

    @NotNull(message = "Number of opponents cannot be null")
    @Min(value = 0, message = "Number of opponents cannot be negative")
    @Max(value = 8, message = "Number of opponents cannot exceed 8")
    private final Integer numberOfOpponents;

    @Min(value = 1, message = "Small blind must be at least 1")
    private final Integer smallBlind;

    @Min(value = 0, message = "Accumulated bet cannot be negative")
    private final Integer accumulatedBet;

    @JsonCreator
    public PokerCalculationRequest(
            @JsonProperty("pocketCards") List<CardDto> pocketCards,
            @JsonProperty("communityCards") List<CardDto> communityCards,
            @JsonProperty("numberOfOpponents") Integer numberOfOpponents,
            @JsonProperty("smallBlind") Integer smallBlind,
            @JsonProperty("accumulatedBet") Integer accumulatedBet) {
        this.pocketCards = pocketCards;
        this.communityCards = communityCards;
        this.numberOfOpponents = numberOfOpponents;
        this.smallBlind = smallBlind != null ? smallBlind : 5;
        this.accumulatedBet = accumulatedBet != null ? accumulatedBet : 0;
    }

    public List<CardDto> getPocketCards() {
        return pocketCards;
    }

    public List<CardDto> getCommunityCards() {
        return communityCards;
    }

    public Integer getNumberOfOpponents() {
        return numberOfOpponents;
    }

    public Integer getSmallBlind() {
        return smallBlind;
    }

    public Integer getAccumulatedBet() {
        return accumulatedBet;
    }
}
