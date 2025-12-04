//! Tests for dice rolling functionality

use dndgamerolls::dice3d::types::DiceType;

#[test]
fn test_dice_type_max_values() {
    assert_eq!(DiceType::D4.max_value(), 4);
    assert_eq!(DiceType::D6.max_value(), 6);
    assert_eq!(DiceType::D8.max_value(), 8);
    assert_eq!(DiceType::D10.max_value(), 10);
    assert_eq!(DiceType::D12.max_value(), 12);
    assert_eq!(DiceType::D20.max_value(), 20);
}

#[test]
fn test_dice_type_names() {
    assert_eq!(DiceType::D4.name(), "D4");
    assert_eq!(DiceType::D6.name(), "D6");
    assert_eq!(DiceType::D8.name(), "D8");
    assert_eq!(DiceType::D10.name(), "D10");
    assert_eq!(DiceType::D12.name(), "D12");
    assert_eq!(DiceType::D20.name(), "D20");
}

#[test]
fn test_dice_type_parse_valid() {
    assert_eq!(DiceType::parse("d4"), Some(DiceType::D4));
    assert_eq!(DiceType::parse("D4"), Some(DiceType::D4));
    assert_eq!(DiceType::parse("d6"), Some(DiceType::D6));
    assert_eq!(DiceType::parse("D6"), Some(DiceType::D6));
    assert_eq!(DiceType::parse("d8"), Some(DiceType::D8));
    assert_eq!(DiceType::parse("d10"), Some(DiceType::D10));
    assert_eq!(DiceType::parse("d12"), Some(DiceType::D12));
    assert_eq!(DiceType::parse("d20"), Some(DiceType::D20));
    assert_eq!(DiceType::parse("D20"), Some(DiceType::D20));
}

#[test]
fn test_dice_type_parse_invalid() {
    assert_eq!(DiceType::parse("d3"), None);
    assert_eq!(DiceType::parse("d100"), None);
    assert_eq!(DiceType::parse("invalid"), None);
    assert_eq!(DiceType::parse(""), None);
}

#[test]
fn test_dice_type_equality() {
    assert_eq!(DiceType::D20, DiceType::D20);
    assert_ne!(DiceType::D20, DiceType::D6);
}
