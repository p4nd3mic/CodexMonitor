use super::service::{detect_intent, truncate};
use super::{CardState, CardType, DomainId};

#[test]
fn test_truncate_ascii() {
    assert_eq!(truncate("hello world", 5), "hello");
    assert_eq!(truncate("hi", 10), "hi");
}

#[test]
fn test_truncate_unicode() {
    assert_eq!(truncate("今日は良い天気", 3), "今日は");
    assert_eq!(truncate("Hello 🌍🌎🌏", 8), "Hello 🌍🌎");
}

#[test]
fn test_truncate_empty() {
    assert_eq!(truncate("", 10), "");
}

#[test]
fn test_card_state_transitions() {
    assert!(CardState::Pending.can_transition_to(&CardState::Processing));
    assert!(CardState::Processing.can_transition_to(&CardState::Complete));
    assert!(CardState::Processing.can_transition_to(&CardState::Error));
    assert!(CardState::Processing.can_transition_to(&CardState::AwaitingInput));
    assert!(CardState::AwaitingInput.can_transition_to(&CardState::Processing));
    assert!(CardState::AwaitingInput.can_transition_to(&CardState::Cancelled));
    assert!(CardState::Error.can_transition_to(&CardState::Processing));

    assert!(!CardState::Complete.can_transition_to(&CardState::Pending));
    assert!(!CardState::Cancelled.can_transition_to(&CardState::Processing));
}

#[test]
fn test_domain_detection() {
    let (card_type, domain, _) = detect_intent("had eggs for breakfast");
    assert_eq!(card_type, CardType::Meal);
    assert_eq!(domain, DomainId::Nutrition);

    let (card_type, domain, _) = detect_intent("watching Alien tonight");
    assert_eq!(card_type, CardType::MediaAdd);
    assert_eq!(domain, DomainId::Media);

    let (card_type, domain, _) = detect_intent("took $15 order");
    assert_eq!(card_type, CardType::DeliveryOrder);
    assert_eq!(domain, DomainId::Delivery);

    let (card_type, domain, _) = detect_intent("thinking about life");
    assert_eq!(card_type, CardType::Thought);
    assert_eq!(domain, DomainId::General);
}
