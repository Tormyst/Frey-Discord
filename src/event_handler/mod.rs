use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};

use Context;
use discord::{State, ChannelRef};
use discord::model::{Message, LiveServer, Game, UserId, ServerId, RoleId};
use discord::model::permissions::Permissions;

static MAX_COLOR: u64 = 16777216; //2u64.pow(24)
static IDENTIFY_ROLE: &'static str = "__";

mod game_message;
#[macro_use]
mod helper {
    use Context;
    use discord::model::{ServerId, RoleId};
    // use discord::model::Role
    // use super::IDENTIFY_ROLE;

    macro_rules! my_server {
        ($self:expr, $state:expr) => ($state.find_server($self).unwrap())
    }

    pub fn reorder_single_rank(server: &ServerId, rank: RoleId, context: &Context) {
        let state = &context.state.lock().unwrap();
        let server = my_server!(*server, state);
        // println!("Server state: {:?}", &server);
        let current_user_id = state.user().id;
        // println!("Current user ID: {:?}", current_user_id);
        let roles = server.roles.clone();

        let my_member_role = server
            .members
            .iter()
            .find(|&member| member.user.id == current_user_id)
            .unwrap()
            .roles
            .get(0)
            .unwrap();

        println!("My_member_role {}", my_member_role);

        let my_position = roles
            .iter()
            .find(|&role| role.id == *my_member_role)
            .unwrap()
            .position;

        println!("My_position: {:?}", my_position);

        let new_role = vec![(rank, my_position as usize)];

        println!("{:?}",
                 context
                     .discord
                     .reorder_roles(server.id, new_role.as_slice())
                     .unwrap());
    }

/*
    pub fn reorder_game_ranks(server: &ServerId, context: &Context) {
        let state = &context.state.lock().unwrap();
        let server = my_server!(*server, state);
        // println!("Server state: {:?}", &server);
        let current_user_id = state.user().id;
        // println!("Current user ID: {:?}", current_user_id);
        let roles = server.roles.clone();


        let my_member_role = server
            .members
            .iter()
            .find(|&member| member.user.id == current_user_id)
            .unwrap()
            .roles
            .get(0)
            .unwrap();

        println!("My_member_role {}", my_member_role);

        let my_position = roles
            .iter()
            .find(|&role| role.id == *my_member_role)
            .unwrap()
            .position;

        println!("My_position: {:?}", my_position);

        let mut new_roles = Vec::new();
        for Role { name, id, .. } in roles {
            if name.starts_with(IDENTIFY_ROLE) {
                println!("Moving role {:?}", name);
                new_roles.push((id, my_position as usize));
            }
        }

        println!("{:?}",
                 context
                     .discord
                     .reorder_roles(server.id, new_roles.as_slice())
                     .unwrap());
    }
    */
}

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

pub fn handle_presence_update_start_game(game: Game,
                                         user_id: UserId,
                                         server_id: ServerId,
                                         context: &Context) {
    // let username = match presence.nick {
    //     Some(u) => u,
    //     None => match presence.user {
    //         Some(u) =>
    // }
    let discord = &context.discord;
    if let Ok(vec) = discord.get_server_channels(server_id) {
        if let Some(c) = vec.first() {
            let member = discord
                .get_member(server_id, user_id)
                .expect("Failed get user");
            let _ = discord.send_message(c.id,
                                         game_message::get_start_game_message(&member, &game)
                                             .as_str(),
                                         "",

                                         false);
            let mut hasher = DefaultHasher::new();
            game.name.hash(&mut hasher);
            let hash = hasher.finish() % MAX_COLOR; // Maximum color value
            println!("Game Hash: {:?}", hash);
            let name = format!("{}{}", IDENTIFY_ROLE, game.name);
            let role = discord
                .create_role(server_id,
                             Some(&name),
                             Some(Permissions::empty()),
                             Some(hash),
                             Some(false),
                             Some(false))
                .unwrap();
            helper::reorder_single_rank(&server_id, role.id, &context);
            // helper::reorder_game_ranks(&server_id, &context);
            println!("{:?}", discord.add_member_role(server_id, user_id, role.id));
        } else {
            println!("[PresenceUpdate] missing channel to send on")
        }
    } else {
        println!("[PresenceUpdate] Did something")
    }
}
pub fn handle_presence_update_end_game(user_id: UserId,
                                       server_id: ServerId,
                                       roles: &[RoleId],
                                       context: &Context) {

    let game_roles: Vec<RoleId> = context
        .discord
        .get_roles(server_id)
        .unwrap()
        .into_iter()
        .filter(|x| x.name.starts_with(IDENTIFY_ROLE) && roles.iter().any(|y| *y == x.id))
        .map(|x| {
                 context
                     .discord
                     .remove_member_role(server_id, user_id, x.id)
                     .unwrap();
                 x.id
             })
        .collect();
    let state = &context.state.lock().unwrap();
    let members = &my_server!(server_id, state).members;
    for id in game_roles {
        if !members.iter().any(|m| m.roles.iter().any(|r| id == *r)) {
            let _ = context.discord.delete_role(server_id, id);
        }
    }
}
