use rust_api_light_simple::items::ItemPayload;

#[test]
fn accepts_non_empty_name() {
    let payload = ItemPayload {
        name: "Notebook".to_string(),
        description: None,
    };

    assert!(payload.validate().is_ok());
}

#[test]
fn rejects_blank_name() {
    let payload = ItemPayload {
        name: "   ".to_string(),
        description: None,
    };

    let error = payload.validate().expect_err("blank name should fail");
    assert_eq!(error.to_string(), "name is required");
}
