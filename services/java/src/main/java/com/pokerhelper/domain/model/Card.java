package com.pokerhelper.domain.model;

import java.util.Objects;

/**
 * Domain model for a playing card
 */
public class Card {
    
    private final char suit;
    private final int rank;

    public Card(char suit, int rank) {
        this.suit = suit;
        // Convert Ace low (1) to Ace high (14) for easier calculations
        this.rank = (rank == 1) ? 14 : rank;
    }

    public char getSuit() {
        return suit;
    }

    public int getRank() {
        return rank;
    }

    /**
     * Check if this card has a higher rank than another card
     */
    public boolean isHigherThan(Card other) {
        return this.rank > other.rank;
    }

    /**
     * Check if this card has the same suit as another card
     */
    public boolean sameSuitAs(Card other) {
        return this.suit == other.suit;
    }

    /**
     * Check if this card has the same rank as another card
     */
    public boolean sameRankAs(Card other) {
        return this.rank == other.rank;
    }

    /**
     * Check if these two cards can form a straight
     */
    public boolean canFormStraightWith(Card other) {
        return Math.abs(this.rank - other.rank) <= 4;
    }

    @Override
    public boolean equals(Object o) {
        if (this == o) return true;
        if (o == null || getClass() != o.getClass()) return false;
        Card card = (Card) o;
        return suit == card.suit && rank == card.rank;
    }

    @Override
    public int hashCode() {
        return Objects.hash(suit, rank);
    }

    @Override
    public String toString() {
        return "Card{" +
                "suit=" + suit +
                ", rank=" + rank +
                '}';
    }
}
