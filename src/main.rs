extern crate discord;

use discord::{Discord, ChannelRef, State};
use discord::model::{Event, ChannelType, PossibleServer};
use std::env;

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
				match state.find_channel(message.channel_id) {
					Some(ChannelRef::Public(server, channel)) => {
						println!("[{} #{}] {}: {}", server.name, channel.name, message.author.name, message.content);
					}
					Some(ChannelRef::Group(group)) => {
						println!("[Group {}] {}: {}", group.name(), message.author.name, message.content);
					}
					Some(ChannelRef::Private(channel)) => {
						if message.author.name == channel.recipient.name {
							println!("[Private] {}: {}", message.author.name, message.content);
						} else {
							println!("[Private] To {}: {}", channel.recipient.name, message.content);
						}
					}
					None => println!("[Unknown Channel] {}: {}", message.author.name, message.content),
				}
			}
            Event::ServerCreate(PossibleServer::Online(server)) => {
                // setup(server)
                println!("[ServerCreate] found online server: {}", server.name)
            }
            Event::PresenceUpdate {presence, server_id: Some(server_id), roles: _ } => {
                // let username = match presence.nick {
                //     Some(u) => u,
                //     None => match presence.user {
                //         Some(u) => 
                // }
                if let Ok(vec) = discord.get_server_channels(server_id) {
                    let c = vec.first().unwrap().id;
                    match presence.game {
                        Some(game) => { discord.send_message(c, format!("[PresenceUpdate] {} is now playing {}", discord.get_member(server_id, presence.user_id).unwrap().display_name(), game.name).as_str(), "", false); () },
                        None => println!("[PresenceUpdate] Did something")
                    }
                }
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
