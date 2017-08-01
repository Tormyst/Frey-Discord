extern crate discord;
extern crate rand;

use discord::{Discord, State};
use std::env;
use std::sync::Mutex;

#[macro_use]
extern crate serde_derive;
extern crate serde_json;

mod event_handler;
mod incoming;
mod server_handler;
mod server_config {
    use std::collections::HashMap;
    use discord::model::ServerId;
    use std::fs::File;
    use std::path::Path;
    use serde_json::{from_reader, to_writer};
    use std::io::ErrorKind;
    

    static CONFIG_DIR: &'static str = "config/";

    macro_rules! file_path {
        ($server:expr) => {Path::new(&format!("{}{}.config", CONFIG_DIR, $server))}
    }

    #[derive(Serialize, Deserialize, Debug)]
    pub struct ServerSettings {
        pub game_messages: HashMap<String, Vec<String>>,
        id: ServerId
    }

    fn new(id: ServerId) -> ServerSettings {
        ServerSettings {
            game_messages: HashMap::new(),
            id,
        }
    }

    pub fn get(server: ServerId) -> ServerSettings {
        match File::open(file_path!(server)) {
            Ok(file) => from_reader(file).unwrap(),
            Err(e) => if e.kind() == ErrorKind::NotFound {new(server)} else {panic!(e)},
        }
    }

    impl Drop for ServerSettings {
        fn drop(&mut self) {
            println!("Calling deconstructor");
           match File::create(file_path!(self.id)) {
            Ok(file) => {let _ = to_writer(file, &self);},
            Err(_) => {},
           };
        }
    }
}


pub struct Context {
    discord: Discord,
    state: Mutex<State>,
}

fn main() {
    // Log in to Discord using a bot token from the environment
    let discord = Discord::from_bot_token(&env::var("DISCORD_TOKEN").expect("Expected token"))
        .expect("login failed");

    let incoming = incoming::Incoming::new(discord);
    incoming.run();

}
