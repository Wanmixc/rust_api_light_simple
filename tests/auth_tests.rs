use rust_api_light_simple::auth::{hash_password, verify_password, Claims};

#[test]
fn hash_and_verify_password_roundtrip() {
    let hash = hash_password("my-secret-password").expect("should hash");
    assert!(hash.starts_with("$argon2"));

    verify_password("my-secret-password", &hash).expect("should verify");
}

#[test]
fn verify_wrong_password_fails() {
    let hash = hash_password("correct-password").expect("should hash");
    let result = verify_password("wrong-password", &hash);
    assert!(result.is_err());
}

#[test]
fn test_claims_serde_roundtrip() {
    use uuid::Uuid;
    let id = Uuid::new_v4();
    let claims = Claims {
        sub: id,
        exp: 9999999999,
        iat: 1000000000,
    };

    let json = serde_json::to_string(&claims).expect("serialize");
    let parsed: Claims = serde_json::from_str(&json).expect("deserialize");
    assert_eq!(parsed.sub, id);
    assert_eq!(parsed.exp, 9999999999);
    assert_eq!(parsed.iat, 1000000000);
}
