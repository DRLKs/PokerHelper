package com.pokerhelper.infrastructure.dto;

import com.fasterxml.jackson.annotation.JsonCreator;
import com.fasterxml.jackson.annotation.JsonIgnore;
import com.fasterxml.jackson.annotation.JsonProperty;
import jakarta.validation.constraints.Max;
import jakarta.validation.constraints.Min;
import jakarta.validation.constraints.NotNull;
import jakarta.validation.constraints.Pattern;

/**
 * DTO representing a playing card
 */
public class CardDto {

    @NotNull(message = "Card suit cannot be null")
    @Pattern(regexp = "[SHDC]", message = "Card suit must be one of: S (Spades), H (Hearts), D (Diamonds), C (Clubs)")
    private final String suit;

    @NotNull(message = "Card rank cannot be null")
    @Min(value = 1, message = "Card rank must be between 1 and 14 (1=Ace, 11=Jack, 12=Queen, 13=King, 14=Ace high)")
    @Max(value = 14, message = "Card rank must be between 1 and 14 (1=Ace, 11=Jack, 12=Queen, 13=King, 14=Ace high)")
    private final Integer rank;

    @JsonCreator
    public CardDto(
            @JsonProperty("suit") String suit,
            @JsonProperty("rank") Integer rank) {
        this.suit = suit;
        this.rank = rank;
    }

    public String getSuit() {
        return suit;
    }

    public Integer getRank() {
        return rank;
    }

    @JsonIgnore
    public char getSuitAsChar() {
        return suit != null ? suit.charAt(0) : 'S';
    }

    @Override
    public String toString() {
        return "CardDto{" +
                "suit='" + suit + '\'' +
                ", rank=" + rank +
                '}';
    }
}
