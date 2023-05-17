use crate::{stopwatch::Stopwatch, AppState};

use crossterm::{
    cursor::{self, MoveTo},
    style::{Color, Colors, Print, SetColors},
    terminal::{Clear, ClearType},
    ExecutableCommand, QueueableCommand,
};

pub fn draw(stdout: &mut std::io::Stdout, state: &AppState) -> crossterm::Result<()> {
    stdout
        .queue(Clear(ClearType::All))?
        .queue(SetColors(Colors::new(Color::White, Color::Black)))?
        .queue(MoveTo(0, 0))?;

    for (i, stack) in state.stacks().iter().enumerate() {
        stdout.queue(SetColors(if state.current_stack() == i {
            Colors::new(Color::Black, Color::White)
        } else {
            Colors::new(Color::White, Color::Black)
        }))?;
        draw_stopwatch(stdout, stack.current())?;
        stdout.queue(MoveTo(0, i as u16))?;
    }

    Ok(())
}

fn draw_stopwatch(
    stdout: &mut std::io::Stdout,
    cur_stopwatch: &Stopwatch,
) -> crossterm::Result<()> {
    stdout.queue(MoveTo(0, 0))?;

    let elapsed = cur_stopwatch.elapsed();
    let millis = elapsed.as_millis() % 1000;
    let seconds = elapsed.as_millis() / 1000 % 60;
    let minutes = elapsed.as_millis() / 1000 / 60 % 60;
    let hours = elapsed.as_millis() / 1000 / 60 / 60;
    let elapsed_str = format!("{:02}:{:02}:{:02}.{:03}", hours, minutes, seconds, millis);

    stdout.queue(Print(elapsed_str))?;

    Ok(())
}
