use std::env;
use dotenv;

use url::Url;
use std::io::{BufRead, BufReader, Write};
use std::net::TcpListener;

use oauth2::prelude::*;
use oauth2::{
    AccessToken,
    AuthorizationCode,
    AuthUrl,
    ClientId,
    ClientSecret,
    CsrfToken,
    RedirectUrl,
    Scope,
    TokenResponse,
    TokenUrl
};
use oauth2::basic::BasicClient;

pub struct OAuth2 {
    pub client_id: String,
    pub client_secret: String,
}

pub fn get_old_access_code() -> String {
    dotenv::dotenv().expect("Failed to read .env file");
    env::var("ACCESS_CODE").expect("ACCESS_CODE not found")
}

pub fn get_init_oauth() -> OAuth2 {
    dotenv::dotenv().expect("Failed to read .env file");
    let client_id = env::var("CLIENT_ID").expect("CLIENT_ID not found");
    let client_secret = env::var("CLIENT_SECRET").expect("CLIENT_SECRET not found");

    OAuth2 {
        client_id,
        client_secret,
    }
}

// main.rs passes in ID and secret, get_access_code() returns access code
pub fn get_new_access_code (oauth_vars: OAuth2) -> Option<String> {
    let google_client_id = ClientId::new(
        oauth_vars.client_id,
    );
    let google_client_secret = ClientSecret::new(
        oauth_vars.client_secret,
    );
    let auth_url = AuthUrl::new(
        Url::parse("https://accounts.google.com/o/oauth2/v2/auth")
            .expect("Invalid authorization endpoint URL"),
    );
    let token_url = TokenUrl::new(
        Url::parse("https://www.googleapis.com/oauth2/v3/token")
            .expect("Invalid token endpoint URL"),
    );

    // Set up the config for the Google OAuth2 process.
    let client = BasicClient::new(
        google_client_id,
        Some(google_client_secret),
        auth_url,
        Some(token_url),
    )
    // This requests access to gmail specifically
    .add_scope(Scope::new(
        "https://mail.google.com/".to_string(),
    ))

    // This example will be running its own server at localhost:8080.
    // See below for the server implementation.
    .set_redirect_url(RedirectUrl::new(
        Url::parse("http://localhost:8080").expect("Invalid redirect URL"),
    ));

    // Generate the authorization URL to which we'll redirect the user.
    let (authorize_url, csrf_state) = client.authorize_url(CsrfToken::new_random);

    println!(
        "Open this URL in your browser:\n{}\n",
        authorize_url.to_string()
    );

    // A very naive implementation of the redirect server.
    let listener = TcpListener::bind("127.0.0.1:8080").unwrap();
    for stream in listener.incoming() {
        if let Ok(mut stream) = stream {
            let state;
            let code;
            {
                let mut reader = BufReader::new(&stream);

                let mut request_line = String::new();
                reader.read_line(&mut request_line).unwrap();

                let redirect_url = request_line.split_whitespace().nth(1).unwrap();
                let url = Url::parse(&("http://localhost".to_string() + redirect_url)).unwrap();

                let code_pair = url
                    .query_pairs()
                    .find(|pair| {
                        let &(ref key, _) = pair;
                        key == "code"
                    })
                    .unwrap();

                let (_, value) = code_pair;
                code = AuthorizationCode::new(value.into_owned());

                let state_pair = url
                    .query_pairs()
                    .find(|pair| {
                        let &(ref key, _) = pair;
                        key == "state"
                    })
                    .unwrap();

                let (_, value) = state_pair;
                state = CsrfToken::new(value.into_owned());
            }

            let message = "Go back to your terminal :)";
            let response = format!(
                "HTTP/1.1 200 OK\r\ncontent-length: {}\r\n\r\n{}",
                message.len(),
                message
            );
            stream.write_all(response.as_bytes()).unwrap();

            println!("Google returned the following code:\n{}\n", code.secret());

            println!(
                "Google returned the following state:\n{} (expected `{}`)\n",
                state.secret(),
                csrf_state.secret()
            );

            // Exchange the code with a token.
            let token = client.exchange_code(code);
            println!("Google returned the following token:\n{:?}\n", token);

            // if token is gucci, return that mf
            match token {
                Ok(response) => return Some(response.access_token().secret().to_string()),
                _ => {
                    println!("bad response: {:?}", token);
                    return None::<String>
                }
            };

            // The server will terminate itself after collecting the first code.
            break;
        }
    }
    println!("got past for loop :0 fuck", );
    None::<String>
}

fn save_token(token: AccessToken){

}