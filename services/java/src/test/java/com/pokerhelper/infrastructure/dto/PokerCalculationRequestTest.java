package com.pokerhelper.infrastructure.dto;

import jakarta.validation.ConstraintViolation;
import jakarta.validation.Validation;
import jakarta.validation.Validator;
import org.junit.jupiter.api.BeforeEach;
import org.junit.jupiter.api.Test;
import org.junit.jupiter.api.DisplayName;

import java.util.Arrays;
import java.util.List;
import java.util.Set;

import static org.junit.jupiter.api.Assertions.*;

/**
 * Tests for PokerCalculationRequest DTO validation
 */
class PokerCalculationRequestTest {

    private Validator validator;

    @BeforeEach
    void setUp() {
        validator = Validation.buildDefaultValidatorFactory().getValidator();
    }

    @Test
    @DisplayName("Should fail validation when pocket cards contain more than 2 cards")
    void shouldFailValidationWhenPocketCardsExceedTwoCards() {
        // Given: A request with 3 pocket cards (invalid)
        List<CardDto> invalidPocketCards = Arrays.asList(
                new CardDto("S", 14), // Ace of Spades
                new CardDto("H", 13), // King of Hearts
                new CardDto("D", 12)  // Queen of Diamonds (EXTRA CARD - INVALID)
        );
        
        List<CardDto> validCommunityCards = Arrays.asList(
                new CardDto("C", 11) // Jack of Clubs
        );
        
        PokerCalculationRequest request = new PokerCalculationRequest(
                invalidPocketCards,
                validCommunityCards,
                2, // numberOfOpponents
                10, // smallBlind
                50  // accumulatedBet
        );

        // When: Validate the request
        Set<ConstraintViolation<PokerCalculationRequest>> violations = validator.validate(request);

        // Then: Should have validation errors
        assertFalse(violations.isEmpty(), "Should have validation errors");
        
        // Check that the specific error message is present
        boolean hasCorrectError = violations.stream()
                .anyMatch(violation -> 
                    violation.getPropertyPath().toString().equals("pocketCards") &&
                    violation.getMessage().contains("Pocket cards must contain exactly 2 cards")
                );
        
        assertTrue(hasCorrectError, "Should have the correct error message for pocket cards size");
    }

    @Test
    @DisplayName("Should fail validation when pocket cards contain less than 2 cards")
    void shouldFailValidationWhenPocketCardsLessThanTwoCards() {
        // Given: A request with only 1 pocket card (invalid)
        List<CardDto> invalidPocketCards = Arrays.asList(
                new CardDto("S", 14) // Only Ace of Spades (MISSING CARD - INVALID)
        );
        
        List<CardDto> validCommunityCards = Arrays.asList(
                new CardDto("C", 11) // Jack of Clubs
        );
        
        PokerCalculationRequest request = new PokerCalculationRequest(
                invalidPocketCards,
                validCommunityCards,
                2, // numberOfOpponents
                10, // smallBlind
                50  // accumulatedBet
        );

        // When: Validate the request
        Set<ConstraintViolation<PokerCalculationRequest>> violations = validator.validate(request);

        // Then: Should have validation errors
        assertFalse(violations.isEmpty(), "Should have validation errors");
        
        // Check that the specific error message is present
        boolean hasCorrectError = violations.stream()
                .anyMatch(violation -> 
                    violation.getPropertyPath().toString().equals("pocketCards") &&
                    violation.getMessage().contains("Pocket cards must contain exactly 2 cards")
                );
        
        assertTrue(hasCorrectError, "Should have the correct error message for pocket cards size");
    }

    @Test
    @DisplayName("Should fail validation when pocket cards are null")
    void shouldFailValidationWhenPocketCardsAreNull() {
        // Given: A request with null pocket cards
        List<CardDto> validCommunityCards = Arrays.asList(
                new CardDto("C", 11) // Jack of Clubs
        );
        
        PokerCalculationRequest request = new PokerCalculationRequest(
                null, // NULL POCKET CARDS - INVALID
                validCommunityCards,
                2, // numberOfOpponents
                10, // smallBlind
                50  // accumulatedBet
        );

        // When: Validate the request
        Set<ConstraintViolation<PokerCalculationRequest>> violations = validator.validate(request);

        // Then: Should have validation errors
        assertFalse(violations.isEmpty(), "Should have validation errors");
        
        // Check that the specific error message is present
        boolean hasCorrectError = violations.stream()
                .anyMatch(violation -> 
                    violation.getPropertyPath().toString().equals("pocketCards") &&
                    violation.getMessage().contains("Pocket cards cannot be null")
                );
        
        assertTrue(hasCorrectError, "Should have the correct error message for null pocket cards");
    }

    @Test
    @DisplayName("Should fail validation when community cards exceed 5 cards")
    void shouldFailValidationWhenCommunityCardsExceedFiveCards() {
        // Given: A request with valid pocket cards but too many community cards
        List<CardDto> validPocketCards = Arrays.asList(
                new CardDto("S", 14), // Ace of Spades
                new CardDto("H", 13)  // King of Hearts
        );
        
        List<CardDto> invalidCommunityCards = Arrays.asList(
                new CardDto("C", 11), // Jack of Clubs
                new CardDto("S", 10), // 10 of Spades
                new CardDto("D", 9),  // 9 of Diamonds
                new CardDto("H", 8),  // 8 of Hearts
                new CardDto("C", 7),  // 7 of Clubs
                new CardDto("S", 6)   // 6 of Spades (EXTRA CARD - INVALID)
        );
        
        PokerCalculationRequest request = new PokerCalculationRequest(
                validPocketCards,
                invalidCommunityCards,
                2, // numberOfOpponents
                10, // smallBlind
                50  // accumulatedBet
        );

        // When: Validate the request
        Set<ConstraintViolation<PokerCalculationRequest>> violations = validator.validate(request);

        // Then: Should have validation errors
        assertFalse(violations.isEmpty(), "Should have validation errors");
        
        // Check that the specific error message is present
        boolean hasCorrectError = violations.stream()
                .anyMatch(violation -> 
                    violation.getPropertyPath().toString().equals("communityCards") &&
                    violation.getMessage().contains("Community cards cannot exceed 5 cards")
                );
        
        assertTrue(hasCorrectError, "Should have the correct error message for community cards size");
    }

    @Test
    @DisplayName("Should pass validation with exactly 2 pocket cards and valid data")
    void shouldPassValidationWithValidData() {
        // Given: A valid request
        List<CardDto> validPocketCards = Arrays.asList(
                new CardDto("S", 14), // Ace of Spades
                new CardDto("H", 13)  // King of Hearts
        );
        
        List<CardDto> validCommunityCards = Arrays.asList(
                new CardDto("C", 11), // Jack of Clubs
                new CardDto("S", 10), // 10 of Spades
                new CardDto("D", 9)   // 9 of Diamonds
        );
        
        PokerCalculationRequest request = new PokerCalculationRequest(
                validPocketCards,
                validCommunityCards,
                2, // numberOfOpponents
                10, // smallBlind
                50  // accumulatedBet
        );

        // When: Validate the request
        Set<ConstraintViolation<PokerCalculationRequest>> violations = validator.validate(request);

        // Then: Should have no validation errors
        assertTrue(violations.isEmpty(), "Should have no validation errors with valid data");
    }

    @Test
    @DisplayName("Should fail validation when number of opponents is negative")
    void shouldFailValidationWhenOpponentsIsNegative() {
        // Given: A request with negative opponents
        List<CardDto> validPocketCards = Arrays.asList(
                new CardDto("S", 14), // Ace of Spades
                new CardDto("H", 13)  // King of Hearts
        );
        
        List<CardDto> validCommunityCards = Arrays.asList(
                new CardDto("C", 11) // Jack of Clubs
        );
        
        PokerCalculationRequest request = new PokerCalculationRequest(
                validPocketCards,
                validCommunityCards,
                -1, // NEGATIVE OPPONENTS - INVALID
                10, // smallBlind
                50  // accumulatedBet
        );

        // When: Validate the request
        Set<ConstraintViolation<PokerCalculationRequest>> violations = validator.validate(request);

        // Then: Should have validation errors
        assertFalse(violations.isEmpty(), "Should have validation errors");
        
        // Check that the specific error message is present
        boolean hasCorrectError = violations.stream()
                .anyMatch(violation -> 
                    violation.getPropertyPath().toString().equals("numberOfOpponents") &&
                    violation.getMessage().contains("Number of opponents cannot be negative")
                );
        
        assertTrue(hasCorrectError, "Should have the correct error message for negative opponents");
    }

    @Test
    @DisplayName("Should use default values for null smallBlind and accumulatedBet")
    void shouldUseDefaultValuesForNullFields() {
        // Given: A request with null optional fields
        List<CardDto> validPocketCards = Arrays.asList(
                new CardDto("S", 14), // Ace of Spades
                new CardDto("H", 13)  // King of Hearts
        );
        
        List<CardDto> validCommunityCards = Arrays.asList(
                new CardDto("C", 11) // Jack of Clubs
        );
        
        PokerCalculationRequest request = new PokerCalculationRequest(
                validPocketCards,
                validCommunityCards,
                2, // numberOfOpponents
                null, // smallBlind - should default to 5
                null  // accumulatedBet - should default to 0
        );

        // When: Get the values
        Integer smallBlind = request.getSmallBlind();
        Integer accumulatedBet = request.getAccumulatedBet();

        // Then: Should use default values
        assertEquals(5, smallBlind, "Small blind should default to 5");
        assertEquals(0, accumulatedBet, "Accumulated bet should default to 0");
    }
}
