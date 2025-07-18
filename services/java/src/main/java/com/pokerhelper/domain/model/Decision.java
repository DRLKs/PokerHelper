package com.pokerhelper.domain.model;

/**
 * Domain model for poker decision
 */
public class Decision {
    
    public static final int FOLD = 0;
    public static final int CHECK = 1;
    public static final int CALL = 2;
    public static final int RAISE = 3;
    public static final int BET = 4;
    public static final int ALL_IN = 5;

    private final int action;
    private final int betAmount;
    private final String description;

    public Decision(int action, int betAmount) {
        this.action = action;
        this.betAmount = betAmount;
        this.description = getActionDescription(action);
    }

    public int getAction() {
        return action;
    }

    public int getBetAmount() {
        return betAmount;
    }

    public String getDescription() {
        return description;
    }

    public String getActionName() {
        return switch (action) {
            case FOLD -> "FOLD";
            case CHECK -> "CHECK";
            case CALL -> "CALL";
            case RAISE -> "RAISE";
            case BET -> "BET";
            case ALL_IN -> "ALL_IN";
            default -> "UNKNOWN";
        };
    }

    private String getActionDescription(int action) {
        return switch (action) {
            case FOLD -> "Fold your cards";
            case CHECK -> "Check (no bet)";
            case CALL -> "Call the current bet";
            case RAISE -> "Raise the bet";
            case BET -> "Place a bet";
            case ALL_IN -> "Go all in";
            default -> "Unknown action";
        };
    }

    @Override
    public String toString() {
        return "Decision{" +
                "action=" + getActionName() +
                ", betAmount=" + betAmount +
                ", description='" + description + '\'' +
                '}';
    }
}
