# Autonomous Poker Agent Architecture (`AGENTS.md`)

## 1. Overview

This document outlines the software architecture for an autonomous agent designed to play Texas Hold'em poker. The system is built using a hybrid, multi-language architecture to leverage the best-in-class tools for each required task:

1.  **High-Performance Computation (Rust):** Used for probability, game theory, and decision-making.
2.  **Computer Vision (Python):** Used for reading the game state from the screen.

The entire system is orchestrated by a **Tauri (Rust)** host application, which provides a User Interface (UI) and manages the two agent components.

## 2. Core Architecture

The system operates on a strict **Separation of Concerns** principle. The components are "agents" that communicate via a well-defined protocol, managed by the main Rust application.

* **Host (Tauri / Rust):** The main application process. It handles the UI, manages the game loop, and orchestrates communication between the "Brain" and the "Eyes."
* **The "Brain" (Rust Module):** A local Rust crate responsible for all logic. It calculates probabilities, pot odds, equity, and makes the final decision (Check, Bet, Fold).
* **The "Eyes" (Python Sidecar):** A standalone Python executable responsible *only* for computer vision. It reads the screen and reports the game state as a JSON object. It makes zero decisions.



### Communication Flow

1.  **Frontend (UI) -> Brain (Rust):** The user clicks "Start" in the UI. This triggers a `tauri::command` in the Rust backend.
2.  **Brain (Rust) -> Eyes (Python):** The Rust process spawns the Python executable as a **Sidecar**. It writes JSON commands (e.g., `{"action": "get_table_state"}`) to the sidecar's `stdin`.
3.  **Eyes (Python) -> Brain (Rust):** The Python process captures and analyzes the screen, then prints a JSON result (e.g., `{"pot": 100, "my_cards": ["As", "Kh"], ...}`) to its `stdout`.
4.  **Brain (Rust) -> Frontend (UI):** The Rust process reads the `stdout` line, parses the JSON, performs its calculations, and emits a Tauri event (e.g., `emit_all("agent_decision", ...)`), which the UI displays.

---

## 3. The "Eyes": Computer Vision Agent (Python Sidecar)

This component's sole responsibility is to answer the question: "What is the current state of the game board?"

* **Technology:** Python 3, `opencv-python`, `pytesseract` (for OCR), `PyInstaller` (for packaging).
* **Packaging:** The Python script and all its dependencies (`cv2`, etc.) are frozen into a single executable (`cv_sidecar.exe` or `cv_sidecar`) using PyInstaller.
* **Interface:** `stdin` for commands, `stdout` for JSON responses.

### Example Code (`cv_sidecar.py`)

```python
import sys
import json
import cv2  # OpenCV for image processing
# import your_ocr_and_template_matching_functions as cv_utils

def get_game_state():
    """
    Captures the screen, runs CV algorithms (template matching for
    cards, OCR for numbers), and builds a state object.
    
    Returns:
        dict: A dictionary representing the current game state.
    """
    # 1. Capture screen (implementation not shown)
    # screen_image = cv_utils.capture_game_window()
    
    # 2. Process image (example values)
    # my_cards = cv_utils.find_my_cards(screen_image) -> ["Ah", "Ad"]
    # pot_size = cv_utils.read_pot_size(screen_image) -> 12.50
    # opponent_bet = cv_utils.read_opponent_bet(screen_image) -> 5.0
    
    state = {
        "status": "success",
        "pot": 12.50,
        "opponent_bet": 5.0,
        "my_cards": ["Ah", "Ad"],
        "board_cards": ["5s", "5c", "9h"]
    }
    
    return state

def main():
    """
    Main loop to listen for commands on stdin and write results to stdout.
    This ensures the process stays alive and responsive.
    """
    try:
        for line in sys.stdin:
            # Parse the command from Rust
            try:
                command = json.loads(line)
            except json.JSONDecodeError:
                continue # Ignore malformed lines

            result = {}
            if command.get("action") == "get_table_state":
                result = get_game_state()
            else:
                result = {"status": "error", "message": "Unknown action"}

            # Write the JSON result to stdout
            # This is read by the Rust host process
            sys.stdout.write(json.dumps(result) + "\n")
            
            # CRITICAL: Flush stdout to ensure Rust receives the message immediately
            sys.stdout.flush()
            
    except EOFError:
        # Exit gracefully when stdin is closed
        pass
    except Exception as e:
        # Log errors to stderr (Rust can also read this)
        sys.stderr.write(f"CV Sidecar Error: {e}\n")
        sys.stderr.flush()

if __name__ == "__main__":
    main()
```

### Testing (Python)

* **Unit Tests:** Create a folder of saved screenshots (`/test_images/`). Write Pytest functions that pass these static images to your `cv_utils` functions and assert the correct JSON is returned.
* **Integration Test:** Write a script that launches `cv_sidecar` as a subprocess, writes JSON commands to its `stdin`, and asserts that valid JSON is received on its `stdout`.

---

## 4. The "Brain": Probability & Decision Agent (Rust Crate)

This component is the core logic. It is a stateful agent that understands the rules of poker, calculates odds, and makes decisions.

* **Technology:** Rust, as a local crate (`/src-tauri/crates/poker_agent`).
* **Interface:** `tauri::command` functions for the UI, and a `SidecarManager` struct to handle communication with the Python process.
* **Good Practice:** Adheres to the "Single Return per Function" pattern as requested.

### Example Code (`src-tauri/src/poker_agent.rs`)

```rust
use serde::{Deserialize, Serialize};
use tauri::api::process::{Command, CommandEvent};
use std::io::{BufRead, BufReader, Write};
use std::sync::Mutex;

// --- 1. Data Structures ---

#[derive(Deserialize, Debug)]
struct CvState {
    status: String,
    pot: f64,
    opponent_bet: f64,
    my_cards: Vec<String>,
    board_cards: Vec<String>,
}

#[derive(Serialize, Debug, PartialEq)]
enum PokerAction {
    Fold,
    Check,
    Call,
    Bet(f64),
}

// Global state for the sidecar process
pub struct SidecarManager {
    child: Mutex<Option<tauri::api::process::Child>>,
    // We would also have a sender/receiver to communicate with the reader thread
}

// --- 2. Probability Module ---

/// Calculates the agent's equity (win probability)
/// This is a complex function, stubbed out for this example.
///
/// # Arguments
/// * `my_cards` - A slice of card strings (e.g., ["Ah", "Kd"])
/// * `board_cards` - A slice of board card strings
///
/// # Returns
/// * `Result<f64, String>` - The win equity (0.0 to 1.0) or an error.
fn calculate_equity(my_cards: &[String], board_cards: &[String]) -> Result<f64, String> {
    // In a real implementation, this would involve complex Monte Carlo
    // simulations or lookup tables using a crate like 'poker-eval'.
    //
    // This function adheres to the single-return pattern.
    
    let result: Result<f64, String>;
    
    if my_cards.len() != 2 {
        result = Err("Must have exactly 2 hole cards".to_string());
    } else {
        // ... complex simulation logic ...
        // Faking a result for this example
        let equity = if my_cards.contains(&"Ah".to_string()) { 0.85 } else { 0.45 };
        result = Ok(equity);
    }
    
    result // Single return
}

// --- 3. Decision Module ---

/// The main decision-making function for the agent.
/// Adheres to the single-return-per-function pattern.
///
/// # Arguments
/// * `state` - The game state reported by the CV agent.
///
/// # Returns
/// * `Result<PokerAction, String>` - The calculated poker action.
fn make_decision(state: CvState) -> Result<PokerAction, String> {
    let final_action: Result<PokerAction, String>;

    // 1. Calculate Pot Odds
    let to_call = state.opponent_bet;
    let pot_size = state.pot + to_call;
    let pot_odds = to_call / pot_size; // e.g., 0.33 (33%)

    // 2. Calculate Equity
    match calculate_equity(&state.my_cards, &state.board_cards) {
        Ok(equity) => {
            // 3. Make Decision (simplified logic)
            if equity > pot_odds {
                // Good odds to call
                if to_call > 0.0 {
                    final_action = Ok(PokerAction::Call);
                } else {
                    // No bet to call, so we check or bet
                    final_action = Ok(PokerAction::Check);
                }
            } else {
                // Bad odds
                if to_call > 0.0 {
                    final_action = Ok(PokerAction::Fold);
                } else {
                    // Free card
                    final_action = Ok(PokerAction::Check);
                }
            }
        }
        Err(e) => {
            final_action = Err(format!("Failed to calculate equity: {}", e));
        }
    }

    final_action // Single return
}

// --- 4. Tauri Command (Glue Code) ---

/// This is the main function called by the frontend.
/// It triggers the CV, gets the state, and runs the decision logic.
#[tauri::command]
pub async fn get_agent_action(
    manager: tauri::State<'_, SidecarManager>,
) -> Result<PokerAction, String> {
    
    // This is simplified. In reality, you would send the command
    // to the sidecar via stdin and listen for the response on stdout.
    // This requires a more complex async setup with channels.
    
    // 1. Send command to Python sidecar (simplified)
    // child.write_stdin(b"{\"action\": \"get_table_state\"}\n").unwrap();
    
    // 2. Receive mock state (in a real app, this comes from stdout)
    let mock_json_response = r#"
    {
        "status": "success",
        "pot": 10.0,
        "opponent_bet": 5.0,
        "my_cards": ["Ah", "Kd"],
        "board_cards": ["5s", "8c", "Js"]
    }
    "#;
    
    let state: CvState = serde_json::from_str(mock_json_response)
        .map_err(|e| format!("Failed to parse CV response: {}", e))?;

    // 3. Pass state to decision module
    let decision = make_decision(state);
    
    decision // Single return
}
```

### Testing (Rust)

* **Unit Tests:** Use Rust's built-in test framework (`#[cfg(test)]`).
    * Test `calculate_equity` with known hand matchups.
    * Test `make_decision` by providing it with mock `CvState` structs and asserting the correct `PokerAction` is returned (e.g., "Given state X, agent MUST fold").

---

## 5. Best Practices for the AI Agent

1.  **Modularity (Eyes vs. Brain):** The architecture already enforces this. The CV agent (Python) must be "dumb" — it only reports what it sees. The Rust agent is "blind" — it only knows the JSON it receives. This allows you to improve the CV model without touching the poker logic, and vice-versa.
2.  **Robustness and Error Handling:** The CV agent will fail. It will misread cards or numbers. The Rust agent *must* be programmed defensively.
    * Use `Result` and `Option` everywhere.
    * If the CV agent returns `{"status": "error"}` or unparseable data, the Rust agent should have a safe default action (e.g., `Check` or `Fold`).
    * Implement timeouts for CV requests.
3.  **State Management:** The Rust "Brain" is the *only* component that should store game state (e.g., opponent's past actions, pot history). The Python "Eyes" should be stateless (input: screen image, output: current state).
4.  **Performance:** Computer Vision is slow.
    * The Rust agent should *only* request a screen-read from Python when it is the agent's turn to act. Do not scan on every frame.
    * Use `async` in Rust to avoid blocking the UI while waiting for the Python process.
5.  **Configuration:** Do not hardcode screen coordinates in the Python script. Create a `config.json` that defines the pixel bounding boxes for "pot," "my_cards," etc. The Python script should load this config on startup.
6.  **Coding Standards:** Adhere to language-specific best practices (`clippy` for Rust, `flake8` for Python). Enforce specific patterns like "single return per function" if it aids team clarity and maintainability.
7.  **Ethical Use:** This agent should only be used in environments where bots are permitted. Using it against real-money players on platforms that forbid it is unethical and against their Terms of Service.