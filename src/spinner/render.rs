// src/spinner/render.rs
use super::state::{HudState, ATOMIC_LINES};
use crossterm::{cursor, execute, terminal::{Clear, ClearType}};
use std::{
    collections::VecDeque,
    io::{self, Write},
    sync::{Arc, Mutex, atomic::{AtomicBool, Ordering}},
    thread,
    time::{Duration, Instant},
};

const FRAMES: &[&str] = &["⠋", "⠙", "⠹", "⠸", "⠼", "⠴", "⠦", "⠧", "⠇", "⠏"];
const INTERVAL: u64 = 80;

trait SimpleColor {
    fn cyan(&self) -> String;
    fn yellow(&self) -> String;
    fn green(&self) -> String;
    fn red(&self) -> String;
    fn white(&self) -> String;
    fn bold(&self) -> String;
    fn dimmed(&self) -> String;
}

impl SimpleColor for str {
    fn cyan(&self) -> String { format!("\x1b[36m{self}\x1b[0m") }
    fn yellow(&self) -> String { format!("\x1b[33m{self}\x1b[0m") }
    fn green(&self) -> String { format!("\x1b[32m{self}\x1b[0m") }
    fn red(&self) -> String { format!("\x1b[31m{self}\x1b[0m") }
    fn white(&self) -> String { format!("\x1b[37m{self}\x1b[0m") }
    fn bold(&self) -> String { format!("\x1b[1m{self}\x1b[0m") }
    fn dimmed(&self) -> String { format!("\x1b[2m{self}\x1b[0m") }
}

pub fn run_hud_loop(running: &Arc<AtomicBool>, state: &Arc<Mutex<HudState>>) {
    let mut frame_idx = 0;
    let mut stdout = io::stdout();

    let _ = execute!(stdout, cursor::Hide);

    let _ = writeln!(stdout); 
    let _ = writeln!(stdout); 
    for _ in 0..ATOMIC_LINES { let _ = writeln!(stdout); }
    
    let height = u16::try_from(ATOMIC_LINES + 2).unwrap_or(5);
    let _ = execute!(stdout, cursor::MoveUp(height));

    while running.load(Ordering::Relaxed) {
        let snapshot = if let Ok(guard) = state.lock() {
            Some(guard.snapshot())
        } else {
            None
        };

        if let Some((title, micro, atomic, start, progress)) = snapshot {
            render_frame(&mut stdout, &title, &micro, &atomic, start, progress, frame_idx);
        }
        
        thread::sleep(Duration::from_millis(INTERVAL));
        frame_idx += 1;
    }

    let _ = execute!(stdout, cursor::Show);
    
    if let Ok(guard) = state.lock() {
        let _ = clear_lines(ATOMIC_LINES + 2);
        let (success, title, start) = guard.completion_info();
        print_final_status(success, title, start.elapsed());
    }
}

fn render_frame(
    stdout: &mut io::Stdout,
    title: &str,
    micro: &str,
    atomic: &VecDeque<String>,
    start: Instant,
    progress: Option<(usize, usize)>,
    frame_idx: usize
) {
    let spinner = FRAMES.get(frame_idx % FRAMES.len()).unwrap_or(&"+");
    let elapsed = start.elapsed().as_secs();
    
    let _ = execute!(stdout, Clear(ClearType::CurrentLine));
    let title_line = format!(
        "{spinner} {} ({elapsed}s)",
        title.cyan().bold()
    );
    let _ = writeln!(stdout, "{title_line}");

    let _ = execute!(stdout, Clear(ClearType::CurrentLine));
    
    let status_text = if let Some((curr, total)) = progress {
        if total > 0 {
            #[allow(clippy::cast_precision_loss)]
            let pct = (curr as f64 / total as f64) * 100.0;
            let bar_width: usize = 20;
            
            #[allow(clippy::cast_precision_loss, clippy::cast_possible_truncation, clippy::cast_sign_loss)]
            let filled = ((pct / 100.0) * (bar_width as f64)) as usize;
            
            let empty = bar_width.saturating_sub(filled);
            let bar = format!("{}{}", "█".repeat(filled).cyan(), "░".repeat(empty).dimmed());
            format!("{bar} {pct:.0}% - {micro}")
        } else {
            micro.to_string()
        }
    } else {
        micro.to_string()
    };

    let micro_line = format!(
        "   {} {}",
        "›".yellow().bold(),
        status_text.white()
    );
    let _ = writeln!(stdout, "{micro_line}");

    for i in 0..ATOMIC_LINES {
        let _ = execute!(stdout, Clear(ClearType::CurrentLine));
        let content = atomic.get(i).map_or("", String::as_str);
        
        let trunc_len = 80;
        let safe_content = if content.len() > trunc_len {
            let mut end = trunc_len;
            while !content.is_char_boundary(end) {
                end = end.saturating_sub(1);
            }
            &content[..end]
        } else {
            content
        };
        
        let _ = writeln!(stdout, "     {}", safe_content.dimmed());
    }

    let height = u16::try_from(ATOMIC_LINES + 2).unwrap_or(5);
    let _ = execute!(stdout, cursor::MoveUp(height));
}

fn clear_lines(count: usize) -> io::Result<()> {
    let mut stdout = io::stdout();
    for _ in 0..count {
        execute!(stdout, Clear(ClearType::CurrentLine))?;
        writeln!(stdout)?;
    }
    let height = u16::try_from(count).unwrap_or(5);
    execute!(stdout, cursor::MoveUp(height))?;
    Ok(())
}

fn print_final_status(success: bool, title: &str, duration: Duration) {
    let icon = if success { "ok".green() } else { "err".red() };
    let time = format!("{}s", duration.as_secs()).dimmed();
    println!("   {icon} {title} ({time})");
}