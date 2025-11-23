use crate::core::card::Card;

/// Where 11 = J, 12 = Q, 13 = K, 14 = A
pub const ALL_RANKS: [u8; 13] = [2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14];

pub const ALL_SUITS: [char; 4] = ['h', 'd', 'c', 's'];

pub const ALL_CARDS: [Card; 52] = [
    // Hearts
    Card::new('h', 2),
    Card::new('h', 3),
    Card::new('h', 4),
    Card::new('h', 5),
    Card::new('h', 6),
    Card::new('h', 7),
    Card::new('h', 8),
    Card::new('h', 9),
    Card::new('h', 10),
    Card::new('h', 11),
    Card::new('h', 12),
    Card::new('h', 13),
    Card::new('h', 14),
    // Diamonds ♦
    Card::new('d', 2),
    Card::new('d', 3),
    Card::new('d', 4),
    Card::new('d', 5),
    Card::new('d', 6),
    Card::new('d', 7),
    Card::new('d', 8),
    Card::new('d', 9),
    Card::new('d', 10),
    Card::new('d', 11),
    Card::new('d', 12),
    Card::new('d', 13),
    Card::new('d', 14),
    // Clubs
    Card::new('c', 2),
    Card::new('c', 3),
    Card::new('c', 4),
    Card::new('c', 5),
    Card::new('c', 6),
    Card::new('c', 7),
    Card::new('c', 8),
    Card::new('c', 9),
    Card::new('c', 10),
    Card::new('c', 11),
    Card::new('c', 12),
    Card::new('c', 13),
    Card::new('c', 14),
    // Spades
    Card::new('s', 2),
    Card::new('s', 3),
    Card::new('s', 4),
    Card::new('s', 5),
    Card::new('s', 6),
    Card::new('s', 7),
    Card::new('s', 8),
    Card::new('s', 9),
    Card::new('s', 10),
    Card::new('s', 11),
    Card::new('s', 12),
    Card::new('s', 13),
    Card::new('s', 14),
];
