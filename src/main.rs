extern crate mattermost_driver as mattermost;

use std::env;

use dotenv::dotenv;
use futures::future::{self, Future};

use mattermost::{
    client::PaginationParameters, post::CreatePost, user::UserParam, Client, Error,
    UnauthenticatedClient,
};

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

    let work = client.authenticate(login_id, password, None);
    let client = rt.block_on(work).expect("error logging in");

    let work = futures::future::ok(client.clone())
        .and_then(|client| {
            client
                .get_user_teams(UserParam::Me)
                .map(|teams| (client, teams))
        })
        .and_then(|(client, teams)| {
            let team = teams.first().expect("no teams");
            client
                .get_team_channels_for_user(&team.id, UserParam::Me)
                .map(|channels| (client, channels))
        })
        .and_then(|(client, mut channels)| {
            // Find geeks channel
            let index = channels
                .iter()
                .position(|channel| channel.name == "geeks")
                .unwrap();
            let geeks = channels.remove(index);

            // Get posts
            // let params = PaginationParameters::default();
            // client.get_channel_posts(geeks.id, params)

            // Post message
            let post = CreatePost {
                channel_id: geeks.id,
                message: "Hello from :rust:".to_string(),
                root_id: None,
                file_ids: None,
            };
            client.create_post(post)
        });

    rt.block_on(work).expect("future error");
}
