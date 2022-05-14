
#[get("/<redir_code>")]
pub fn route_meme(redir_code: String) -> String {
    format!("REDIR: {}", redir_code)
}