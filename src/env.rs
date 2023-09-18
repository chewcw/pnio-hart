use std::env;

pub const HTTP_SERVER: &str = "HTTP_SERVER";
pub const HTTP_PORT: &str = "HTTP_PORT";

pub fn init() {
    if env::var(HTTP_SERVER).is_err() {
        env::set_var(HTTP_SERVER, "0.0.0.0");
    }
    if env::var(HTTP_PORT).is_err() {
        env::set_var(HTTP_PORT, "8081");
    }
}
