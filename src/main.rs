#[macro_use]
extern crate serde_derive;

use std::{
    io::{stdout, Write},
    path::PathBuf,
    time::Duration,
};

use crossterm::{
    self, cursor,
    event::{Event, KeyCode, KeyEventKind, KeyModifiers},
    terminal::{EnterAlternateScreen, LeaveAlternateScreen},
    QueueableCommand, Result,
};

use stopwatch::StopwatchStack;

#[derive(Serialize, Deserialize)]
pub struct AppState {
    stacks: Vec<StopwatchStack>,
    current_stack_index: usize,
    config: Config,
    deleted_stacks: Vec<StopwatchStack>,
}

impl AppState {
    pub fn stacks(&self) -> &[StopwatchStack] {
        self.stacks.as_ref()
    }

    pub fn current_stack_index(&self) -> usize {
        self.current_stack_index
    }

    pub fn config(&self) -> &Config {
        &self.config
    }

    pub fn current_stack(&self) -> &StopwatchStack {
        &self.stacks[self.current_stack_index]
    }

    pub fn current_stack_mut(&mut self) -> &mut StopwatchStack {
        &mut self.stacks[self.current_stack_index]
    }
}

impl Default for AppState {
    fn default() -> Self {
        AppState {
            stacks: vec![StopwatchStack::default()],
            current_stack_index: 0,
            config: Default::default(),
            deleted_stacks: Default::default(),
        }
    }
}

#[derive(Serialize, Deserialize)]
pub struct Config {
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
        Err(_) => Config::default(),
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
        let mut changed = false;

        if crossterm::event::poll(update_rate)? {
            match crossterm::event::read()? {
                Event::Key(key) if key.kind == KeyEventKind::Press => match key.code {
                    // quit
                    KeyCode::Char('q') if key.modifiers.is_empty() => break,
                    KeyCode::Char('c') if key.modifiers == KeyModifiers::CONTROL => break,
                    // pause
                    KeyCode::Char(' ') if key.modifiers.is_empty() => {
                        app_state.current_stack_mut().push();
                        let cur_stopwatch = app_state.current_stack_mut().current_mut();
                        if cur_stopwatch.is_running() {
                            cur_stopwatch.stop();
                            changed = true;
                        } else {
                            cur_stopwatch.start();
                            changed = true;
                        }
                    }
                    // restart
                    KeyCode::Char('r') if key.modifiers.is_empty() => {
                        app_state.current_stack_mut().push();
                        let cur_stopwatch = app_state.current_stack_mut().current_mut();
                        cur_stopwatch.reset();
                        changed = true;
                    }
                    // undo/redo
                    KeyCode::Char('u') if key.modifiers.is_empty() => {
                        app_state.current_stack_mut().undo();
                        changed = true;
                    }
                    KeyCode::Char('r') if key.modifiers == KeyModifiers::CONTROL => {
                        app_state.current_stack_mut().redo();
                        changed = true;
                    }
                    KeyCode::Char('U') if key.modifiers == KeyModifiers::SHIFT => {
                        let deleted = app_state.deleted_stacks.pop();
                        if let Some(deleted) = deleted {
                            app_state.stacks.push(deleted);
                            changed = true;
                        }
                    }
                    // new stack
                    KeyCode::Char('o') if key.modifiers.is_empty() => {
                        app_state
                            .stacks
                            .insert(app_state.current_stack_index() + 1, Default::default());
                        app_state.current_stack_index += 1;
                        changed = true;
                    }
                    KeyCode::Char('O') if key.modifiers == KeyModifiers::SHIFT => {
                        app_state
                            .stacks
                            .insert(app_state.current_stack_index(), Default::default());
                        changed = true;
                    }
                    // moving
                    KeyCode::Char('j') if key.modifiers.is_empty() => {
                        app_state.current_stack_index =
                            (app_state.current_stack_index() + 1) % app_state.stacks.len();
                        changed = true;
                    }
                    KeyCode::Char('k') if key.modifiers.is_empty() => {
                        app_state.current_stack_index =
                            (app_state.current_stack_index() + app_state.stacks.len() - 1)
                                % app_state.stacks.len();
                        changed = true;
                    }
                    // delete stack
                    KeyCode::Char('d') if key.modifiers.is_empty() => {
                        // TODO: don't make a memory leak
                        if app_state.stacks.len() > 1 {
                            let removed = app_state.stacks.remove(app_state.current_stack_index());
                            app_state.current_stack_index = app_state.current_stack_index.min(app_state.stacks.len() - 1);
                            app_state.deleted_stacks.push(removed);
                            changed = true;
                        }
                    }
                    _ => {}
                },
                _ => {}
            }
        }

        draw::draw(&mut stdout, &app_state)?;

        if changed {
            store_state(&config.state_file, &app_state)?;
        }
    }

    store_state(&config.state_file, &app_state)?;

    stdout
        .queue(LeaveAlternateScreen)?
        .queue(cursor::Show)?
        .flush()?;

    crossterm::terminal::disable_raw_mode()?;
    Ok(())
}

fn load_state(path: &PathBuf) -> AppState {
    match std::fs::read_to_string(path) {
        Ok(content) => match serde_json::from_str::<AppState>(&content) {
            Ok(state) => state,
            Err(_) => AppState::default(),
        },
        Err(_) => AppState::default(),
    }
}

fn store_state(path: &PathBuf, state: &AppState) -> Result<()> {
    let str = serde_json::to_string(&state)?;
    std::fs::write(path, str)?;
    Ok(())
}
