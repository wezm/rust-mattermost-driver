extern crate mattermost_driver as mattermost;

use std::env;

use dotenv::dotenv;

use mattermost::{Client, Error, UnauthenticatedClient};

fn main() {
    dotenv().ok();

    let url = env::var("MATTERMOST_URL")
        .expect("MATTERMOST_URL must be set")
        .parse()
        .expect("MATTERMOST_URL is invalid");
    let client = UnauthenticatedClient::new(url).expect("URL is not https");
    let login_id = env::var("MATTERMOST_USER").expect("MATTERMOST_USER must be set");
    let password = env::var("MATTERMOST_PASS").expect("MATTERMOST_PASS must be set");

    let mut rt = tokio::runtime::Runtime::new().unwrap();

    let client = client.authenticate(login_id, password, None);
    let client = rt.block_on(client);

    eprintln!("Client = {:?}", client);
}
