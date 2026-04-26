mod common;

use argon2::{Argon2, PasswordHash, PasswordHasher, PasswordVerifier};
use argon2::password_hash::{rand_core::OsRng, SaltString};

#[tokio::test]
async fn test_password_hash_and_verify() {
    let password = b"supersecret123";
    let salt = SaltString::generate(&mut OsRng);
    let hash = Argon2::default()
        .hash_password(password, &salt)
        .expect("hash failed")
        .to_string();

    let parsed = PasswordHash::new(&hash).expect("parse failed");
    assert!(Argon2::default().verify_password(password, &parsed).is_ok());
    assert!(Argon2::default().verify_password(b"wrongpassword", &parsed).is_err());
}
