use std::io;
use std::io::Stdout;
use failure::Error;
use termion::screen::AlternateScreen;
use termion::raw::IntoRawMode;
use termion::raw::RawTerminal;
use tui::widgets::Row;
use tui::Terminal;
use tui::backend::Backend;
use tui::backend::TermionBackend;
use tui::Frame;
use tui::widgets::*;
use tui::style::*;
use tui::layout::*;
use termion::input::MouseTerminal;


pub enum Status {
    Succeeded,
    Pending,
    Failed,
}

impl Status {
    fn render<B>(&self, frame: &mut Frame<B>, area: Rect) where B: Backend {
        let text = match self {
            Status::Succeeded => "Succeeded",
            Status::Pending => "Pending",
            Status::Failed => "Failed",
        };

        let lines = [Text::raw(text)];

        let style = match self {
            Status::Succeeded => Style::default()
                .fg(Color::Green),
            Status::Pending => Style::default()
                .fg(Color::Yellow)
                .modifier(Modifier::Blink),
            Status::Failed => Style::default()
                .bg(Color::Red)
                .fg(Color::White)
                .modifier(Modifier::Blink),
        };

        let block = Block::default()
            .borders(Borders::ALL)
            .title("Status");

        Paragraph::new(lines.iter())
            .style(style)
            .block(block)
            .render(frame, area)
    }
}

pub struct Property {
    name: String,
    value: String,
}

impl Property {
    pub fn new(name: &str, value: &str) -> Self {
        Property {
            name: name.to_string(),
            value: value.to_string(),
        }
    }
}

struct PropertyTable {
    properties: Vec<Property>
}

impl PropertyTable {
    fn render<B>(&self, frame: &mut Frame<B>, area: Rect) where B: Backend {
        let rows = self.properties.iter()
            .map(move |p| Row::Data(vec![&p.name, &p.value].into_iter()));

        let block = Block::default()
            .borders(Borders::ALL)
            .title("Agent");

        Table::new(["Properties", ""].iter(), rows)
            .widths(&[12, 18])
            .header_style(Style::default().fg(Color::DarkGray))
            .block(block)
            .render(frame, area)
    }
}

pub struct Summary {
    terminal: Terminal<TermionBackend<AlternateScreen<MouseTerminal<RawTerminal<Stdout>>>>>,
    status: Status,
    property_table: PropertyTable,
}

impl Summary {
    pub fn new(properties: Vec<Property>) -> Result<Self, Error> {
        let stdout = io::stdout().into_raw_mode()?;
        let stdout = MouseTerminal::from(stdout);
        let stdout = AlternateScreen::from(stdout);
        let backend = TermionBackend::new(stdout);
        let terminal = Terminal::new(backend)?;
        let summary = Summary {
            status: Status::Pending,
            terminal,
            property_table: PropertyTable { properties },
        };
        Ok(summary)
    }

    pub fn render(&mut self) -> Result<(), Error> {
        let status = &self.status;
        let property_table = &self.property_table;
        self.terminal.draw(|mut frame| {
            let main = Layout::default()
                .direction(Direction::Vertical)
                .constraints(vec![Constraint::Min(3), Constraint::Min(10)])
                .split(frame.size());

            status.render(&mut frame, main[0]);
            property_table.render(&mut frame, main[1])
        })?;
        Ok(())
    }
}
