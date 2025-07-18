package com.pokerhelper;

import com.pokerhelper.infrastructure.config.ApplicationConfig;
import io.javalin.Javalin;
import io.javalin.json.JavalinJackson;
import com.fasterxml.jackson.databind.ObjectMapper;
import com.fasterxml.jackson.datatype.jsr310.JavaTimeModule;
import org.slf4j.Logger;
import org.slf4j.LoggerFactory;

/**
 * Main application class for the Poker Helper service
 * Uses Javalin for lightweight REST API with hexagonal architecture
 */
public class PokerHelperApplication {
    
    private static final Logger logger = LoggerFactory.getLogger(PokerHelperApplication.class);
    private static final int DEFAULT_PORT = 8080;
    
    public static void main(String[] args) {
        logger.info("Starting Poker Helper Application...");
        
        // Initialize application configuration
        ApplicationConfig config = new ApplicationConfig();
        
        // Configure Jackson for JSON handling
        ObjectMapper objectMapper = new ObjectMapper();
        objectMapper.registerModule(new JavaTimeModule());
        
        // Create Javalin app
        Javalin app = Javalin.create(javalinConfig -> {
            javalinConfig.jsonMapper(new JavalinJackson(objectMapper));
            javalinConfig.showJavalinBanner = false;
        });
        
        // Configure routes
        configureRoutes(app, config);
        
        // Start server
        int port = getPort(args);
        app.start(port);
        
        logger.info("🃏 Poker Helper Service started successfully on port {}", port);
        logger.info("📊 API available at: http://localhost:{}/api/poker", port);
        logger.info("🔍 Health check: http://localhost:{}/health", port);
        logger.info("📋 API info: http://localhost:{}/api/poker/info", port);
    }
    
    private static void configureRoutes(Javalin app, ApplicationConfig config) {
        // Health check endpoint
        app.get("/health", ctx -> {
            ctx.json(new HealthResponse("UP", "Poker Helper Service is running"));
        });
        
        // API base path
        app.get("/api", ctx -> {
            ctx.json(new ApiInfo(
                "Poker Helper - Probability Service",
                "1.0.0",
                "API for calculating poker hand probabilities using hexagonal architecture"
            ));
        });
        
        // Configure poker-specific routes through the HTTP adapter
        config.getPokerHttpAdapter().configureRoutes(app);
        
        // Global error handlers
        app.exception(Exception.class, (e, ctx) -> {
            logger.error("Unexpected error: ", e);
            ctx.status(500).json(new ErrorResponse("Internal server error", e.getMessage()));
        });
        
        app.error(404, ctx -> {
            ctx.json(new ErrorResponse("Not found", "The requested resource was not found"));
        });
    }
    
    private static int getPort(String[] args) {
        if (args.length > 0) {
            try {
                return Integer.parseInt(args[0]);
            } catch (NumberFormatException e) {
                logger.warn("Invalid port number provided: {}. Using default port {}", args[0], DEFAULT_PORT);
            }
        }
        return DEFAULT_PORT;
    }
    
    // Response DTOs
    public static class HealthResponse {
        public final String status;
        public final String message;
        
        public HealthResponse(String status, String message) {
            this.status = status;
            this.message = message;
        }
    }
    
    public static class ApiInfo {
        public final String name;
        public final String version;
        public final String description;
        
        public ApiInfo(String name, String version, String description) {
            this.name = name;
            this.version = version;
            this.description = description;
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
}
