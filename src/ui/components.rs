use tui::widgets::{Block, Borders, Gauge, Paragraph};
use tui::layout::Rect;
use tui::style::{Color, Style};
use tui::text::Span;

pub fn create_gauge(title: &str, value: u16, color: Color) -> Gauge {
    Gauge::default()
        .block(Block::default().title(title).borders(Borders::ALL))
        .gauge_style(Style::default().fg(color))
        .percent(value)
}

pub fn create_paragraph(title: &str, value: String) -> Paragraph {
    Paragraph::new(value)
        .block(Block::default().title(title).borders(Borders::ALL))
}

pub fn create_status_text(title: &str, status: bool) -> Paragraph {
    let (text, color) = if status {
        ("Online", Color::Green)
    } else {
        ("Offline", Color::Red)
    };
    
    Paragraph::new(Span::styled(text, Style::default().fg(color)))
        .block(Block::default().title(title).borders(Borders::ALL))
}