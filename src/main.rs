extern crate discord;
extern crate rand;

use discord::{Discord, State};
use discord::model::{Event, ServerId, LiveServer, ChannelType, PossibleServer, Server, Presence};
use std::env;
use std::thread;
use std::collections::HashMap;
use std::sync::{mpsc, Arc, Mutex};

mod event_handler;

struct Context {
    discord: Discord,
    state: Mutex<State>,
}

struct Incoming {
    servers: HashMap<ServerId, mpsc::Sender<Event>>,
    context: Arc<Context>,
    connection: discord::Connection,
}

impl Incoming {
    pub fn new(discord: Discord) -> Incoming {
        let (mut connection, ready) = discord.connect().expect("connect failed");
        let mut state = State::new(ready);
        Incoming {
            servers: HashMap::new(),
            context: Arc::new(Context {
                                  discord,
                                  state: Mutex::new(state),
                              }),
            connection,
        }
    }

    pub fn run(mut self) {
        self.startup();
        loop {
            // Receive an event and update the state with it
            let event = match self.connection.recv_event() {
                Ok(event) => event,
                Err(discord::Error::Closed(code, body)) => {
                    println!("[Error] Connection closed with status {:?}: {}", code, body);
                    // Close all threads.
                    break;
                }
                Err(err) => {
                    println!("[Warning] Receive error: {:?}", err);
                    continue;
                }
            };
            self.context.state.lock().unwrap().update(&event);

            // Log messages
            match event {
                Event::ServerCreate(PossibleServer::Online(server)) => {
                    self.add_server(&server);
                    event_handler::handle_server_create_online(server)
                }
                Event::ServerDelete(ps) => {
                    self.remove_server(match ps {
                                           PossibleServer::Online(Server { id, .. }) => id,
                                           PossibleServer::Offline(id) => id,
                                       });
                }
                /*
            Event::MessageCreate(message) => {
                event_handler::handle_message_create(message, &state);
            }
            Event::Unknown(name, data) => {
                // log unknown event types for later study
                println!("[Unknown Event] {}: {:?}", name, data);
            }
            */
                x => {
                    println!("[Debug] uncaught event  = {:?}", x);
                } // discard other known events
            }
        }
    }

    fn startup(&mut self) {
        let state = self.context.state.lock().unwrap();
        // Establish the websocket connection
        println!("[Debug] state.servers() = {:?}", state.servers());
        println!("[Debug] state.unavailable_servers() = {:?}",
                 state.unavailable_servers());
        let channel_count: usize = state
            .servers()
            .iter()
            .map(|srv| {
                     srv.channels
                         .iter()
                         .filter(|chan| chan.kind == ChannelType::Text)
                         .count()
                 })
            .fold(0, |v, s| v + s);
        println!("[Ready] {} logging {} servers with {} text channels",
                 state.user().username,
                 state.servers().len(),
                 channel_count);
        let servers = state.servers();
        for server in servers {
            Incoming::add_server_internal(&mut self.servers, server, self.context.clone());
        }
    }

    fn add_server_internal(servers: &mut HashMap<ServerId, mpsc::Sender<Event>>,
                           ls: &LiveServer,
                           context: Arc<Context>) {
        let (sender, recever): (mpsc::Sender<Event>, mpsc::Receiver<Event>) = mpsc::channel();
        servers.insert(ls.id, sender);
        ServerHandler::create(ls.id, recever, context);
    }

    pub fn add_server(&mut self, ls: &LiveServer) {
        Incoming::add_server_internal(&mut self.servers, ls, self.context.clone())
    }

    pub fn remove_server(&mut self, id: ServerId) {
        match self.servers.get(&id) {
            Some(s) => {
                s.send(Event::ServerDelete(PossibleServer::Offline(id)))
                    .unwrap()
            }
            None => {}
        }
        self.servers.remove(&id);
    }
}

struct ServerHandler {
    id: ServerId,
    recever: mpsc::Receiver<Event>,
}

impl ServerHandler {
    pub fn create(id: ServerId, recever: mpsc::Receiver<Event>, context: Arc<Context>) {
        let mut t = ServerHandler { id, recever };
        thread::spawn(move || t.main(context));
    }

    fn main(&mut self, context: Arc<Context>) {
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
                    // event_handler::handle_message_create(message, &state);
                }
                Event::ServerDelete(server) => {
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
                    event_handler::handle_presence_update_start_game(&context.discord,
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
}

fn main() {
    // Log in to Discord using a bot token from the environment
    let discord = Discord::from_bot_token(&env::var("DISCORD_TOKEN").expect("Expected token"))
        .expect("login failed");

    let mut incoming = Incoming::new(discord);
    incoming.run();

}
