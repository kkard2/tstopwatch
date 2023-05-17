use std::io::Write;

use crate::{stopwatch::Stopwatch, AppState};

use crossterm::{
    cursor::{Hide, MoveTo},
    style::{Color, Colors, Print, SetColors},
    terminal::{Clear, ClearType},
    QueueableCommand,
};

const NORMAL_COLORS: Colors = Colors {
    foreground: Some(Color::White),
    background: Some(Color::Black),
};
const HIGHLIGHT_COLORS: Colors = Colors {
    foreground: Some(Color::Yellow),
    background: Some(Color::Black),
};
const TIMESTAMP_COLORS: Colors = Colors {
    foreground: Some(Color::Cyan),
    background: Some(Color::Black),
};

pub fn draw(stdout: &mut std::io::Stdout, state: &AppState) -> crossterm::Result<()> {
    let size = crossterm::terminal::size()?;
    stdout
        .queue(Clear(ClearType::All))?
        .queue(Hide)?
        .queue(MoveTo(0, 0))?;

    for (i, stack) in state.stacks().iter().enumerate() {
        if i > (size.1 - 2) as usize {
            break;
        }
        stdout.queue(SetColors(if state.current_stack_index() == i {
            HIGHLIGHT_COLORS
        } else {
            NORMAL_COLORS
        }))?;
        stdout.queue(MoveTo(0, i as u16))?;
        draw_stopwatch(stdout, stack.current())?;
    }

    stdout
        .queue(SetColors(TIMESTAMP_COLORS))?
        .queue(MoveTo(0, size.1 - 1))?;

    draw_timestamp(stdout, &chrono::Local::now())?;
    stdout.queue(SetColors(NORMAL_COLORS))?;
    stdout.flush()
}

fn draw_stopwatch(
    stdout: &mut std::io::Stdout,
    cur_stopwatch: &Stopwatch,
) -> crossterm::Result<()> {
    let elapsed = cur_stopwatch.elapsed();
    let millis = elapsed.as_millis() % 1000;
    let seconds = elapsed.as_millis() / 1000 % 60;
    let minutes = elapsed.as_millis() / 1000 / 60 % 60;
    let hours = elapsed.as_millis() / 1000 / 60 / 60;
    let elapsed_str = format!("{:02}:{:02}:{:02}.{:03}", hours, minutes, seconds, millis);

    stdout.queue(Print(elapsed_str))?;

    Ok(())
}

fn draw_timestamp(
    stdout: &mut std::io::Stdout,
    timestamp: &chrono::DateTime<chrono::Local>,
) -> crossterm::Result<()> {
    let timestamp_str = format!("{}", timestamp.format("%Y-%m-%d %H:%M:%S"));

    stdout.queue(Print(timestamp_str))?;

    Ok(())
}
