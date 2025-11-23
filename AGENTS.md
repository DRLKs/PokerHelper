# Autonomous Poker Agent Architecture (`AGENTS.md`)

## 1. Overview

This document outlines the software architecture for an autonomous agent designed to play Texas Hold'em poker. The system is built using a hybrid, multi-language architecture to leverage the best-in-class tools for each required task:

1.  **High-Performance Computation (Rust):** Used for probability, game theory, and decision-making.
2.  **Computer Vision (Python):** Used for reading the game state from the screen.
3.  **User Interface (React/TypeScript):** Provides control and visualization using a Clean Architecture approach.

The entire system is orchestrated by a **Tauri (Rust)** host application.

## 2. Core Architecture

The system operates on a strict **Separation of Concerns** principle. The components are "agents" that communicate via a well-defined protocol, managed by the main Rust application.

* **Host (Tauri / Rust):** The main application process. It handles the UI, manages the game loop, and orchestrates communication between the "Brain" and the "Eyes."
* **The "Brain" (Rust Module):** A local Rust crate responsible for all logic. It calculates probabilities, pot odds, equity, and makes the final decision (Check, Bet, Fold).
* **The "Eyes" (Python Sidecar):** A standalone Python executable responsible *only* for computer vision. It reads the screen and reports the game state as a JSON object. It makes zero decisions.
* **The "Face" (Frontend):** A React application that visualizes the state and sends user commands to the Host.

### Communication Flow

1.  **Frontend (UI) -> Brain (Rust):** The user triggers an action (e.g., "Start"). This calls a Repository method which triggers a `tauri::command`.
2.  **Brain (Rust) -> Eyes (Python):** The Rust process spawns the Python executable as a **Sidecar** and writes JSON commands to `stdin`.
3.  **Eyes (Python) -> Brain (Rust):** The Python process captures/analyzes the screen and prints JSON results to `stdout`.
4.  **Brain (Rust) -> Frontend (UI):** The Rust process parses the result, makes a decision, and emits a Tauri event to the Frontend Adapter, which updates the React State.

---

## 3. The "Eyes": Computer Vision Agent (Python Sidecar)

This component's sole responsibility is to answer the question: "What is the current state of the game board?"

* **Technology:** Python 3, `opencv-python`, `pytesseract` (for OCR), `PyInstaller` (for packaging).
* **Packaging:** The Python script and all its dependencies are frozen into a single executable (`cv_sidecar`) using PyInstaller.
* **Interface:** `stdin` for commands, `stdout` for JSON responses.
* **Testing Strategy:** Unit tests should run against a folder of static screenshots to verify that the CV algorithms correctly identify cards and numbers before integration.

---

## 4. The "Brain": Probability & Decision Agent (Rust Crate)

This component is the core logic. It is a stateful agent that understands the rules of poker, calculates odds, and makes decisions.

* **Technology:** Rust, as a local crate (`/src-tauri/crates/poker_agent`).
* **Responsibility:** It manages the `SidecarManager` to talk to Python and contains the math logic for equity calculation.
* **Interface:** Exposes `tauri::command` functions for the UI.
* **Testing Strategy:** Unit tests for mathematical accuracy (equity calculations) and state machine logic (ensuring correct decisions based on specific game states).

---

## 5. The "Face": Frontend Architecture (React + Clean Arch)

The frontend is built using **React** (via Vite) and styled with **Tailwind CSS**. To ensure scalability and decouple the UI from the specific Tauri implementation, we apply **Clean Architecture principles**.

### Architectural Layers

1.  **Domain (Entities):** TypeScript interfaces that define the data (e.g., `Card`, `GameState`, `Decision`). These are pure and have no dependencies on React or Tauri.
2.  **Application (Use Cases):** Hooks or Service definitions that describe *what* can be done (e.g., `useStartAgent`, `useGameState`).
3.  **Infrastructure (Adapters):** The implementation of the calls to the Rust backend using `tauri/api`. This is the *only* layer that is aware of the Tauri environment.
4.  **Presentation (UI):** React components styled with Tailwind. They only receive data and trigger actions defined in the Application layer, keeping them purely presentational.

### Directory Structure Plan

```text
src/
├── domain/                 # Enterprise Business Rules (Types/Interfaces)
├── infrastructure/         # Interface Adapters (Tauri API calls & Event Listeners)
├── application/            # Application Business Rules (Hooks, State Management)
└── presentation/           # Frameworks & Drivers (React Components & Pages)
```

## 6. Development Environment

* **Package Manager:** We use `pnpm` for faster and more efficient dependency management.
* **Documentation:** Project documentation is located in `docs/`.