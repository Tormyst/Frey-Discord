extern crate discord;

use discord::{Discord,  State};
use discord::model::{Event, ChannelType, PossibleServer, Presence};
use std::env;

mod event_handler;

fn main() {
    // Log in to Discord using a bot token from the environment
    let discord = Discord::from_bot_token(
        &env::var("DISCORD_TOKEN").expect("Expected token"),
        ).expect("login failed");

    // Establish the websocket connection
    let (mut connection, ready) = discord.connect().expect("connect failed");
    let mut state = State::new(ready);
    println!("[Debug] state.servers() = {:?}", state.servers());
    println!("[Debug] state.unavailable_servers() = {:?}", state.unavailable_servers());
    let channel_count: usize = state.servers().iter()
        .map(|srv| srv.channels.iter()
             .filter(|chan| chan.kind == ChannelType::Text)
             .count()
            ).fold(0, |v, s| v + s);
    println!("[Ready] {} logging {} servers with {} text channels", state.user().username, state.servers().len(), channel_count);

    loop {
        // Receive an event and update the state with it
        let event = match connection.recv_event() {
            Ok(event) => event,
            Err(discord::Error::Closed(code, body)) => {
                println!("[Error] Connection closed with status {:?}: {}", code, body);
                break
            }
            Err(err) => {
                println!("[Warning] Receive error: {:?}", err);
                continue
            }
        };
        state.update(&event);

        // Log messages
        match event {
            Event::MessageCreate(message) => {
                event_handler::handleMessageCreate(message, &state)
            }
            Event::ServerCreate(PossibleServer::Online(server)) => {
                event_handler::handleServerCreateOnline(server)
            }
            Event::PresenceUpdate {presence: Presence{ game: Some(game), user_id, .. }, server_id: Some(server_id), roles: _ } => {
                event_handler::handlePresenceUpdateStartGame(&discord, game, user_id, server_id)
            }
            Event::Unknown(name, data) => {
                // log unknown event types for later study
                println!("[Unknown Event] {}: {:?}", name, data);
            }
            x => {
                println!("[Debug] uncaught event  = {:?}", x);
            }, // discard other known events
        }
    }
}
