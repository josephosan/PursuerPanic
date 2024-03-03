use std::{
    io::{stdout, Stdout, Write},
    time::{Duration, Instant},
};

use crossterm::{
    cursor::{Hide, MoveTo, Show},
    event::{self, poll, read, Event, KeyCode},
    execute,
    style::{Color, Print, ResetColor, SetBackgroundColor, SetForegroundColor},
    terminal::{self, disable_raw_mode, enable_raw_mode, Clear, ClearType},
    ExecutableCommand, QueueableCommand,
};
use rand::Rng;
use tui::{widgets::Block, Terminal};

struct Board {
    // board size
    r: u16,
    c: u16,

    // default user
    cursor_r: u16,
    cursor_c: u16,

    // all killers
    killers: Killers,
}

struct Killers {
    // killer 1
    k1_c: i16,
    k1_r: i16,
    // killer 2
    k2_c: i16,
    k2_r: i16,
    // killer 3
    k3_c: i16,
    k3_r: i16,
}

fn killer_move(board: &Board, c: i16, r: i16) -> i16 {
    let mut interval: i16 = 0;
    if c > 0 {
        interval += board.cursor_c as i16 - c as i16;
    } else {
        interval += board.cursor_r as i16 - r as i16;
    }

    if interval < 0 {
        -1
    } else {
        1
    }
}

fn move_killers(board: &Board) -> Killers {
    let mut _killers = Killers {
        k1_c: board.killers.k1_c + killer_move(&board, board.killers.k1_c, 0),
        k1_r: board.killers.k1_r + killer_move(&board, 0, board.killers.k1_r),

        k2_c: board.killers.k2_c + killer_move(&board, board.killers.k2_c, 0),
        k2_r: board.killers.k2_r + killer_move(&board, 0, board.killers.k2_r),

        k3_c: board.killers.k3_c + killer_move(&board, board.killers.k3_c, 0),
        k3_r: board.killers.k3_r + killer_move(&board, 0, board.killers.k3_r),
    };

    _killers
}

fn game_over(mut board: &Board) -> bool {
    if (board.killers.k1_c == board.cursor_c as i16 && board.killers.k1_r == board.cursor_r as i16)
        || (board.killers.k2_c == board.cursor_c as i16
            && board.killers.k2_r == board.cursor_r as i16)
        || (board.killers.k3_c == board.cursor_c as i16
            && board.killers.k3_r == board.cursor_r as i16)
    {
        true
    } else {
        false
    }
}

fn draw(mut scr: &Stdout, mut board: &mut Board, killers_update: bool) -> std::io::Result<()> {
    if killers_update {
        board.killers = move_killers(&board);
    }

    scr.queue(Clear(ClearType::All))?;
    scr.queue(MoveTo(board.cursor_c, board.cursor_r))?;
    scr.queue(Print("0"))?;
    scr.queue(Hide)?;

    // killer 1
    scr.queue(MoveTo(board.killers.k1_c as u16, board.killers.k1_r as u16))?;
    scr.queue(Print("X"))?;

    // killer 2
    scr.queue(MoveTo(board.killers.k2_c as u16, board.killers.k2_r as u16))?;
    scr.queue(Print("X"))?;

    // killer 3
    scr.queue(MoveTo(board.killers.k3_c as u16, board.killers.k3_r as u16))?;
    scr.queue(Print("X"))?;

    scr.flush()?;

    Ok(())
}

fn gen_rand_killers(r: u16, c: u16) -> Killers {
    let mut rng = rand::thread_rng();
    let killers = Killers {
        k1_c: rng.gen_range(0..c) as i16,
        k1_r: rng.gen_range(0..r) as i16,

        k2_c: rng.gen_range(0..c) as i16,
        k2_r: rng.gen_range(0..r) as i16,

        k3_c: rng.gen_range(0..c) as i16,
        k3_r: rng.gen_range(0..r) as i16,
    };

    killers
}

fn main() -> std::io::Result<()> {
    let mut scr = stdout();
    let (c, r) = terminal::size().unwrap();
    enable_raw_mode()?;

    let killers = gen_rand_killers(r, c);
    let mut board = Board {
        r,
        c,
        cursor_r: r / 2 - 1,
        cursor_c: c / 2,
        killers,
    };

    draw(&scr, &mut board, false)?;
    let mut last_execution_time = Instant::now();
    let mut last_killer_generate_time = Instant::now();

    let mut killers_speed: u16 = 0;

    loop {
        if Instant::now() - last_execution_time >= Duration::from_millis(1) {
            if game_over(&board) {
                scr.queue(Clear(ClearType::All))?;
                scr.queue(MoveTo(board.c / 2 - 5, board.r / 2))?;
                scr.queue(Print("GAME OVER!"))?;
                break;
            }

            if killers_speed < 1 {
                killers_speed = 70; // increase to make them slower.
                draw(&scr, &mut board, true)?;
            } else {
                draw(&scr, &mut board, false)?;
            }

            killers_speed -= 1;
            last_execution_time = Instant::now();
        }

        if Instant::now() - last_killer_generate_time >= Duration::from_millis(5000) {
            board.killers = gen_rand_killers(r, c);
            last_killer_generate_time = Instant::now();
        }

        if poll(Duration::from_millis(1))? {
            let key = read().unwrap();
            match key {
                Event::Key(ev) => match ev.code {
                    KeyCode::Up => {
                        if board.cursor_r > 0 {
                            board.cursor_r -= 1
                        };
                    }
                    KeyCode::Down => {
                        if board.cursor_r < r {
                            board.cursor_r += 1
                        };
                    }
                    KeyCode::Left => {
                        if board.cursor_c > 0 {
                            board.cursor_c -= 1
                        };
                    }
                    KeyCode::Right => {
                        if board.cursor_c < c - 1 {
                            board.cursor_c += 1
                        };
                    }
                    KeyCode::Char('q') => {
                        break;
                    }
                    _ => {}
                },
                _ => {}
            }
        } else {
        }
    }

    disable_raw_mode()?;
    scr.queue(Show)?;
    scr.flush()?;
    Ok(())
}
