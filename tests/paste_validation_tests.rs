use rust_api_light_simple::pastes::{generate_paste_id, PastePayload, PASTE_ID_LEN};

#[test]
fn accepts_non_empty_content() {
    let payload = PastePayload {
        content: "hello world".to_string(),
    };

    assert!(payload.validate().is_ok());
}

#[test]
fn rejects_blank_content() {
    let payload = PastePayload {
        content: "   ".to_string(),
    };

    let error = payload.validate().expect_err("blank content should fail");
    assert_eq!(error.to_string(), "content is required");
}

#[test]
fn generated_paste_id_is_case_sensitive_five_character_token() {
    let id = generate_paste_id();

    assert_eq!(id.len(), PASTE_ID_LEN);
    assert!(id.chars().all(|ch| ch.is_ascii_alphanumeric()));
}
