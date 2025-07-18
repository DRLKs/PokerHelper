# Tests for Javalin Poker Helper API

## Validation Tests for Pocket Cards

This document summarizes the comprehensive test suite for validating pocket card input in the Javalin-based Poker Helper REST API.

### Test Coverage

#### PokerHttpAdapterTest - Pocket Card Validation

✅ **shouldReturnBadRequestWhenPocketCardsExceedTwoCards**
- Tests the scenario when exactly 3 pocket cards are provided (original request)
- Validates that the API returns 400 Bad Request
- Confirms validation error message contains "Pocket cards must contain exactly 2 cards"

✅ **shouldReturnBadRequestWhenPocketCardsContainExactlyThreeCards**
- Edge case test for exactly 3 pocket cards
- Ensures validation works for the most common "too many cards" scenario
- Includes detailed response body logging for debugging

✅ **shouldReturnBadRequestWhenPocketCardsContainFourCards**
- Tests extreme case with 4 pocket cards
- Validates robustness of validation logic for multiple excess cards

✅ **shouldReturnBadRequestWhenPocketCardsLessThanTwoCards**
- Tests scenario with only 1 pocket card
- Ensures minimum card requirement is enforced

✅ **shouldReturnBadRequestWhenPocketCardsAreNull**
- Tests null pocket cards scenario
- Validates null safety of the validation logic

✅ **shouldReturnBadRequestWhenCommunityCardsExceedFiveCards**
- Tests community card validation (related validation)
- Ensures overall card validation consistency

✅ **shouldReturnBadRequestWhenOpponentsExceedEight**
- Tests opponent count validation
- Comprehensive request validation coverage

### Technical Implementation

#### Key Components Tested
- **PokerHttpAdapter**: REST endpoint validation and error handling
- **PokerCalculationRequest DTO**: Jakarta Bean Validation annotations
- **CardDto**: Card structure and JSON serialization/deserialization
- **Validator**: Hibernate Validator integration
- **Javalin**: HTTP request/response handling

#### Validation Constraints
```java
@Size(min = 2, max = 2, message = "Pocket cards must contain exactly 2 cards")
private final List<CardDto> pocketCards;
```

#### Error Response Format
```json
{
  "error": "Validation error",
  "message": "Pocket cards must contain exactly 2 cards",
  "timestamp": "2025-07-18T11:30:00Z"
}
```

### Test Results
All tests pass successfully, confirming that:
1. The Javalin REST API correctly validates pocket card input
2. Proper 400 Bad Request responses are returned for invalid requests
3. Validation messages are clear and helpful
4. The hexagonal architecture properly integrates validation at the adapter level
5. Mock verification ensures business logic is not called for invalid requests

### Architecture Benefits
- **Clean validation**: Input validation happens at the infrastructure layer
- **Proper error codes**: HTTP 400 for validation errors, not 500
- **Comprehensive coverage**: All edge cases for pocket card validation
- **Maintainable tests**: Clear test names and good separation of concerns
- **Integration testing**: Full HTTP request/response cycle testing
