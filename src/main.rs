extern crate base64;
extern crate oauth2;
extern crate rand;
extern crate url;
extern crate regex;

mod login;

fn main() {
    let gmail_login = login::get_login();
    login::connect(gmail_login);
}
