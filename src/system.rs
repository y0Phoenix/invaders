use rodio::{mixer::Mixer, Decoder, Player};
use serde::{Deserialize, Serialize};
use serde_json;
use std::{
    collections::HashMap,
    error::Error,
    fs::File,
    io::{self, Read},
    sync::{
        mpsc::{self, Sender},
        Arc, Mutex,
    },
    thread::{self, JoinHandle},
};

pub const AUDIO_THREAD_COUNT: u32 = 4;

use crate::menu::Menu;

#[derive(Serialize, Deserialize, Debug)]
pub struct SystemPlayer {
    pub name: String,
    pub score: u32,
    pub is: String,
}

impl SystemPlayer {
    pub fn new(name: String) -> Self {
        Self {
            name,
            score: 0,
            is: "score".to_string(),
        }
    }
}

pub struct System {
    pub menu: Menu,
    pub player: SystemPlayer,
    pub high_scores: Vec<SystemPlayer>,
}

impl System {
    pub fn new() -> Self {
        Self {
            menu: Menu::Main,
            player: SystemPlayer::new("new player".to_string()),
            high_scores: Vec::new(),
        }
    }
    pub fn read_data(&mut self) -> Result<bool, Box<dyn Error>> {
        let mut file = File::open("data/scores.json")?;
        let mut contents = String::new();
        file.read_to_string(&mut contents)?;
        let data: Vec<SystemPlayer> = serde_json::from_str(&contents)?;
        self.high_scores = data;
        Ok(true)
    }
    pub fn get_menu_input(&mut self) {
        let mut input = String::new();
        io::stdin()
            .read_line(&mut input)
            .expect("Failed To Get User Input");
        match input.trim() {
            "Main" => self.menu = Menu::Main,
            "Game" => self.menu = Menu::Game,
            "Help" => self.menu = Menu::Help,
            "Scores" => self.menu = Menu::Leaderboard,
            _ => println!("Invalid Menu Input"),
        }
    }
}

pub struct AudioThread {
    handle: JoinHandle<()>,
    tx: Sender<Arc<AudioChannel>>,
    is_playing: Arc<Mutex<bool>>,
}

fn get_ready_thread(threads: &Vec<AudioThread>) -> Option<usize> {
    for (i, t) in threads.iter().enumerate() {
        if *t.is_playing.lock().unwrap() == false {
            return Some(i);
        }
    }
    None
}

pub struct AudioChannel {
    file: &'static str,
    stop: bool,
}

pub struct Audio {
    files: HashMap<&'static str, &'static str>,
    thread_handle: JoinHandle<()>,
    audio_tx: Sender<AudioChannel>,
}

impl Audio {
    /// `mixer` comes from the device sink opened in main:
    /// `rodio::DeviceSinkBuilder::open_default_sink()?.mixer().clone()`
    pub fn new(mixer: Mixer) -> Self {
        let (tx, rx) = mpsc::channel::<AudioChannel>();
        let handle = thread::spawn(move || {
            // initialize seperate threads
            let mut threads = Vec::new();
            for _ in 0..AUDIO_THREAD_COUNT {
                // initilize channel
                let (tx, rx) = mpsc::channel::<Arc<AudioChannel>>();
                // initilize player (formerly Sink)
                let player = Player::connect_new(&mixer);
                // initilize is_playing to determine if the thread is playing a sound
                let is_playing = Arc::new(Mutex::new(false));
                let is_playing_clone = Arc::clone(&is_playing);
                // initilize thread
                let handle = thread::spawn(move || {
                    let is_playing = is_playing_clone;
                    loop {
                        match rx.recv() {
                            Ok(data) => {
                                if data.stop {
                                    player.stop()
                                } else {
                                    let file = File::open(data.file).expect(
                                        format!("Failed To Open File {}", data.file).as_str(),
                                    );
                                    let source = Decoder::try_from(file).unwrap();
                                    *is_playing.lock().unwrap() = true;
                                    player.append(source);
                                    player.sleep_until_end();
                                    *is_playing.lock().unwrap() = false;
                                }
                            }
                            Err(_) => break,
                        }
                    }
                });
                // push the data to the thread management Vector
                threads.push(AudioThread {
                    handle,
                    tx,
                    is_playing,
                });
            }
            loop {
                match rx.recv() {
                    Ok(data) => {
                        let data = Arc::new(data);
                        if data.stop {
                            for t in threads.iter() {
                                let data = Arc::clone(&data);
                                let _ = t.tx.send(data);
                            }
                        } else if let Some(i) = get_ready_thread(&threads) {
                            let t = threads.get(i).unwrap();
                            let _ = t.tx.send(data);
                        }
                    }
                    Err(_) => break,
                }
            }
            for t in threads.into_iter() {
                drop(t.tx);
                t.handle.join().unwrap();
            }
        });
        Self {
            files: HashMap::new(),
            thread_handle: handle,
            audio_tx: tx,
        }
    }

    pub fn add(&mut self, name: &'static str, path: &'static str) {
        self.files.insert(name, path);
    }

    pub fn play(&mut self, key: &'static str) {
        let file = self.files.get(key).expect("Invalid Audio Name");
        let _ = self.audio_tx.send(AudioChannel { file, stop: false });
    }

    pub fn stop(&mut self) {
        let _ = self.audio_tx.send(AudioChannel {
            file: "nothing",
            stop: true,
        });
    }
    pub fn close(self) {
        drop(self.audio_tx);
        self.thread_handle.join().unwrap();
    }
}
