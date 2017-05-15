extern crate discord;

use discord::{Discord, State, ChannelRef};
use discord::model::{Message, LiveServer, Game, UserId, ServerId};

pub fn handleMessageCreate(message: Message, state: &State) {
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

pub fn handleServerCreateOnline(server: LiveServer) {
    // setup(server)
    println!("[ServerCreate] found online server: {}", server.name)
}

pub fn handlePresenceUpdateStartGame(discord: &Discord, game: Game, user_id: UserId, server_id: ServerId) {
    // let username = match presence.nick {
    //     Some(u) => u,
    //     None => match presence.user {
    //         Some(u) => 
    // }
    if let Ok(vec) = discord.get_server_channels(server_id) {
        if let Some(c) = vec.first() {
            discord.send_message(c.id, format!("[PresenceUpdate] {} is now playing {}", discord.get_member(server_id, user_id).unwrap().display_name(), game.name).as_str(), "", false); () 
        }
        else { 
            println!("[PresenceUpdate] missing channel to send on") 
        }
    }
    else {
        println!("[PresenceUpdate] Did something") 
    }
}
