package com.pokerhelper.infrastructure.adapters.input.http;

import com.fasterxml.jackson.databind.ObjectMapper;
import com.fasterxml.jackson.datatype.jsr310.JavaTimeModule;
import com.pokerhelper.application.ports.input.PokerCalculatorUseCases;
import com.pokerhelper.infrastructure.dto.CardDto;
import com.pokerhelper.infrastructure.dto.PokerCalculationRequest;
import com.pokerhelper.infrastructure.mappers.PokerDtoMapper;
import io.javalin.Javalin;
import io.javalin.json.JavalinJackson;
import io.javalin.testtools.JavalinTest;
import jakarta.validation.Validation;
import jakarta.validation.Validator;
import org.junit.jupiter.api.BeforeEach;
import org.junit.jupiter.api.Test;
import org.junit.jupiter.api.DisplayName;
import org.mockito.Mock;
import org.mockito.MockitoAnnotations;

import java.util.Arrays;
import java.util.List;

import static org.junit.jupiter.api.Assertions.*;
import static org.mockito.Mockito.*;

/**
 * Tests for PokerHttpAdapter
 */
class PokerHttpAdapterTest {

    @Mock
    private PokerCalculatorUseCases mockPokerCalculatorUseCases;

    private PokerDtoMapper pokerDtoMapper;
    private Validator validator;
    private PokerHttpAdapter pokerHttpAdapter;
    private ObjectMapper objectMapper;

    @BeforeEach
    void setUp() {
        MockitoAnnotations.openMocks(this);
        
        // Initialize real dependencies
        this.pokerDtoMapper = new PokerDtoMapper();
        this.validator = Validation.buildDefaultValidatorFactory().getValidator();
        this.objectMapper = new ObjectMapper();
        this.objectMapper.registerModule(new JavaTimeModule());
        
        // Create adapter with mock use case
        this.pokerHttpAdapter = new PokerHttpAdapter(
                mockPokerCalculatorUseCases, 
                pokerDtoMapper, 
                validator
        );
    }

    @Test
    @DisplayName("Should return 400 Bad Request when pocket cards contain more than 2 cards")
    void shouldReturnBadRequestWhenPocketCardsExceedTwoCards() {
        // Given: A request with 3 pocket cards (invalid)
        List<CardDto> invalidPocketCards = Arrays.asList(
                new CardDto("S", 14), // Ace of Spades
                new CardDto("H", 13), // King of Hearts
                new CardDto("D", 12)  // Queen of Diamonds (EXTRA CARD - INVALID)
        );
        
        List<CardDto> communityCards = Arrays.asList(
                new CardDto("C", 11), // Jack of Clubs
                new CardDto("S", 10)  // 10 of Spades
        );
        
        PokerCalculationRequest invalidRequest = new PokerCalculationRequest(
                invalidPocketCards,
                communityCards,
                2, // numberOfOpponents
                10, // smallBlind
                50  // accumulatedBet
        );

        // When & Then: Create Javalin app and test the endpoint
        Javalin app = createTestApp();
        
        JavalinTest.test(app, (server, client) -> {
            String requestBody = objectMapper.writeValueAsString(invalidRequest);
            
            var response = client.post("/api/poker/calculate", requestBody, 
                    headers -> headers.header("Content-Type", "application/json"));
            
            // Should return 400 Bad Request
            assertEquals(400, response.code());
            
            // Response should contain validation error message
            String responseBody = response.body().string();
            assertTrue(responseBody.contains("Validation error"));
            assertTrue(responseBody.contains("Pocket cards must contain exactly 2 cards"));
        });
        
        // Verify that the use case was never called due to validation failure
        verifyNoInteractions(mockPokerCalculatorUseCases);
    }

    @Test
    @DisplayName("Should return 400 Bad Request when pocket cards contain less than 2 cards")
    void shouldReturnBadRequestWhenPocketCardsLessThanTwoCards() {
        // Given: A request with only 1 pocket card (invalid)
        List<CardDto> invalidPocketCards = Arrays.asList(
                new CardDto("S", 14) // Only Ace of Spades (MISSING CARD - INVALID)
        );
        
        List<CardDto> communityCards = Arrays.asList(
                new CardDto("C", 11), // Jack of Clubs
                new CardDto("S", 10)  // 10 of Spades
        );
        
        PokerCalculationRequest invalidRequest = new PokerCalculationRequest(
                invalidPocketCards,
                communityCards,
                2, // numberOfOpponents
                10, // smallBlind
                50  // accumulatedBet
        );

        // When & Then: Create Javalin app and test the endpoint
        Javalin app = createTestApp();
        
        JavalinTest.test(app, (server, client) -> {
            String requestBody = objectMapper.writeValueAsString(invalidRequest);
            
            var response = client.post("/api/poker/calculate", requestBody, 
                    headers -> headers.header("Content-Type", "application/json"));
            
            // Should return 400 Bad Request
            assertEquals(400, response.code());
            
            // Response should contain validation error message
            String responseBody = response.body().string();
            assertTrue(responseBody.contains("Validation error"));
            assertTrue(responseBody.contains("Pocket cards must contain exactly 2 cards"));
        });
        
        // Verify that the use case was never called due to validation failure
        verifyNoInteractions(mockPokerCalculatorUseCases);
    }

    @Test
    @DisplayName("Should return 400 Bad Request when pocket cards are null")
    void shouldReturnBadRequestWhenPocketCardsAreNull() {
        // Given: A request with null pocket cards (invalid)
        List<CardDto> communityCards = Arrays.asList(
                new CardDto("C", 11), // Jack of Clubs
                new CardDto("S", 10)  // 10 of Spades
        );
        
        PokerCalculationRequest invalidRequest = new PokerCalculationRequest(
                null, // NULL POCKET CARDS - INVALID
                communityCards,
                2, // numberOfOpponents
                10, // smallBlind
                50  // accumulatedBet
        );

        // When & Then: Create Javalin app and test the endpoint
        Javalin app = createTestApp();
        
        JavalinTest.test(app, (server, client) -> {
            String requestBody = objectMapper.writeValueAsString(invalidRequest);
            
            var response = client.post("/api/poker/calculate", requestBody, 
                    headers -> headers.header("Content-Type", "application/json"));
            
            // Should return 400 Bad Request
            assertEquals(400, response.code());
            
            // Response should contain validation error message
            String responseBody = response.body().string();
            assertTrue(responseBody.contains("Validation error"));
            assertTrue(responseBody.contains("Pocket cards cannot be null"));
        });
        
        // Verify that the use case was never called due to validation failure
        verifyNoInteractions(mockPokerCalculatorUseCases);
    }

    @Test
    @DisplayName("Should return 400 Bad Request when community cards exceed 5 cards")
    void shouldReturnBadRequestWhenCommunityCardsExceedFiveCards() {
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
        
        PokerCalculationRequest invalidRequest = new PokerCalculationRequest(
                validPocketCards,
                invalidCommunityCards,
                2, // numberOfOpponents
                10, // smallBlind
                50  // accumulatedBet
        );

        // When & Then: Create Javalin app and test the endpoint
        Javalin app = createTestApp();
        
        JavalinTest.test(app, (server, client) -> {
            String requestBody = objectMapper.writeValueAsString(invalidRequest);
            
            var response = client.post("/api/poker/calculate", requestBody, 
                    headers -> headers.header("Content-Type", "application/json"));
            
            // Should return 400 Bad Request
            assertEquals(400, response.code());
            
            // Response should contain validation error message
            String responseBody = response.body().string();
            assertTrue(responseBody.contains("Validation error"));
            assertTrue(responseBody.contains("Community cards cannot exceed 5 cards"));
        });
        
        // Verify that the use case was never called due to validation failure
        verifyNoInteractions(mockPokerCalculatorUseCases);
    }

    @Test
    @DisplayName("Should return 400 Bad Request when number of opponents exceeds 8")
    void shouldReturnBadRequestWhenOpponentsExceedEight() {
        // Given: A request with valid cards but too many opponents
        List<CardDto> validPocketCards = Arrays.asList(
                new CardDto("S", 14), // Ace of Spades
                new CardDto("H", 13)  // King of Hearts
        );
        
        List<CardDto> validCommunityCards = Arrays.asList(
                new CardDto("C", 11), // Jack of Clubs
                new CardDto("S", 10)  // 10 of Spades
        );
        
        PokerCalculationRequest invalidRequest = new PokerCalculationRequest(
                validPocketCards,
                validCommunityCards,
                10, // TOO MANY OPPONENTS - INVALID (max is 8)
                10, // smallBlind
                50  // accumulatedBet
        );

        // When & Then: Create Javalin app and test the endpoint
        Javalin app = createTestApp();
        
        JavalinTest.test(app, (server, client) -> {
            String requestBody = objectMapper.writeValueAsString(invalidRequest);
            
            var response = client.post("/api/poker/calculate", requestBody, 
                    headers -> headers.header("Content-Type", "application/json"));
            
            // Should return 400 Bad Request
            assertEquals(400, response.code());
            
            // Response should contain validation error message
            String responseBody = response.body().string();
            assertTrue(responseBody.contains("Validation error"));
            assertTrue(responseBody.contains("Number of opponents cannot exceed 8"));
        });
        
        // Verify that the use case was never called due to validation failure
        verifyNoInteractions(mockPokerCalculatorUseCases);
    }

    @Test
    @DisplayName("Should return 400 Bad Request when pocket cards contain exactly 3 cards (edge case)")
    void shouldReturnBadRequestWhenPocketCardsContainExactlyThreeCards() {
        // Given: A request with exactly 3 pocket cards (1 more than allowed)
        List<CardDto> invalidPocketCards = Arrays.asList(
                new CardDto("S", 14), // Ace of Spades
                new CardDto("H", 13), // King of Hearts  
                new CardDto("D", 12)  // Queen of Diamonds (THIRD CARD - INVALID)
        );
        
        List<CardDto> communityCards = Arrays.asList(
                new CardDto("C", 11), // Jack of Clubs
                new CardDto("S", 10), // 10 of Spades
                new CardDto("H", 9)   // 9 of Hearts
        );
        
        PokerCalculationRequest invalidRequest = new PokerCalculationRequest(
                invalidPocketCards,
                communityCards,
                3, // numberOfOpponents
                20, // smallBlind
                100  // accumulatedBet
        );

        // When & Then: Create Javalin app and test the endpoint
        Javalin app = createTestApp();
        
        JavalinTest.test(app, (server, client) -> {
            String requestBody = objectMapper.writeValueAsString(invalidRequest);
            
            var response = client.post("/api/poker/calculate", requestBody, 
                    headers -> headers.header("Content-Type", "application/json"));
            
            // Should return 400 Bad Request
            assertEquals(400, response.code());
            
            // Response should contain validation error message
            String responseBody = response.body().string();
            assertTrue(responseBody.contains("Validation error"));
            assertTrue(responseBody.contains("Pocket cards must contain exactly 2 cards"));
            
            // Log the response for debugging
            System.out.println("Response body: " + responseBody);
        });
        
        // Verify that the use case was never called due to validation failure
        verifyNoInteractions(mockPokerCalculatorUseCases);
    }

    @Test
    @DisplayName("Should return 400 Bad Request when pocket cards contain 4 cards (many cards)")
    void shouldReturnBadRequestWhenPocketCardsContainFourCards() {
        // Given: A request with 4 pocket cards (way more than allowed)
        List<CardDto> invalidPocketCards = Arrays.asList(
                new CardDto("S", 14), // Ace of Spades
                new CardDto("H", 13), // King of Hearts
                new CardDto("D", 12), // Queen of Diamonds
                new CardDto("C", 11)  // Jack of Clubs (FOURTH CARD - INVALID)
        );
        
        List<CardDto> communityCards = Arrays.asList(
                new CardDto("S", 10), // 10 of Spades
                new CardDto("H", 9)   // 9 of Hearts
        );
        
        PokerCalculationRequest invalidRequest = new PokerCalculationRequest(
                invalidPocketCards,
                communityCards,
                1, // numberOfOpponents
                5, // smallBlind
                25  // accumulatedBet
        );

        // When & Then: Create Javalin app and test the endpoint
        Javalin app = createTestApp();
        
        JavalinTest.test(app, (server, client) -> {
            String requestBody = objectMapper.writeValueAsString(invalidRequest);
            
            var response = client.post("/api/poker/calculate", requestBody, 
                    headers -> headers.header("Content-Type", "application/json"));
            
            // Should return 400 Bad Request
            assertEquals(400, response.code());
            
            // Response should contain validation error message
            String responseBody = response.body().string();
            assertTrue(responseBody.contains("Validation error"));
            assertTrue(responseBody.contains("Pocket cards must contain exactly 2 cards"));
        });
        
        // Verify that the use case was never called due to validation failure
        verifyNoInteractions(mockPokerCalculatorUseCases);
    }

    private Javalin createTestApp() {
        Javalin app = Javalin.create(config -> {
            config.jsonMapper(new JavalinJackson(objectMapper));
            config.showJavalinBanner = false;
        });
        
        // Configure routes using the adapter
        pokerHttpAdapter.configureRoutes(app);
        
        return app;
    }
}
