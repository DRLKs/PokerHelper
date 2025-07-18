package com.pokerhelper.infrastructure.config;

import com.pokerhelper.application.ports.input.PokerCalculatorUseCases;
import com.pokerhelper.application.services.PokerCalculatorService;
import com.pokerhelper.infrastructure.adapters.input.http.PokerHttpAdapter;
import com.pokerhelper.infrastructure.adapters.legacy.LegacyPokerCalculatorAdapter;
import com.pokerhelper.infrastructure.mappers.PokerDtoMapper;
import jakarta.validation.Validation;
import jakarta.validation.Validator;
import org.slf4j.Logger;
import org.slf4j.LoggerFactory;

/**
 * Application configuration and dependency injection
 */
public class ApplicationConfig {

    private static final Logger logger = LoggerFactory.getLogger(ApplicationConfig.class);
    
    private final Validator validator;
    private final PokerDtoMapper pokerDtoMapper;
    private final LegacyPokerCalculatorAdapter legacyAdapter;
    private final PokerCalculatorUseCases pokerCalculatorUseCases;
    private final PokerHttpAdapter pokerHttpAdapter;

    public ApplicationConfig() {
        logger.info("Initializing application configuration");
        
        // Initialize validation
        this.validator = Validation.buildDefaultValidatorFactory().getValidator();
        
        // Initialize mappers
        this.pokerDtoMapper = new PokerDtoMapper();
        
        // Initialize adapters
        this.legacyAdapter = new LegacyPokerCalculatorAdapter();
        
        // Initialize use cases
        this.pokerCalculatorUseCases = new PokerCalculatorService(legacyAdapter);
        
        // Initialize HTTP adapter
        this.pokerHttpAdapter = new PokerHttpAdapter(pokerCalculatorUseCases, pokerDtoMapper, validator);
        
        logger.info("Application configuration initialized successfully");
    }

    public Validator getValidator() {
        return validator;
    }

    public PokerDtoMapper getPokerDtoMapper() {
        return pokerDtoMapper;
    }

    public LegacyPokerCalculatorAdapter getLegacyAdapter() {
        return legacyAdapter;
    }

    public PokerCalculatorUseCases getPokerCalculatorUseCases() {
        return pokerCalculatorUseCases;
    }

    public PokerHttpAdapter getPokerHttpAdapter() {
        return pokerHttpAdapter;
    }
}
