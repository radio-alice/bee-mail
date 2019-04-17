extern crate base64;
extern crate imap;
extern crate native_tls;

use native_tls::TlsConnector;
use std::env;
use dotenv;
mod auth;

pub struct GmailOAuth2 {
    pub user: String,
    pub access_token: String,
}

impl imap::Authenticator for GmailOAuth2 {
    type Response = String;
    #[allow(unused_variables)]
    fn process(&self, data: &[u8]) -> Self::Response {
        format!(
            "user={}\x01auth=Bearer {}\x01\x01",
            self.user, self.access_token
        )
    }
}

// call to get new oauth variables
pub fn get_login() -> GmailOAuth2 {
    dotenv::dotenv().expect("Failed to read .env file");
    let user = env::var("USERNAME").expect("USERNAME not found");
    let init_oauth = auth::get_init_oauth();

    let access_token = match auth::get_new_access_code(init_oauth) {
        Some(access_token) => access_token,
        None => panic!("no access token returned"),
    };

    GmailOAuth2 {
        user,
        access_token,
    }
}

pub fn connect(gmail_auth: GmailOAuth2) {
    let domain = "https://mail.google.com";
    let port = 993;
    let socket_addr = (domain, port);
    let ssl_connector = TlsConnector::builder().build().unwrap();
    let client = imap::connect(socket_addr, domain, &ssl_connector).unwrap();

    let mut imap_session = match client.authenticate("XOAUTH2", &gmail_auth) {
        Ok(c) => c,
        Err((e, _unauth_client)) => {
            println!("error authenticating: {}", e);
            return;
        }
    };

    match imap_session.select("INBOX") {
        Ok(mailbox) => println!("{}", mailbox),
        Err(e) => println!("Error selecting INBOX: {}", e),
    };

    match imap_session.fetch("2", "body[text]") {
        Ok(msgs) => {
            for msg in &msgs {
                print!("{:?}", msg);
            }
        }
        Err(e) => println!("Error Fetching email 2: {}", e),
    };

    imap_session.logout().unwrap();
}