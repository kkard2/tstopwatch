use crate::{stopwatch::Stopwatch, AppState};

use crossterm::{cursor, style::Print, ExecutableCommand, QueueableCommand};

pub fn draw(stdout: &mut std::io::Stdout, state: &AppState) -> crossterm::Result<()> {
    Ok(())
}

fn draw_stopwatch(
    stdout: &mut std::io::Stdout,
    cur_stopwatch: &Stopwatch,
) -> crossterm::Result<()> {
    stdout.queue(cursor::MoveTo(0, 0))?;

    let elapsed = cur_stopwatch.elapsed();
    let millis = elapsed.as_millis() % 1000;
    let seconds = elapsed.as_millis() / 1000 % 60;
    let minutes = elapsed.as_millis() / 1000 / 60 % 60;
    let hours = elapsed.as_millis() / 1000 / 60 / 60;
    let elapsed_str = format!("{:02}:{:02}:{:02}.{:03}", hours, minutes, seconds, millis);

    stdout.execute(Print(elapsed_str))?;

    Ok(())
}
