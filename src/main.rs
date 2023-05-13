#[macro_use]
extern crate serde_derive;

use std::{
    io::{stdout, Write},
    path::PathBuf,
    time::Duration,
};

use crossterm::{
    self, cursor,
    event::{Event, KeyCode, KeyEventKind},
    style::{Color, Colored, Print, SetColors},
    terminal::{EnterAlternateScreen, LeaveAlternateScreen},
    ExecutableCommand, QueueableCommand, Result,
};

use stopwatch::{Stopwatch, StopwatchSerializable};

pub struct AppState {}

#[derive(Serialize, Deserialize)]
struct Config {
    update_rate_millis: u64,
    state_file: PathBuf,
}

impl Default for Config {
    fn default() -> Self {
        Config {
            update_rate_millis: 50,
            state_file: PathBuf::from("stopwatch.json"),
        }
    }
}

mod draw;
mod stopwatch;

const APP_NAME: &str = "tstopwatch";

fn main() -> Result<()> {
    let config: Config = match confy::load(APP_NAME, None) {
        Ok(config) => config,
        Err(e) => {
            eprintln!("error loading config: {}", e);
            Config::default()
        }
    };

    let update_rate = Duration::from_millis(config.update_rate_millis);
    let mut stdout = stdout();
    crossterm::terminal::enable_raw_mode()?;

    stdout
        .queue(cursor::Hide)?
        .queue(EnterAlternateScreen)?
        .flush()?;

    let mut app_state = load_state(&config.state_file);

    loop {
        if crossterm::event::poll(update_rate)? {
            match crossterm::event::read()? {
                Event::Key(key) if key.kind == KeyEventKind::Press => match key.code {
                    KeyCode::Char('q') => break,
                    KeyCode::Char(' ') => {
                        if cur_stopwatch.is_running() {
                            cur_stopwatch.stop();
                            store_state(&config.state_file, &cur_stopwatch)
                                .expect("epic file save fail");
                        } else {
                            cur_stopwatch.start();
                            store_state(&config.state_file, &cur_stopwatch)
                                .expect("epic file save fail");
                        }
                    }
                    KeyCode::Char('r') => {
                        cur_stopwatch.reset();
                        store_state(&config.state_file, &cur_stopwatch)
                            .expect("epic file save fail");
                    }
                    _ => {}
                },
                _ => {}
            }
        }

        draw::draw(&mut stdout, &cur_stopwatch)?;
    }

    store_state(&config.state_file, &cur_stopwatch).expect("epic file save fail");

    stdout
        .queue(LeaveAlternateScreen)?
        .queue(cursor::Show)?
        .flush()?;

    crossterm::terminal::disable_raw_mode()?;
    Ok(())
}

fn load_state(path: &PathBuf) -> AppStateSerializable {
    match std::fs::read_to_string(path) {
        Ok(content) => match serde_json::from_str::<AppStateSerializable>(&content) {
            Ok(state) => state.into(),
            Err(_) => AppStateSerializable::default(),
        },
        Err(_) => AppStateSerializable::default(),
    }
}

fn store_state(path: &PathBuf, state: &Stopwatch) -> Result<()> {
    let serialized = StopwatchSerializable::from(state);
    let serialized_str = serde_json::to_string(&serialized)?;
    std::fs::write(path, serialized_str)?;
    Ok(())
}
