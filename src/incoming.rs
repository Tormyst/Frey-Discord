extern crate discord;
extern crate rand;

use discord::{Discord, State};
use discord::model::{Event, ServerId, LiveServer, ChannelType, PossibleServer, Server};
use std::collections::HashMap;
use std::sync::{mpsc, Arc, Mutex};
use {Context, event_handler};
use server_handler;


pub struct Incoming {
    servers: HashMap<ServerId, mpsc::Sender<Event>>,
    context: Arc<Context>,
    connection: discord::Connection,
}

impl Incoming {
    pub fn new(discord: Discord) -> Incoming {
        let (connection, ready) = discord.connect().expect("connect failed");
        let state = State::new(ready);
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
                Event::MessageCreate(message) => {
                    let state = self.context.state.lock().unwrap();
                    match state.find_channel(message.channel_id) {
                        Some(discord::ChannelRef::Public(&LiveServer { id, .. }, ..)) => {
                            println!("sending to child");
                            self.pass_event(id, Event::MessageCreate(message));
                        }
                        _ => {
                            println!("handleing in main");
                            event_handler::handle_message_create(message, &state);
                        }
                    }
                }
                Event::PresenceUpdate {
                    presence,
                    server_id: Some(id),
                    roles,
                } => {
                    self.pass_event(id,
                                    Event::PresenceUpdate {
                                        presence,
                                        server_id: Some(id),
                                        roles,
                                    });
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

    fn pass_event(&self, id: ServerId, e: Event) {
        match self.servers.get(&id) {
            Some(s) => s.send(e).unwrap(),
            None => {}
        }
    }

    fn add_server_internal(servers: &mut HashMap<ServerId, mpsc::Sender<Event>>,
                           ls: &LiveServer,
                           context: Arc<Context>) {
        let (sender, recever): (mpsc::Sender<Event>, mpsc::Receiver<Event>) = mpsc::channel();
        servers.insert(ls.id, sender);
        server_handler::ServerHandler::create(ls.id, recever, context);
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
