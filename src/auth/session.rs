use rand::Rng;

#[derive(Debug, Clone)]
pub struct Session {
    pub user_id: i64,
    pub pub_id: i64,
    pub csrf_token: String,
    pub must_change_password: bool,
}

pub fn generate_csrf_token() -> String {
    let bytes: [u8; 32] = rand::thread_rng().gen();
    hex::encode(bytes)
}

pub fn csrf_token_field(token: &str) -> maud::Markup {
    maud::html! {
        input type="hidden" name="csrf_token" value=(token);
    }
}
