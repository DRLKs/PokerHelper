// Prevents additional console window on Windows
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use poker_agent::core::card::Card;
use poker_agent::core::community_cards::CommunityCards;
use poker_agent::core::hand::Hand;
use poker_agent::probability::calculate_equity;

const ITERATIONS: u16 = 10000;

fn parse_card(card_str: &str) -> Result<Card, String> {
    if card_str.len() < 2 || card_str.len() > 3 {
        return Err(format!("Invalid card string length: {}", card_str));
    }

    let (rank_str, suit_char) = if card_str.len() == 3 {
        (&card_str[0..2], card_str.chars().nth(2).unwrap())
    } else {
        (&card_str[0..1], card_str.chars().nth(1).unwrap())
    };

    let rank = match rank_str {
        "2" => 2,
        "3" => 3,
        "4" => 4,
        "5" => 5,
        "6" => 6,
        "7" => 7,
        "8" => 8,
        "9" => 9,
        "10" | "T" => 10,
        "J" => 11,
        "Q" => 12,
        "K" => 13,
        "A" => 14,
        _ => return Err(format!("Invalid rank: {}", rank_str)),
    };

    let suit = match suit_char {
        'h' | 'd' | 'c' | 's' => suit_char,
        _ => return Err(format!("Invalid suit: {}", suit_char)),
    };

    Ok(Card::new(suit, rank))
}

#[tauri::command]
async fn calculate_equity_command(
    my_cards: Vec<String>,
    community_cards: Vec<String>,
    num_opponents: u8,
) -> Result<f64, String> {
    if my_cards.len() != 2 {
        return Err("Hand must have exactly 2 cards".to_string());
    }

    let card1 = parse_card(&my_cards[0])?;
    let card2 = parse_card(&my_cards[1])?;
    let hand = Hand::new([card1, card2]);

    let mut community_cards_vec = Vec::new();
    for card_str in community_cards {
        community_cards_vec.push(parse_card(&card_str)?);
    }
    let community = CommunityCards::new(community_cards_vec);

    calculate_equity(hand, community, num_opponents, ITERATIONS)
}

use std::process::{Command, Stdio};
use std::io::{Write, BufReader, BufRead};
use std::sync::Mutex;
use tauri::State;

struct SidecarState {
    child: Mutex<Option<std::process::Child>>,
}

#[tauri::command]
async fn start_sidecar(state: State<'_, SidecarState>) -> Result<String, String> {
    let mut child_guard = state.child.lock().map_err(|e| e.to_string())?;
    
    if child_guard.is_some() {
        return Ok("Sidecar already running".to_string());
    }

    // Note: In a real production build, you'd use tauri::api::process::Command
    // But for dev with a python script, we'll spawn python directly
    // Adjust path as needed relative to CWD
    // src-tauri is in poker-helper/src-tauri, and computer_vision is in ../computer_vision
    // Also use the venv python if available
    let python_cmd = if std::path::Path::new("../../.venv/bin/python").exists() {
        "../../.venv/bin/python"
    } else {
        "python3"
    };

    let child = Command::new(python_cmd)
        .arg("../computer_vision/cv_sidecar.py")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn()
        .map_err(|e| format!("Failed to spawn sidecar: {}", e))?;

    *child_guard = Some(child);
    Ok("Sidecar started".to_string())
}

#[tauri::command]
async fn list_windows(state: State<'_, SidecarState>) -> Result<Vec<String>, String> {
    let mut child_guard = state.child.lock().map_err(|e| e.to_string())?;
    
    if let Some(child) = child_guard.as_mut() {
        let stdin = child.stdin.as_mut().ok_or("Failed to open stdin")?;
        let msg = serde_json::json!({ "type": "list_windows" }).to_string();
        writeln!(stdin, "{}", msg).map_err(|e| e.to_string())?;

        let stdout = child.stdout.as_mut().ok_or("Failed to open stdout")?;
        let mut reader = BufReader::new(stdout);
        let mut line = String::new();
        reader.read_line(&mut line).map_err(|e| e.to_string())?;

        let response: serde_json::Value = serde_json::from_str(&line).map_err(|e| e.to_string())?;
        
        if let Some(windows) = response.get("windows") {
             let w: Vec<String> = serde_json::from_value(windows.clone()).unwrap_or_default();
             return Ok(w);
        }
        Err("Invalid response from sidecar".to_string())
    } else {
        Err("Sidecar not running".to_string())
    }
}

fn main() {
    tauri::Builder::default()
        .manage(SidecarState { child: Mutex::new(None) })
        .invoke_handler(tauri::generate_handler![
            calculate_equity_command,
            start_sidecar,
            list_windows
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
