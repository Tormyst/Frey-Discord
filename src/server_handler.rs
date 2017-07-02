extern crate discord;
extern crate rand;

use discord::model::{Event, ServerId, Presence};
use std::thread;
use std::sync::{mpsc, Arc};
use {Context, event_handler};

macro_rules! my_server {
    ($self:expr, $state:expr) => ($state.find_server($self.id).unwrap())
}

pub struct ServerHandler {
    id: ServerId,
    recever: mpsc::Receiver<Event>,
    context: Arc<Context>,
}

impl ServerHandler {
    pub fn create(id: ServerId, recever: mpsc::Receiver<Event>, context: Arc<Context>) {
        let mut t = ServerHandler {
            id,
            recever,
            context,
        };
        let _ = t.load_config();
        thread::spawn(move || t.main());
    }

    fn reorder_game_ranks(&self, context: &Context) {
        let state = &context.state.lock().unwrap();
        let server = my_server!(self, state);
        println!("Server state: {:?}", &server);
        let current_user_id = state.user().id;
        println!("Current user ID: {:?}", current_user_id);
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
            .position as usize;
        let mut new_roles = Vec::new();
        for discord::model::Role { name, id, position, .. } in roles {
            if name.chars().nth(0).unwrap() == '_' {
                println!("Role {}: {} : {}", position, name, id);
                new_roles.push((id, my_position));
            }
        }
        println!("{:?}",
                 context.discord.reorder_roles(self.id, new_roles.as_slice()));
    }

    fn main(&mut self) {
        {
            self.reorder_game_ranks(&self.context)
        }
        loop {
            let event = match self.recever.recv() {
                Err(err) => {
                    println!("{}", err);
                    continue;
                }
                Ok(event) => event, 
            };
            match event {
                Event::MessageCreate(message) => {
                    event_handler::handle_message_create(message,
                                                         &self.context.state.lock().unwrap());
                }
                Event::ServerDelete(..) => {
                    println!("[ServerDelete] Server delete sent.  Closing thread {}",
                             self.id);
                    break;
                }
                Event::PresenceUpdate {
                    presence: Presence {
                        game: Some(game),
                        user_id,
                        ..
                    },
                    server_id: Some(server_id),
                    roles: _,
                } => {
                    println!("[PresenceUpdate] matched game start.");
                    event_handler::handle_presence_update_start_game(&self.context.discord,
                                                                     game,
                                                                     user_id,
                                                                     server_id)
                }
                Event::Unknown(name, data) => {
                    // log unknown event types for later study
                    println!("[Unknown Event] {}: {:?}", name, data);
                }
                x => {
                    println!("[Debug] uncaught event  = {:?}", x);
                } // discard other known events
            }
        }
    }

    fn load_config(&self) -> Option<bool> {
        Some(true)
    }
}
