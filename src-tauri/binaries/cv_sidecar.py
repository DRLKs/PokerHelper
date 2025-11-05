#!/usr/bin/env python3
"""
CV Sidecar - Computer Vision Agent (The "Eyes")
TODO: Implementar la lógica de visión por computador
"""

import sys
import json

def get_game_state():
    """TODO: Implementar captura y análisis de pantalla"""
    return {
        "status": "success",
        "pot": 0.0,
        "opponent_bet": 0.0,
        "my_cards": [],
        "board_cards": []
    }

def main():
    """Loop principal para comunicación stdin/stdout"""
    try:
        for line in sys.stdin:
            try:
                command = json.loads(line)
            except json.JSONDecodeError:
                continue

            if command.get("action") == "get_table_state":
                result = get_game_state()
            elif command.get("action") == "shutdown":
                result = {"status": "success", "message": "shutting down"}
                sys.stdout.write(json.dumps(result) + "\n")
                sys.stdout.flush()
                break
            else:
                result = {"status": "error", "message": "Unknown action"}

            sys.stdout.write(json.dumps(result) + "\n")
            sys.stdout.flush()
            
    except EOFError:
        pass
    except Exception as e:
        sys.stderr.write(f"CV Sidecar Error: {e}\n")
        sys.stderr.flush()

if __name__ == "__main__":
    main()
