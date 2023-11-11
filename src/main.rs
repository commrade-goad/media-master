use mpris::{MetadataValue, PlaybackStatus, Player, PlayerFinder};
use rofi;
use std::process;
use std::env;

enum ProgramArgs {
    PlayPause,
    Loop
}

fn connect() -> Result<Vec<Player>, String> {
    let player = match PlayerFinder::new() {
        Err(err) => return Err(err.to_string()),
        Ok(v) => v,
    }
    .find_all();
    match player {
        Ok(player) => return Ok(player),
        Err(err) => return Err(err.to_string()),
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

fn player_loop_status(player: &Player){
    let player_status: mpris::LoopStatus = match player.get_loop_status(){
        Ok(val) => val,
        Err(err) => {
            println!("ERROR: {}", err);
            process::exit(1);
        }
    };
    match player_status {
        mpris::LoopStatus::None => player.set_loop_status(mpris::LoopStatus::Track).unwrap(),
        mpris::LoopStatus::Track => player.set_loop_status(mpris::LoopStatus::Playlist).unwrap(),
        mpris::LoopStatus::Playlist => player.set_loop_status(mpris::LoopStatus::None).unwrap()
    };
}

fn get_args() -> Result<ProgramArgs, ()> {
    let user_args: Vec<String> = env::args().collect();
    if user_args.len() < 2 {
        println!("ERROR : Didnt get any argument!");
        process::exit(1);
    }
    match &user_args[1][..] {
        "-p" | "--play-pause" => Ok(ProgramArgs::PlayPause),
        "-l" | "--loop" => Ok(ProgramArgs::Loop),
        _ => {
            return Err(())
        }
    }
}

fn get_player_name(active_player : &Player) -> String{
    let player_string: String = active_player.bus_name_player_name_part().to_string();
    return player_string;
}

fn main() {
    let user_args:ProgramArgs = match get_args() {
        Ok(val) => val,
        Err(()) =>{
            println!("ERROR : Invalid argument!");
            process::exit(1)
        }
    };
    let mut data: Vec<String> = Vec::new();
    let player: Vec<Player> = match connect() {
        Ok(val) => val,
        Err(err) => {
            println!("ERROR : {err}");
            process::exit(1);
        }
    };
    if player.len() == 0 {
        println!("ERROR : No player detected!");
        process::exit(1);
    } else if player.len() == 1 {
        match user_args{
            ProgramArgs::Loop => player_loop_status(&player[0]),
            ProgramArgs::PlayPause => play_pause_player(&player[0]),
        } 
        process::exit(0);
    }
    for n in 0..player.len() {
        let player_metadata: Vec<String> = get_player_metadata(&player[n]);
        let mut status_icon: String = String::new();
        let mut loop_status_icon: String = String::new();
        let individual_player_status = match player[n].get_playback_status() {
            Ok(val) => val,
            Err(err) => {
                println!("ERROR : {}", err);
                process::exit(1);
            }
        };
        let individual_player_loop_status: Result<mpris::LoopStatus, ()> = match player[n].get_loop_status(){
            Ok(val) => Ok(val),
            Err(_) => Err(()),
        };
        match individual_player_status {
            PlaybackStatus::Paused => status_icon.push_str(""),
            PlaybackStatus::Playing => status_icon.push_str(""),
            PlaybackStatus::Stopped => status_icon.push_str("󰄮"),
        }
        match individual_player_loop_status {
            Ok(mpris::LoopStatus::Track) => loop_status_icon.push_str("󰓦"),
            Ok(mpris::LoopStatus::Playlist) => loop_status_icon.push_str(" 󰓦"),
            Ok(mpris::LoopStatus::None) => loop_status_icon.push_str("󰓨"),
            Err(_) => loop_status_icon.push_str("x"),
        }
        data.push(format!(
            "{} {} {} - {} ({})",
            status_icon,
            loop_status_icon,
            player_metadata[0],
            player_metadata[1],
            get_player_name(&player[n])
        ));
    }

    let user_choice: usize = match spawn_rofi(&data, " Select Player ") {
        Ok(val) => val,
        Err(err) => {
            println!("ERROR : {}", err);
            process::exit(1);
        }
    };
    match user_args {
        ProgramArgs::Loop => player_loop_status(&player[user_choice]),
        ProgramArgs::PlayPause => play_pause_player(&player[user_choice]),
    }
}
