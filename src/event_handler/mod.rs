use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};
use discord::{Discord, State, ChannelRef};
use discord::model::{Message, LiveServer, Game, UserId, ServerId};
use discord::model::permissions::Permissions;

mod game_message;

pub fn handle_message_create(message: Message, state: &State) {
    match state.find_channel(message.channel_id) {
        Some(ChannelRef::Public(server, channel)) => {
            println!("[{} #{}] {}: {}",
                     server.name,
                     channel.name,
                     message.author.name,
                     message.content);
        }
        Some(ChannelRef::Group(group)) => {
            println!("[Group {}] {}: {}",
                     group.name(),
                     message.author.name,
                     message.content);
        }
        Some(ChannelRef::Private(channel)) => {
            if message.author.name == channel.recipient.name {
                println!("[Private] {}: {}", message.author.name, message.content);
            } else {
                println!("[Private] To {}: {}",
                         channel.recipient.name,
                         message.content);
            }
        }
        None => {
            println!("[Unknown Channel] {}: {}",
                     message.author.name,
                     message.content)
        }
    }
}

pub fn handle_server_create_online(server: LiveServer) {
    // setup(server)
    println!("[ServerCreate] found online server: {}", server.name)
}

pub fn handle_presence_update_start_game(discord: &Discord,
                                         game: Game,
                                         user_id: UserId,
                                         server_id: ServerId) {
    // let username = match presence.nick {
    //     Some(u) => u,
    //     None => match presence.user {
    //         Some(u) =>
    // }
    if let Ok(vec) = discord.get_server_channels(server_id) {
        if let Some(c) = vec.first() {
            let _ = discord
                .send_message(c.id,
                              game_message
                                  ::get_start_game_message(discord
                                                              .get_member(server_id,
                                                                          user_id)
                                                              .expect("Failed get user"),
                                                          &game)
                                  .as_str(),
                              "",

                              false);
            let mut hasher = DefaultHasher::new();
            game.name.hash(&mut hasher);
            let hash = hasher.finish() % 2 ^ 24; // Maximum color value
            println!("Game Hash: {:?}", hash);
            let name = format!("__{}", game.name);
            let role = discord.create_role(server_id,
                                           Some(&name),
                                           Some(Permissions::empty()),
                                           Some(hash),
                                           Some(false),
                                           Some(false));

            // discord.reorder_roles(server_id,
            println!("{:?}", role);
        } else {
            println!("[PresenceUpdate] missing channel to send on")
        }
    } else {
        println!("[PresenceUpdate] Did something")
    }
}
