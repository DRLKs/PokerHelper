# Poker Helper Service - Hexagonal Architecture with Javalin

## Overview

This is a lightweight poker probability calculation service built with **Javalin** and **Hexagonal Architecture**. The service provides REST endpoints to calculate poker hand probabilities and make strategic decisions.

## 🏗️ Architecture

### Hexagonal Architecture Layers:

```
┌─────────────────────────────────────────────────────────────┐
│                    Infrastructure Layer                     │
├─────────────────────────────────────────────────────────────┤
│  HTTP Adapters (Javalin) │  DTOs  │  Legacy Adapter        │
├─────────────────────────────────────────────────────────────┤
│                    Application Layer                        │
├─────────────────────────────────────────────────────────────┤
│     Use Cases (Ports)     │     Services                    │
├─────────────────────────────────────────────────────────────┤
│                     Domain Layer                            │
├─────────────────────────────────────────────────────────────┤
│  Card  │  Decision  │  PokerProbabilities                   │
└─────────────────────────────────────────────────────────────┘
```

### Directory Structure:
```
src/main/java/com/pokerhelper/
├── PokerHelperApplication.java              # Main application
├── application/
│   ├── ports/input/
│   │   └── PokerCalculatorUseCases.java     # Use case interface
│   └── services/
│       └── PokerCalculatorService.java      # Application service
├── domain/model/
│   ├── Card.java                           # Domain card model
│   ├── Decision.java                       # Domain decision model
│   └── PokerProbabilities.java             # Domain probabilities model
└── infrastructure/
    ├── adapters/
    │   ├── input/http/
    │   │   └── PokerHttpAdapter.java        # HTTP controller
    │   └── legacy/
    │       └── LegacyPokerCalculatorAdapter.java  # Legacy integration
    ├── config/
    │   └── ApplicationConfig.java           # DI configuration
    ├── dto/
    │   ├── CardDto.java                     # Card DTO
    │   ├── PokerCalculationRequest.java     # Request DTO
    │   └── PokerCalculationResponse.java    # Response DTO
    └── mappers/
        └── PokerDtoMapper.java              # DTO/Domain mapper
```

## Getting Started

### Prerequisites
- Java 17+
- Maven 3.8+

### Build & Run
```bash
# Compile
mvn clean compile

# Run tests
mvn test

# Create executable JAR
mvn clean package

# Run application
java -jar target/poker-probability-service-1.0.0.jar

# Or run with custom port
java -jar target/poker-probability-service-1.0.0.jar 9090
```

### Development Mode
```bash
# Run directly with Maven
mvn clean compile exec:java -Dexec.mainClass="com.pokerhelper.PokerHelperApplication"
```

## API Endpoints

### Base URLs
- **Health Check**: `GET /health`
- **API Info**: `GET /api`
- **Poker API Info**: `GET /api/poker/info`

### Poker Endpoints

#### Calculate Probabilities
```http
POST /api/poker/calculate
Content-Type: application/json

{
  "pocketCards": [
    {"suit": "S", "rank": 14},
    {"suit": "H", "rank": 13}
  ],
  "communityCards": [
    {"suit": "D", "rank": 12},
    {"suit": "C", "rank": 11}
  ],
  "numberOfOpponents": 3,
  "smallBlind": 10,
  "accumulatedBet": 50
}
```

## 🃏 Card Format

### Suits
- `S` = Spades (♠)
- `H` = Hearts (♥)
- `D` = Diamonds (♦)
- `C` = Clubs (♣)

### Ranks
- `1` or `14` = Ace
- `2-10` = Number cards
- `11` = Jack
- `12` = Queen
- `13` = King

## Why Javalin?

### Advantages for This Use Case:
- **Lightweight**: ~2MB vs Spring Boot's ~15MB
- **Fast Startup**: ~100ms vs Spring Boot's ~2-3s
- **Low Memory**: ~20-30MB vs Spring Boot's ~100-150MB
- **CPU Efficient**: Minimal overhead for intensive calculations
- **Simple**: No complex configuration
- **Embeddable**: Perfect for desktop applications

Perfect for embedding in desktop applications!