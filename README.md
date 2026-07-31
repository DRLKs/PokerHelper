# PokerHelper

PokerHelper is an in-development Texas Hold'em decision-support desktop application. It estimates hand equity from the current table state and compares that equity with the price of a call.

> Development status: the project is not ready for production use and no installer is currently provided.

## Current capabilities

The user can enter:

- Two hole cards.
- Up to five community cards.
- The number of opponents remaining in the hand.
- The current pot size.
- The amount required to call.

The application calculates estimated equity and, when betting information is available, presents the required equity and a simple call-or-fold indication.

Manual input is the primary workflow today. Computer vision support is under active development and is intended to read the same state from an online poker window without moving decision logic into the vision service.

## Architecture

PokerHelper uses a hybrid architecture:

- **Tauri / Rust host:** orchestrates the application and exposes commands to the frontend.
- **Rust poker agent:** owns poker rules, probability, equity, and decision logic.
- **Python computer-vision sidecar:** reads the screen and returns structured game state only.
- **React / TypeScript frontend:** provides a minimal interface organized with Clean Architecture boundaries.

See [AGENTS.md](./AGENTS.md) for the full system architecture and [Frontend design context](./docs/FRONTEND_DESIGN.md) for the current UI direction.

## Development

Requirements:

- Node.js
- `pnpm`
- Rust and the Tauri prerequisites for your operating system
- Python 3 for computer-vision development

Run the frontend locally:

```bash
cd poker-helper
pnpm install
pnpm dev
```

Create a production frontend build:

```bash
cd poker-helper
pnpm build
```

Run the Tauri application:

```bash
cd poker-helper
pnpm tauri dev
```

## Computer vision

The vision service and its dataset notes live in [`poker-helper/computer_vision`](./poker-helper/computer_vision). The current training dataset was sourced from [Roboflow Universe](https://universe.roboflow.com/poker-nnnrc/poker-cards-nesyh/dataset/6).

## Intended use

This project is currently a learning and experimentation environment for poker mathematics, Rust, computer vision, and desktop application architecture. Calculated results are estimates and should not be treated as financial advice.
