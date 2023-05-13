use mpris::{MetadataValue, PlaybackStatus, Player, PlayerFinder};
use rofi;
use std::process;

fn connect() -> Result<Vec<Player>, ()> {
    let player = match PlayerFinder::new() {
        Err(_) => return Err(()),
        Ok(v) => v,
    }
    .find_all();
    if let Ok(player) = player {
        return Ok(player);
    } else {
        return Err(());
    }
}

fn get_player_metadata(player_name: &Player) -> Vec<String> {
    let mut data: Vec<String> = Vec::new();
    let metadata = match player_name.get_metadata() {
        Err(e) => {
            // no metadata but there is player
            println!("Error : {e}");
            data.push("Not available".to_string());
            data.push("Not available".to_string());
            return data;
        }
        Ok(v) => v,
    };
    if let Some(MetadataValue::String(title)) = metadata.get("xesam:title") {
        data.push(title.to_owned());
    } else {
        data.push("Not available".to_string());
    };
    if let Some(MetadataValue::Array(artist)) = metadata.get("xesam:artist") {
        if artist.len() > 1 {
            let mut data_to_push: String = String::new();
            for n in 0..artist.len() {
                if let Some(MetadataValue::String(artist_str)) = artist.get(n) {
                    match n {
                        0 => data_to_push.push_str(&artist_str.to_string()),
                        _ => data_to_push.push_str(&format!(", {artist_str}")),
                    };
                } else {
                    data.push("Not available".to_string());
                }
            }
        } else {
            if let Some(MetadataValue::String(artist_str)) = artist.get(0) {
                data.push(artist_str.to_string());
            } else {
                data.push("Not available".to_string());
            }
        }
    };
    if data.len() < 2 {
        data.push("Not available".to_string());
    } else if data[1].chars().count() < 1 {
        data[1].push_str("Not available");
    }
    return data;
}

fn spawn_rofi(output_vec: &Vec<String>, promt: &str) -> Result<usize, rofi::Error> {
    let selected = rofi::Rofi::new(&output_vec).prompt(promt).run_index();
    return selected;
}

fn play_pause_player(player: &Player) {
    match player.play_pause() {
        Ok(()) => {}
        Err(err) => {
            println!("ERROR : {}", err);
            process::exit(1);
        }
    }
}

fn main() {
    let mut data: Vec<String> = Vec::new();
    let player: Vec<Player> = match connect() {
        Ok(val) => val,
        Err(()) => {
            println!("ERROR : Cant connect to Dbus");
            process::exit(1);
        }
    };
    for n in 0..player.len() {
        let player_metadata: Vec<String> = get_player_metadata(&player[n]);
        let mut status_icon: String = String::new();
        let selected_player_status = match player[n].get_playback_status() {
            Ok(val) => val,
            Err(err) => {
                println!("ERROR : {}", err);
                process::exit(1);
            }
        };
        match selected_player_status {
            PlaybackStatus::Paused => status_icon.push_str(""),
            PlaybackStatus::Playing => status_icon.push_str(""),
            PlaybackStatus::Stopped => status_icon.push_str("󰄮"),
        }
        data.push(format!(
            "{} {} - {} ({})",
            status_icon,
            player_metadata[0],
            player_metadata[1],
            player[n].bus_name_player_name_part().to_string()
        ));
    }

    let user_choice: usize = match spawn_rofi(&data, " Select Player ") {
        Ok(val) => val,
        Err(err) => {
            println!("ERROR : {}", err);
            process::exit(1);
        }
    };
    play_pause_player(&player[user_choice]);
}
