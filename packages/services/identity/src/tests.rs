use crate::config::Config;
use crate::models::Claims;
use crate::routes::generate_token_pair;
use schoolccb_common::auth::require_role;
use uuid::Uuid;

fn make_admin_claims() -> Claims {
    Claims {
        sub: Uuid::new_v4().to_string(),
        role: "GerenteGeneral".into(),
        name: "Test Admin".into(),
        email: "admin@test.cl".into(),
        exp: 9999999999,
        iat: 0,
        school_id: None,
        corporation_id: None,
        admin_type: None,
    }
}

fn make_user_claims() -> Claims {
    Claims {
        sub: Uuid::new_v4().to_string(),
        role: "Profesor".into(),
        name: "Test User".into(),
        email: "user@test.cl".into(),
        exp: 9999999999,
        iat: 0,
        school_id: None,
        corporation_id: None,
        admin_type: None,
    }
}

#[test]
fn test_generate_token_pair_creates_valid_jwt() {
    let config = Config {
        database_url: "postgres://localhost/test".into(),
        host: "0.0.0.0".into(),
        port: 3001,
        jwt_secret: "test-secret-key-for-testing".into(),
        internal_api_secret: "test-secret".into(),
    };
    let user_id = Uuid::new_v4();
    let (token, claims) =
        generate_token_pair(&config, user_id, "GerenteGeneral", "Test", "test@test.cl", None, None, None)
            .expect("should generate token pair");

    assert!(!token.is_empty());
    assert_eq!(claims.sub, user_id.to_string());
    assert_eq!(claims.role, "GerenteGeneral");
    assert_eq!(claims.name, "Test");
    assert_eq!(claims.email, "test@test.cl");
}

#[test]
fn test_require_role_returns_ok_for_admin() {
    let claims = make_admin_claims();
    assert!(require_role(&claims, "Profesor").is_ok());
}

#[test]
fn test_require_role_returns_err_for_wrong_role() {
    let claims = make_user_claims();
    let result = require_role(&claims, "Administrador");
    assert!(result.is_err());
}
