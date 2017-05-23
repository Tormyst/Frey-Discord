use discord::{Discord, State, ChannelRef};
use discord::model::{Message, LiveServer, Game, UserId, ServerId, Member};
use std::collections::HashMap;
use rand::thread_rng;
use rand::Rng;

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
            let _ = discord.send_message(c.id,
                                         get_start_game_message(discord
                                                                    .get_member(server_id,
                                                                                user_id)
                                                                    .expect("Failed get user",),
                                                                game)
                                                 .as_str(),
                                         "",
                                         false);
        } else {
            println!("[PresenceUpdate] missing channel to send on")
        }
    } else {
        println!("[PresenceUpdate] Did something")
    }
}

fn get_start_game_message(member: Member, game: Game) -> String {
    let mut map = HashMap::new();
    map.insert("TIS-100",
               ["$user is having a brain melting time playing $game",
                "mov $user, TIS-100"]);
    let string = match map.get(game.name.as_str()) {
        Some(options) => thread_rng().choose(options).unwrap(),
        None => "$user is now playing $game",
    };

    string
        .replace("$user", member.display_name())
        .replace("$game", game.name.as_str())

}
