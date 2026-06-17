use crossterm::{
    cursor, execute, queue,
    style::{Color, Print, ResetColor, SetForegroundColor},
    terminal::{self, Clear, ClearType, EnterAlternateScreen, LeaveAlternateScreen},
    event::{self, Event, KeyCode, KeyModifiers},
};
use rand::Rng;
use std::io::{stdout, Write};
use std::time::{Duration, Instant};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

const CHAR_SET: &[char] = &[
    '0', '1', '2', '3', '4', '5', '6', '7', '8', '9',
    'ア', 'イ', 'ウ', 'エ', 'オ', 'カ', 'キ', 'ク', 'ケ', 'コ',
    'サ', 'シ', 'ス', 'セ', 'ソ', 'タ', 'チ', 'ツ', 'テ', 'ト',
    'ナ', 'ニ', 'ヌ', 'ネ', 'ノ', 'ハ', 'ヒ', 'フ', 'ヘ', 'ホ',
    'マ', 'ミ', 'ム', 'メ', 'モ', 'ヤ', 'ユ', 'ヨ', 'ラ', 'リ',
    'ル', 'レ', 'ロ', 'ワ', 'ヲ', 'ン',
];

struct Column {
    x: u16,
    y: i16,
    length: u16,
    speed: u16,
    chars: Vec<char>,
    last_update: Instant,
    trail_chars: Vec<(u16, char, u8)>, // (y, char, intensity)
}

impl Column {
    fn new(x: u16, height: u16, rng: &mut impl Rng) -> Self {
        let length = rng.gen_range(8..=20).min(height);
        let speed = rng.gen_range(30..=120);
        let chars: Vec<char> = (0..length).map(|_| *CHAR_SET.choose(rng).unwrap()).collect();
        Column {
            x,
            y: -(length as i16),
            length,
            speed,
            chars,
            last_update: Instant::now(),
            trail_chars: Vec::new(),
        }
    }

    fn update(&mut self, height: u16, rng: &mut impl Rng, config: &Config) {
        if self.last_update.elapsed() < Duration::from_millis(self.speed as u64 / config.speed_divisor) {
            return;
        }
        self.last_update = Instant::now();

        // Shift trail
        for (y, _, intensity) in &mut self.trail_chars {
            if *intensity > 0 {
                *intensity = intensity.saturating_sub(config.fade_rate);
            }
        }
        self.trail_chars.retain(|(_, _, i)| *i > 0);

        // Add new head position to trail
        if self.y >= 0 && (self.y as u16) < height {
            self.trail_chars.push((self.y as u16, self.chars[0], 255));
        }

        // Move down
        self.y += 1;

        // Rotate chars
        self.chars.rotate_left(1);
        self.chars[self.length as usize - 1] = *CHAR_SET.choose(rng).unwrap();

        // Reset if fully off screen
        if self.y > height as i16 + self.length as i16 + 10 {
            self.y = -(self.length as i16) - rng.gen_range(0..=20);
            self.length = rng.gen_range(8..=20).min(height);
            self.speed = rng.gen_range(30..=120);
            self.chars = (0..self.length).map(|_| *CHAR_SET.choose(rng).unwrap()).collect();
            self.trail_chars.clear();
        }
    }
}

struct Config {
    density: f32,
    speed_divisor: u16,
    fade_rate: u8,
    color_scheme: ColorScheme,
    show_fps: bool,
}

#[derive(Clone, Copy, PartialEq)]
enum ColorScheme {
    ClassicGreen,
    Amber,
    Blue,
    Red,
    Rainbow,
}

impl ColorScheme {
    fn head_color(&self) -> Color {
        match self {
            ColorScheme::ClassicGreen => Color::Rgb { r: 0, g: 255, b: 70 },
            ColorScheme::Amber => Color::Rgb { r: 255, g: 191, b: 0 },
            ColorScheme::Blue => Color::Rgb { r: 0, g: 180, b: 255 },
            ColorScheme::Red => Color::Rgb { r: 255, g: 50, b: 50 },
            ColorScheme::Rainbow => Color::Rgb { r: 255, g: 255, b: 255 },
        }
    }

    fn trail_color(&self, intensity: u8, y: u16) -> Color {
        let factor = intensity as f32 / 255.0;
        match self {
            ColorScheme::ClassicGreen => Color::Rgb {
                r: 0,
                g: (70.0 + 185.0 * factor) as u8,
                b: 0,
            },
            ColorScheme::Amber => Color::Rgb {
                r: (100.0 + 155.0 * factor) as u8,
                g: (50.0 + 141.0 * factor) as u8,
                b: 0,
            },
            ColorScheme::Blue => Color::Rgb {
                r: 0,
                g: (50.0 + 130.0 * factor) as u8,
                b: (100.0 + 155.0 * factor) as u8,
            },
            ColorScheme::Red => Color::Rgb {
                r: (100.0 + 155.0 * factor) as u8,
                g: 0,
                b: (20.0 * factor) as u8,
            },
            ColorScheme::Rainbow => {
                let hue = (y as f32 * 0.1 + intensity as f32 * 0.01) % 360.0;
                hsv_to_rgb(hue, 1.0, factor)
            }
        }
    }

    fn name(&self) -> &'static str {
        match self {
            ColorScheme::ClassicGreen => "Classic Green",
            ColorScheme::Amber => "Amber",
            ColorScheme::Blue => "Blue",
            ColorScheme::Red => "Red",
            ColorScheme::Rainbow => "Rainbow",
        }
    }

    fn next(&self) -> Self {
        match self {
            ColorScheme::ClassicGreen => ColorScheme::Amber,
            ColorScheme::Amber => ColorScheme::Blue,
            ColorScheme::Blue => ColorScheme::Red,
            ColorScheme::Red => ColorScheme::Rainbow,
            ColorScheme::Rainbow => ColorScheme::ClassicGreen,
        }
    }
}

fn hsv_to_rgb(h: f32, s: f32, v: f32) -> Color {
    let c = v * s;
    let x = c * (1.0 - ((h / 60.0) % 2.0 - 1.0).abs());
    let m = v - c;
    let (r, g, b) = if h < 60.0 { (c, x, 0.0) }
    else if h < 120.0 { (x, c, 0.0) }
    else if h < 180.0 { (0.0, c, x) }
    else if h < 240.0 { (0.0, x, c) }
    else if h < 300.0 { (x, 0.0, c) }
    else { (c, 0.0, x) };
    Color::Rgb {
        r: ((r + m) * 255.0) as u8,
        g: ((g + m) * 255.0) as u8,
        b: ((b + m) * 255.0) as u8,
    }
}

fn draw_ui(stdout: &mut std::io::Stdout, width: u16, height: u16, config: &Config, fps: f32, frame: u64) -> crossterm::Result<()> {
    queue!(stdout, cursor::MoveTo(0, height))?;
    queue!(stdout, SetForegroundColor(Color::Rgb { r: 100, g: 100, b: 100 }))?;
    queue!(stdout, Print("─".repeat(width as usize)))?;
    
    let controls = "[q]uit  [←/→] density  [↑/↓] speed  [c] color  [f] FPS  [r] reset  [space] pause";
    let status = format!(" Density: {:.0}% | Speed: {}x | Color: {} | FPS: {:.1} | Frame: {} ",
        config.density * 100.0,
        config.speed_divisor,
        config.color_scheme.name(),
        fps,
        frame
    );
    
    let x = (width.saturating_sub(controls.len() as u16)) / 2;
    queue!(stdout, cursor::MoveTo(x, height + 1))?;
    queue!(stdout, Print(controls))?;
    
    let x = (width.saturating_sub(status.len() as u16)) / 2;
    queue!(stdout, cursor::MoveTo(x, height + 2))?;
    queue!(stdout, Print(status))?;
    
    queue!(stdout, ResetColor)?;
    Ok(())
}

fn render_frame(
    stdout: &mut std::io::Stdout,
    columns: &[Column],
    width: u16,
    height: u16,
    config: &Config,
) -> crossterm::Result<()> {
    queue!(stdout, Clear(ClearType::All))?;
    
    // Render trails and heads
    for col in columns {
        // Trail
        for (y, ch, intensity) in &col.trail_chars {
            if *y < height {
                queue!(stdout, cursor::MoveTo(col.x, *y))?;
                queue!(stdout, SetForegroundColor(config.color_scheme.trail_color(*intensity, *y)))?;
                queue!(stdout, Print(*ch))?;
            }
        }
        // Head
        if col.y >= 0 && (col.y as u16) < height {
            queue!(stdout, cursor::MoveTo(col.x, col.y as u16))?;
            queue!(stdout, SetForegroundColor(config.color_scheme.head_color()))?;
            queue!(stdout, Print(col.chars[0]))?;
        }
    }
    
    queue!(stdout, ResetColor)?;
    stdout.flush()?;
    Ok(())
}

fn main() -> crossterm::Result<()> {
    let mut stdout = stdout();
    
    terminal::enable_raw_mode()?;
    execute!(stdout, EnterAlternateScreen, cursor::Hide)?;
    
    let (width, height) = terminal::size()?;
    let draw_height = height.saturating_sub(3);
    
    let mut rng = rand::thread_rng();
    let num_columns = ((width as f32 * 0.35) as u16).max(10);
    let mut columns: Vec<Column> = (0..num_columns)
        .filter_map(|_| {
            if rng.gen::<f32>() < 0.35 {
                Some(Column::new(rng.gen_range(0..width), draw_height, &mut rng))
            } else {
                None
            }
        })
        .collect();
    
    // Fill remaining columns
    while columns.len() < num_columns {
        columns.push(Column::new(rng.gen_range(0..width), draw_height, &mut rng));
    }
    
    let mut config = Config {
        density: 0.35,
        speed_divisor: 1,
        fade_rate: 12,
        color_scheme: ColorScheme::ClassicGreen,
        show_fps: true,
    };
    
    let running = Arc::new(AtomicBool::new(true));
    let r = running.clone();
    ctrlc::set_handler(move || {
        r.store(false, Ordering::SeqCst);
    }).ok();
    
    let mut last_fps_check = Instant::now();
    let mut frame_count = 0u64;
    let mut fps = 0.0;
    let mut paused = false;
    
    while running.load(Ordering::SeqCst) {
        let frame_start = Instant::now();
        
        // Handle events
        while event::poll(Duration::from_millis(0))? {
            match event::read()? {
                Event::Key(key) => match key.code {
                    KeyCode::Char('q') | KeyCode::Char('Q') => {
                        running.store(false, Ordering::SeqCst);
                    }
                    KeyCode::Char('c') | KeyCode::Char('C') => {
                        config.color_scheme = config.color_scheme.next();
                    }
                    KeyCode::Char('f') | KeyCode::Char('F') => {
                        config.show_fps = !config.show_fps;
                    }
                    KeyCode::Char('r') | KeyCode::Char('R') => {
                        columns.clear();
                        for _ in 0..num_columns {
                            columns.push(Column::new(rng.gen_range(0..width), draw_height, &mut rng));
                        }
                    }
                    KeyCode::Char(' ') => {
                        paused = !paused;
                    }
                    KeyCode::Left => {
                        config.density = (config.density - 0.05).max(0.05);
                        // Adjust column count
                        let target = ((width as f32 * config.density) as usize).max(5);
                        if columns.len() > target {
                            columns.truncate(target);
                        } else while columns.len() < target {
                            columns.push(Column::new(rng.gen_range(0..width), draw_height, &mut rng));
                        }
                    }
                    KeyCode::Right => {
                        config.density = (config.density + 0.05).min(0.95);
                        let target = ((width as f32 * config.density) as usize).max(5);
                        while columns.len() < target {
                            columns.push(Column::new(rng.gen_range(0..width), draw_height, &mut rng));
                        }
                    }
                    KeyCode::Up => {
                        config.speed_divisor = (config.speed_divisor + 1).min(10);
                    }
                    KeyCode::Down => {
                        config.speed_divisor = (config.speed_divisor - 1).max(1);
                    }
                    KeyCode::Char('+') | KeyCode::Char('=') => {
                        config.fade_rate = config.fade_rate.saturating_add(2).min(50);
                    }
                    KeyCode::Char('-') | KeyCode::Char('_') => {
                        config.fade_rate = config.fade_rate.saturating_sub(2).max(2);
                    }
                    _ => {}
                },
                Event::Resize(w, h) => {
                    // Columns will adapt on next frame
                }
                _ => {}
            }
        }
        
        if !paused {
            for col in &mut columns {
                col.update(draw_height, &mut rng, &config);
            }
        }
        
        let (w, h) = terminal::size()?;
        render_frame(&mut stdout, &columns, w, h.saturating_sub(3), &config)?;
        
        if config.show_fps {
            draw_ui(&mut stdout, w, h.saturating_sub(3), &config, fps, frame_count)?;
        }
        
        frame_count += 1;
        if last_fps_check.elapsed() >= Duration::from_secs(1) {
            fps = frame_count as f32 / last_fps_check.elapsed().as_secs_f32();
            frame_count = 0;
            last_fps_check = Instant::now();
        }
        
        // Target ~60 FPS
        let elapsed = frame_start.elapsed();
        let target = Duration::from_millis(16);
        if elapsed < target {
            std::thread::sleep(target - elapsed);
        }
    }
    
    execute!(stdout, LeaveAlternateScreen, cursor::Show)?;
    terminal::disable_raw_mode()?;
    Ok(())
}

trait ChooseExt {
    fn choose(&mut self) -> Option<&char>;
}

impl ChooseExt for rand::rngs::ThreadRng {
    fn choose(&mut self) -> Option<&char> {
        CHAR_SET.get(self.gen_range(0..CHAR_SET.len()))
    }
}