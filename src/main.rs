use std::{error::Error, time::{Duration, Instant}, sync::{mpsc}, thread};
use invaders::{render::{render}, frame::{self, Drawable, Frame}, player::Player, invaders::Invaders, system::{System, SystemPlayer, Audio}, menu::{Menu, NewMenu}, request::{ReqClient}};
use rodio::OutputStream;
use std::io;
use crossterm::{terminal::{self, LeaveAlternateScreen}, ExecutableCommand, cursor::{Hide, Show}, event::{self, Event, KeyCode}};
use crossterm::terminal::EnterAlternateScreen;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    dotenv::dotenv().ok();
    // initialize audio
    let (_stream, stream_handle) = OutputStream::try_default().unwrap();
    let mut audio = Audio::new(stream_handle);
    audio.add("explosion", "audio/explosion.wav");
    audio.add("lose", "audio/lose.wav");
    audio.add("move", "audio/move.wav");
    audio.add("pew", "audio/pew.wav");
    audio.add("startup", "audio/startup.wav");
    audio.add("win", "audio/win.wav");

    // init reqwest client
    let client = ReqClient::new();

    // init system
    let mut system = System::new();
    
    // init terminal
    let mut stdout = io::stdout();
    terminal::enable_raw_mode()?;
    stdout.execute(EnterAlternateScreen)?;
    stdout.execute(Hide)?;

    // Render loop in a seperate thread
    let (tx, rx) = mpsc::channel::<Frame>();
    let handle = thread::spawn(move || {
        let mut last_frame = frame::new_frame();
        let mut stdout = io::stdout();
        render(&mut stdout, &last_frame, &last_frame, true);
        loop {
            let curr_frame = match rx.recv() {
                Ok(x) => x,
                Err(_) => break,
            };
            render(&mut stdout, &last_frame, &curr_frame, false);
            last_frame = curr_frame.to_vec();
        }
    });

    audio.play("startup");

    'mainloop: loop {
        let mut instant = Instant::now();
        if system.menu == Menu::Main {
            let text = 
            " Welcome To Space Invaders\n
       1: Play Game\n
       2: Leaderboard\n
       3: How To Play\n
       4: Exit";
            let main_menu = NewMenu::new(text.to_string(), 6, 5);
            'mainmenu: loop {
                let mut curr_frame = frame::new_frame();
                while event::poll(Duration::default())? {
                    if let Event::Key(key_event) = event::read()? {
                        match key_event.code {
                            KeyCode::Char('1') => {
                                system.menu = Menu::Game;
                                render(&mut stdout, &curr_frame, &curr_frame, true);
                                break 'mainmenu;
                            },
                            KeyCode::Char('2') => {
                                system.menu = Menu::Leaderboard;
                                render(&mut stdout, &curr_frame, &curr_frame, true);
                                break 'mainmenu; 
                            }
                            KeyCode::Char('3') => {
                                system.menu = Menu::Help;
                                render(&mut stdout, &curr_frame, &curr_frame, true);
                                break 'mainmenu;
                            }
                            KeyCode::Char('4') | KeyCode::Esc => {
                                break 'mainloop;
                            }
                            _ => {}
                        }
                    }
                }
                main_menu.draw(&mut curr_frame);
                let _ = tx.send(curr_frame);
                thread::sleep(Duration::from_millis(1));
            }
        }
        if system.menu == Menu::Game {
            let mut player = Player::new();
            let mut invaders = Invaders::new(1);
            let mut valid_name = true;
            'game: loop {
                // frame init
                let mut curr_frame = frame::new_frame();
                let delta = instant.elapsed();
                instant = Instant::now();
                if !valid_name {
                    system.menu = Menu::Main;
                    render(&mut stdout, &curr_frame, &curr_frame, true);
                    break 'game;
                }
                if player.name.is_empty() {
                    valid_name = player.get_name();
                    render(&mut stdout, &curr_frame, &curr_frame, true);
                }
                let score_str = format!("Level: {}, Score: {}", player.level, player.score);
                let score_display = NewMenu::new(score_str, 8, 0);
                // Input
                while event::poll(Duration::default())? {
                    if let Event::Key(key_event) = event::read()? {
                        match key_event.code {
                            KeyCode::Left | KeyCode::Char('a')=> {
                                player.move_left();
                            }
                            KeyCode::Right | KeyCode::Char('d') => {
                                player.move_right();
                            }
                            KeyCode::Char(' ') => {
                                if player.shoot() {
                                    audio.play("pew");
                                }
                            }
                            KeyCode::Esc | KeyCode::Char('q') => {
                                audio.stop();
                                audio.play("lose");
                                render(&mut stdout, &curr_frame, &curr_frame, true);
                                system.menu = Menu::Main;
                                break 'game;
                            }
                            _ => {}
                        }
                    }
                }
                
                // render + draw
                invaders.draw(&mut curr_frame);
                player.draw(&mut curr_frame);
                score_display.draw(&mut curr_frame);
                player.update(delta);
                if invaders.update(delta) {
                    audio.play("move");
                }
                if player.detect_hits(&mut invaders) {
                    audio.play("explosion");
                }
                
                // win lose conditions
                if invaders.all_dead() {
                    // audio.play("win");
                    invaders = Invaders::new(player.level + 1);
                    player.clear_shots();
                    player.level += 1;
                }
                if invaders.reached_bottom() {
                    audio.stop();
                    audio.play("lose");
                    render(&mut stdout, &frame::new_frame(), &frame::new_frame(), true);
                    system.menu = Menu::Main;
                    break 'game;
                }       
                let _ = tx.send(curr_frame);
                thread::sleep(Duration::from_millis(1));
            }
            let text = match client.update_scores(SystemPlayer {name: player.name.clone(), score: player.score, is: "score".to_string()}).await {
                Ok(str) => str,
                Err(e) => e.to_string()
            };
            let beat_score_display = NewMenu::new(text, 5, 10);

            'endgame: loop {
                let mut curr_frame = frame::new_frame();
                while event::poll(Duration::default())? {
                    if let Event::Key(key_event) = event::read()? {
                        match key_event.code {
                            KeyCode::Esc => {
                                render(&mut stdout, &frame::new_frame(), &frame::new_frame(), true);
                                system.menu = Menu::Main;
                                break 'endgame;
                            },
                            _ => {}
                        }
                    }
                }
                beat_score_display.draw(&mut curr_frame);
                let _ = tx.send(curr_frame);
                thread::sleep(Duration::from_millis(1));
            }
        }
        if system.menu == Menu::Leaderboard {
            let text = match client.get_scores().await {
                Ok(scores) => {
                    let mut tmp = String::from("Space Invaders Leaderboars\n\n");
                    for (i, p) in scores.iter().enumerate() {
                        tmp.push_str(&(" ".to_string() + &(i + 1).to_string().as_str() + ": " + &p.name + " With a Score of " + &p.score.to_string() + "\n\n"))
                    }
                    tmp
                },
                Err(e) => format!("{}", e)
            };
            let leaderboard_menu = NewMenu::new(text, 5, 2);

            'leaderboard: loop {
                let mut curr_frame = frame::new_frame();
                while event::poll(Duration::default())? {
                    if let Event::Key(key_event) = event::read()? {
                        match key_event.code {
                            KeyCode::Esc => {
                                system.menu = Menu::Main;
                                render(&mut stdout, &curr_frame, &curr_frame, true);
                                break 'leaderboard;
                            }
                            _ => {}
                        }
                    }
                }
                leaderboard_menu.draw(&mut curr_frame);
                let _ = tx.send(curr_frame);
                thread::sleep(Duration::from_millis(1));
            }
        }
        if system.menu == Menu::Help {
            let text = 
            "How To Play\n
 Movement: WASD/Arrow Keys\n
 Shoot: Spacebar".to_string();
            let help_menu = NewMenu::new(text, 12, 2);
            'help: loop {
                let mut curr_frame = frame::new_frame();
                while event::poll(Duration::default())? {
                    if let Event::Key(key_event) = event::read()? {
                        match key_event.code {
                            KeyCode::Esc => {
                                system.menu = Menu::Main;
                                render(&mut stdout, &curr_frame, &curr_frame, true);
                                break 'help;
                            }
                            _ => {}
                        }
                    }
                }
                help_menu.draw(&mut curr_frame);
                let _ = tx.send(curr_frame);
                thread::sleep(Duration::from_millis(1));
            }
        }
    }

    // cleanup
    stdout.execute(Show)?;
    stdout.execute(LeaveAlternateScreen)?;
    terminal::disable_raw_mode()?;
    drop(tx);
    handle.join().unwrap();
    audio.close();
    Ok(())
}