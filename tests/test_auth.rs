mod common;

#[tokio::test]
async fn test_placeholder_auth() {
    // TODO: implement auth tests
    // - test that unauthenticated requests to /admin/* are redirected to /auth/login
    // - test that login with valid credentials stores session and redirects
    // - test that login with invalid credentials returns error
    // - test that must_change_password flag redirects to change-password page
}
