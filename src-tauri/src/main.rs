// Prevents additional console window on Windows
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use poker_agent::core::card::Card;
use poker_agent::core::community_cards::CommunityCards;
use poker_agent::core::hand::Hand;
use poker_agent::probability::calculate_equity;

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

    // Default iterations to 10000 for now, could be a parameter
    calculate_equity(hand, community, num_opponents, 10000)
}

fn main() {
    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![calculate_equity_command])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
