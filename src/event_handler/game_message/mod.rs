use discord::model::{Member, Game};
use std::collections::HashMap;
use rand::Rng;
use rand::thread_rng;

pub fn get_start_game_message(member: Member, game: &Game) -> String {
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
