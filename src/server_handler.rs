extern crate discord;
extern crate rand;

use discord::model::{Event, ServerId, Presence};
use std::thread;
use std::sync::{mpsc, Arc};
use {Context, event_handler};
use server_config;

pub struct ServerHandler {
    id: ServerId,
    recever: mpsc::Receiver<Event>,
    context: Arc<Context>,
    settings: server_config::ServerSettings,
}

impl ServerHandler {
    pub fn create(id: ServerId, recever: mpsc::Receiver<Event>, context: Arc<Context>) {
        let mut t = ServerHandler {
            id,
            recever,
            context,
            settings: server_config::get(id),
        };
        let _ = t.load_config();
        thread::spawn(move || t.main());
    }

    fn main(&mut self) {
        {
            // helper.reorder_game_ranks(&self.context)
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
                    event_handler::handle_presence_update_start_game(game,
                                                                     user_id,
                                                                     server_id,
                                                                     &self.context,
                                                                     &self.settings,)
                }
                Event::PresenceUpdate {
                    presence: Presence {
                        game: None,
                        user_id,
                        ..
                    },
                    server_id: Some(server_id),
                    roles: Some(roles),
                } => {
                    println!("[PresenceUpdate] matched no game.");
                    event_handler::handle_presence_update_end_game(user_id,
                                                                   server_id,
                                                                   &roles,
                                                                   &self.context)
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
