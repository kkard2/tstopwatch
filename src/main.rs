#[macro_use]
extern crate serde_derive;

use std::{
    io::{stdout, Write},
    time::Duration,
};

use crossterm::{
    self, cursor,
    event::{Event, KeyCode, KeyEventKind},
    style::{Color, Colored, Print, SetColors},
    terminal::{EnterAlternateScreen, LeaveAlternateScreen},
    ExecutableCommand, QueueableCommand, Result,
};

use stopwatch::Stopwatch;

#[derive(Default, Serialize, Deserialize, Clone, Copy)]
enum AlignMode {
    NoClear,
    #[default]
    TopLeft,
    BottomLeft,
}

#[derive(Serialize, Deserialize)]
struct Config {
    color: Color,
    update_rate_millis: u64,
    align: AlignMode,
}

impl Default for Config {
    fn default() -> Self {
        Config {
            color: Color::White,
            update_rate_millis: 50,
            align: AlignMode::NoClear,
        }
    }
}

struct DrawContext {
    start_pos: (u16, u16),
    size: (u16, u16),
    align: AlignMode,
}

mod stopwatch;

fn main() -> Result<()> {
    let config: Config = match confy::load("tstopwatch", None) {
        Ok(config) => config,
        Err(e) => {
            eprintln!("Error loading config: {}", e);
            Config::default()
        }
    };

    let update_rate = Duration::from_millis(config.update_rate_millis);
    let mut stdout = stdout();
    crossterm::terminal::enable_raw_mode()?;

    let draw_context = DrawContext {
        start_pos: crossterm::cursor::position()?,
        size: crossterm::terminal::size()?,
        align: config.align,
    };

    stdout
        .queue(cursor::Hide)?
        .queue(SetColors(Colored::ForegroundColor(config.color).into()))?
        .flush()?;

    match config.align {
        AlignMode::NoClear => {}
        _ => {
            stdout.execute(EnterAlternateScreen)?;
        }
    };

    let mut cur_stopwatch = Stopwatch::new();

    loop {
        if crossterm::event::poll(update_rate)? {
            match crossterm::event::read()? {
                Event::Key(key) if key.kind == KeyEventKind::Press => match key.code {
                    KeyCode::Char('q') => break,
                    KeyCode::Char(' ') => {
                        if cur_stopwatch.is_running() {
                            cur_stopwatch.stop();
                        } else {
                            cur_stopwatch.start();
                        }
                    }
                    KeyCode::Char('r') => {
                        cur_stopwatch.reset();
                    }
                    _ => {}
                },
                _ => {}
            }
        }

        draw_stopwatch(&mut stdout, &cur_stopwatch, &draw_context)?;
    }

    match config.align {
        AlignMode::NoClear => {}
        _ => {
            stdout.execute(LeaveAlternateScreen)?;
        }
    };

    stdout.queue(cursor::Show)?.flush()?;
    crossterm::terminal::disable_raw_mode()?;
    Ok(())
}

fn draw_stopwatch(
    stdout: &mut std::io::Stdout,
    cur_stopwatch: &Stopwatch,
    draw_context: &DrawContext,
) -> Result<()> {
    stdout.queue(match draw_context.align {
        AlignMode::NoClear => cursor::MoveTo(draw_context.start_pos.0, draw_context.start_pos.1),
        AlignMode::TopLeft => cursor::MoveTo(0, 0),
        AlignMode::BottomLeft => cursor::MoveTo(0, draw_context.size.1),
    })?;

    let elapsed = cur_stopwatch.elapsed();
    let millis = elapsed.as_millis() % 1000;
    let seconds = elapsed.as_millis() / 1000 % 60;
    let minutes = elapsed.as_millis() / 1000 / 60 % 60;
    let hours = elapsed.as_millis() / 1000 / 60 / 60;
    let elapsed_str = format!("{:02}:{:02}:{:02}.{:03}", hours, minutes, seconds, millis);

    stdout.queue(Print(elapsed_str))?;
    stdout.flush()?;

    Ok(())
}
