extern crate discord;
extern crate rand;

use discord::{Discord, State};
use std::env;
use std::sync::Mutex;

mod event_handler;
mod incoming;
mod server_handler;
mod server_config {}

pub struct Context {
    discord: Discord,
    state: Mutex<State>,
}



fn main() {
    // Log in to Discord using a bot token from the environment
    let discord = Discord::from_bot_token(&env::var("DISCORD_TOKEN").expect("Expected token"))
        .expect("login failed");

    let incoming = incoming::Incoming::new(discord);
    incoming.run();

}
