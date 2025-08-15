package com.pokerhelper.infrastructure.adapters.input.http;

import com.pokerhelper.application.ports.input.PokerCalculatorUseCases;
import com.pokerhelper.domain.model.Card;
import com.pokerhelper.domain.model.PokerProbabilities;
import com.pokerhelper.infrastructure.dto.PokerCalculationRequest;
import com.pokerhelper.infrastructure.dto.PokerCalculationResponse;
import com.pokerhelper.infrastructure.mappers.PokerDtoMapper;
import io.javalin.Javalin;
import io.javalin.http.Context;
import io.javalin.http.HttpStatus;
import jakarta.validation.ConstraintViolation;
import jakarta.validation.Validator;
import org.slf4j.Logger;
import org.slf4j.LoggerFactory;

import java.util.List;
import java.util.Set;
import java.util.stream.Collectors;


/**
 * HTTP adapter for poker calculation endpoints
 */
public class PokerHttpAdapter {

    private static final Logger logger = LoggerFactory.getLogger(PokerHttpAdapter.class);
    
    private final PokerCalculatorUseCases pokerCalculatorUseCases;
    private final PokerDtoMapper pokerDtoMapper;
    private final Validator validator;

    public PokerHttpAdapter(
            PokerCalculatorUseCases pokerCalculatorUseCases,
            PokerDtoMapper pokerDtoMapper,
            Validator validator) {
        this.pokerCalculatorUseCases = pokerCalculatorUseCases;
        this.pokerDtoMapper = pokerDtoMapper;
        this.validator = validator;
    }

    /**
     * Configure routes for this adapter
     */
    public void configureRoutes(Javalin app) {
        String javaApiUrl = System.getenv("JAVA_API_PATH");
        if (javaApiUrl == null) {
            javaApiUrl = "/api/poker"; // Default path
        }
        app.post(javaApiUrl + "/calculate", this::calculateProbabilities);
        app.post(javaApiUrl + "/decision", this::makeDecision);
        app.get(javaApiUrl + "/info", this::getApiInfo);
    }

    /**
     * Calculate poker probabilities endpoint
     */
    private void calculateProbabilities(Context ctx) {
        try {
            logger.info("Received poker calculation request");
            
            // Parse and validate request
            PokerCalculationRequest request = ctx.bodyAsClass(PokerCalculationRequest.class);
            validateRequest(request);
            
            logger.debug("Request validated successfully: {}", request);
            
            // Convert DTO to domain objects
            List<Card> pocketCards = pokerDtoMapper.toDomainCards(request.getPocketCards());
            List<Card> communityCards = pokerDtoMapper.toDomainCards(request.getCommunityCards());

            // Calculate probabilities
            PokerProbabilities result = pokerCalculatorUseCases.calculateProbabilities(
                    pocketCards, 
                    communityCards, 
                    request.getNumberOfOpponents(),
                    request.getSmallBlind(),
                    request.getAccumulatedBet()
            );
            
            // Convert result to DTO
            PokerCalculationResponse response = pokerDtoMapper.toResponseDto(result);
            
            logger.info("Poker calculation completed successfully");
            ctx.status(HttpStatus.OK).json(response);
            
        } catch (ValidationException e) {
            logger.warn("Validation error in poker calculation: {}", e.getMessage());
            ctx.status(HttpStatus.BAD_REQUEST).json(new ErrorResponse("Validation error", e.getMessage()));
        } catch (Exception e) {
            logger.error("Error calculating poker probabilities", e);
            ctx.status(HttpStatus.INTERNAL_SERVER_ERROR).json(new ErrorResponse("Internal server error", "An error occurred while calculating probabilities"));
        }
    }

    /**
     * Make poker decision endpoint
     */
    private void makeDecision(Context ctx) {
        try {
            logger.info("Received poker decision request");
            
            // Parse and validate request
            PokerCalculationRequest request = ctx.bodyAsClass(PokerCalculationRequest.class);
            validateRequest(request);
            
            // Convert DTO to domain objects
            var pocketCards = pokerDtoMapper.toDomainCards(request.getPocketCards());
            var communityCards = pokerDtoMapper.toDomainCards(request.getCommunityCards());
            
            // Make decision
            var decision = pokerCalculatorUseCases.makeDecision(
                    pocketCards, 
                    communityCards, 
                    request.getNumberOfOpponents(),
                    request.getSmallBlind(),
                    request.getAccumulatedBet()
            );
            
            // Convert result to DTO
            var decisionDto = pokerDtoMapper.toDecisionDto(decision);
            
            logger.info("Poker decision completed successfully: {}", decisionDto.getAction());
            ctx.status(HttpStatus.OK).json(decisionDto);
            
        } catch (ValidationException e) {
            logger.warn("Validation error in poker decision: {}", e.getMessage());
            ctx.status(HttpStatus.BAD_REQUEST).json(new ErrorResponse("Validation error", e.getMessage()));
        } catch (Exception e) {
            logger.error("Error making poker decision", e);
            ctx.status(HttpStatus.INTERNAL_SERVER_ERROR).json(new ErrorResponse("Internal server error", "An error occurred while making decision"));
        }
    }

    /**
     * Get API information endpoint
     */
    private void getApiInfo(Context ctx) {
        ctx.json(new ApiInfo(
                "Poker Helper API",
                "1.0.0",
                "API for calculating poker probabilities and making decisions",
                new String[]{
                        "POST /api/poker/calculate - Calculate poker probabilities",
                        "POST /api/poker/decision - Make poker decision",
                        "GET /api/poker/info - Get API information"
                }
        ));
    }

    private void validateRequest(PokerCalculationRequest request) {
        Set<ConstraintViolation<PokerCalculationRequest>> violations = validator.validate(request);
        if (!violations.isEmpty()) {
            String errorMessage = violations.stream()
                    .map(ConstraintViolation::getMessage)
                    .collect(Collectors.joining(", "));
            throw new ValidationException(errorMessage);
        }
    }

    public static class ValidationException extends RuntimeException {
        public ValidationException(String message) {
            super(message);
        }
    }

    public static class ErrorResponse {
        public final String error;
        public final String message;

        public ErrorResponse(String error, String message) {
            this.error = error;
            this.message = message;
        }
    }

    public static class ApiInfo {
        public final String name;
        public final String version;
        public final String description;
        public final String[] endpoints;

        public ApiInfo(String name, String version, String description, String[] endpoints) {
            this.name = name;
            this.version = version;
            this.description = description;
            this.endpoints = endpoints;
        }
    }
}
