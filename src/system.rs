use std::{fs::{File}, error::Error, io::{Read, self, BufReader}, collections::HashMap, thread::{JoinHandle, self}, sync::{mpsc::{Sender, self}, Arc, Mutex}};
use rodio::{Decoder,Sink, OutputStreamHandle};
use serde::{Serialize, Deserialize};
use serde_json;

pub const AUDIO_THREAD_COUNT: u32 = 4;

use crate::{menu::Menu};

#[derive(Serialize, Deserialize, Debug)]
pub struct SystemPlayer {
    pub name: String,
    pub score: u32,
    pub is: String
}

impl SystemPlayer {
    pub fn new(name: String) -> Self {
        Self { name, score: 0, is: "score".to_string() }
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
            menu: Menu::Main, player: 
            SystemPlayer::new("new player".to_string()), 
            high_scores: Vec::new(),
        }
    }
    pub fn read_data(&mut self) -> Result<bool, Box<dyn Error>> {
        let mut file = match File::open("data/scores.json") {
            Ok(file) => file,
            Err(e) => return Err(Box::new(e)),
        };
        let mut contents = String::new();
        match file.read_to_string(&mut contents) {
            Ok(data) => data,
            Err(e) => return Err(Box::new(e))
        };
        let data: Vec<SystemPlayer> = match serde_json::from_str(&contents) {
            Ok(data) => data,
            Err(e) => return Err(Box::new(e))
        };
        self.high_scores = data;
        Ok(true)
    }
    pub fn get_menu_input(&mut self) {
        let mut input = String::new();
        io::stdin().read_line(&mut input).expect("Failed To Get User Input");
        input.trim().to_string();
        match input.as_str() {
            "Main" => self.menu = Menu::Main,
            "Game" => self.menu = Menu::Game,
            "Help" => self.menu = Menu::Help,
            "Scores" => self.menu = Menu::Leaderboard,
            _ => println!("Invalid Menu Input")
        }
    }
}

pub struct AudioThread {
    handle: JoinHandle<()>,
    tx: Sender<Arc<AudioChannel>>,
    is_playing: Arc<Mutex<bool>>
}

fn get_ready_thread(threads: &Vec<AudioThread>) -> Option<usize> {
    for (i, t) in threads.iter().enumerate() {
        if *t.is_playing.lock().unwrap() == false {
            return  Some(i);
        }
    }
    None
}

pub struct AudioChannel {
    file: &'static str,
    stop: bool
}

pub struct Audio {
    files: HashMap<&'static str, &'static str>,
    thread_handle: JoinHandle<()>,
    audio_tx: Sender<AudioChannel>,
}

impl Audio {
    pub fn new(stream: OutputStreamHandle) -> Self {
        let (tx, rx) = mpsc::channel::<AudioChannel>();
        let handle = thread::spawn(move || {
            // initialize seperate threads
            let mut threads = Vec::new();
            for _ in 0..AUDIO_THREAD_COUNT {
                // initilize channel
                let (tx, rx) = mpsc::channel::<Arc<AudioChannel>>();
                // initilize sink
                let sink = Sink::try_new(&stream).unwrap();
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
                                    sink.stop()
                                }
                                else {
                                    let file = File::open(data.file).expect(format!("Failed To Open File {}", data.file).as_str());
                                    let buf = BufReader::new(file);
                                    let source: Decoder<BufReader<File>> = Decoder::new(buf).unwrap();
                                    *is_playing.lock().unwrap() = true;
                                    sink.append(source);
                                    sink.sleep_until_end();
                                    *is_playing.lock().unwrap() = false;           
                                }
                            }
                            Err(_) => break
                        }
                    }
                });
                // push the data to the thread management Vector
                threads.push(AudioThread { handle, tx, is_playing });
            }
            loop {
                // thread::sleep(Duration::from_millis(1));
                match rx.recv() {
                    Ok(data) => {
                        let data = Arc::new(data);
                        if data.stop {
                            for t in threads.iter() {
                                let data = Arc::clone(&data);
                                let _ = t.tx.send(data);
                            }
                        }
                        else {
                            // println!("hit spacebar");
                            if let Some(i) = get_ready_thread(&threads) {
                                let t = threads.get(i).unwrap();
                                let _ = t.tx.send(data);
                                
                            }
                        }
                    },
                    Err(_) => break
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
        let _ = self.audio_tx.send(AudioChannel { file: file, stop: false });
    }

    pub fn stop(&mut self) {
        let _ = self.audio_tx.send(AudioChannel { file: "nothing", stop: true });
    }
    pub fn close(self) {
        drop(self.audio_tx);
        self.thread_handle.join().unwrap();
    }
}