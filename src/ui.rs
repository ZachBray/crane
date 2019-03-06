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
use std::time::Instant;
use std::cmp::max;

#[derive(Clone, Copy)]
pub enum Status {
    Succeeded,
    Pending,
    Failed,
}

impl Status {
    fn text(&self) -> &'static str {
        match self {
            Status::Succeeded => "Succeeded",
            Status::Pending => "Pending",
            Status::Failed => "Failed",
        }
    }

    fn style(&self) -> Style {
        match self {
            Status::Succeeded => Style::default()
                .fg(Color::Green),
            Status::Pending => Style::default()
                .fg(Color::Yellow)
                .modifier(Modifier::Blink),
            Status::Failed => Style::default()
                .bg(Color::Red)
                .fg(Color::White)
                .modifier(Modifier::Blink),
        }
    }

    fn render<B>(&self, frame: &mut Frame<B>, area: Rect) where B: Backend {
        let lines = [Text::raw(self.text())];
        let style = self.style();

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

struct BuildResult {
    sha: String,
    status: Status,
}

struct BuildTable {
    builds: Vec<BuildResult>,
}

impl BuildTable {
    fn new() -> Self {
        BuildTable {
            builds: vec![]
        }
    }

    fn render<B>(&self, frame: &mut Frame<B>, area: Rect) where B: Backend {
        let rows = self.builds.iter()
            .map(|result| Row::StyledData(
                vec![result.sha.to_string(), result.status.text().to_string()].into_iter(), result.status.style()));

        let block = Block::default()
            .borders(Borders::ALL)
            .title("Builds");

        Table::new(["Commit", "Status"].iter(), rows)
            .widths(&[12, 18])
            .header_style(Style::default().fg(Color::DarkGray))
            .block(block)
            .render(frame, area)
    }
}

struct RetryWindow {
    start_time: Instant,
    due_time: Instant,
}

impl RetryWindow {
    fn new() -> Self {
        RetryWindow {
            start_time: Instant::now(),
            due_time: Instant::now(),
        }
    }

    fn render<B>(&self, frame: &mut Frame<B>, area: Rect) where B: Backend {
        let block = Block::default()
            .borders(Borders::ALL)
            .title("Retry period");

        let now = Instant::now();
        let window = self.due_time - self.start_time;
        let window_millis = window.as_secs() * 1000 + (window.subsec_millis() as u64);
        let remaining = self.due_time - now;
        let remaining_millis = remaining.as_secs() * 1000 + (remaining.subsec_millis() as u64);
        let ratio = remaining_millis as f64 / (max(window_millis, 1) as f64);

        Gauge::default()
            .block(block)
            .ratio(ratio)
            .style(Style::default().fg(Color::DarkGray))
            .label(&format!("{}s remaining", remaining.as_secs()))
            .render(frame, area)
    }
}

struct LastError {
    error: Error,
}

impl LastError {
    fn render<B>(&self, frame: &mut Frame<B>, area: Rect) where B: Backend {
        let lines = [Text::raw(format!("{}", &self.error))];

        let block = Block::default()
            .borders(Borders::ALL)
            .title("Last error");

        Paragraph::new(lines.iter())
            .style(Style::default().fg(Color::Red))
            .block(block)
            .wrap(true)
            .render(frame, area)
    }
}

pub struct Summary {
    terminal: Terminal<TermionBackend<AlternateScreen<MouseTerminal<RawTerminal<Stdout>>>>>,
    status: Status,
    property_table: PropertyTable,
    retry_window: RetryWindow,
    build_table: BuildTable,
    last_error: Option<LastError>,
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
            retry_window: RetryWindow::new(),
            build_table: BuildTable::new(),
            last_error: None
        };
        Ok(summary)
    }

    pub fn render(&mut self) -> Result<(), Error> {
        let status = &self.status;
        let property_table = &self.property_table;
        let retry_window = &self.retry_window;
        let build_table = &self.build_table;
        let last_error = &self.last_error;

        self.terminal.draw(|mut frame| {
            let outer_horizontal_pane = Layout::default()
                .direction(Direction::Horizontal)
                .constraints(vec![Constraint::Min(40), Constraint::Percentage(65)])
                .split(frame.size());

            let left_vertical_pane_constraints = if last_error.is_none() {
                vec![Constraint::Length(5), Constraint::Min(10)]
            } else {
                vec![Constraint::Length(5), Constraint::Min(10), Constraint::Length(5)]
            };

            let left_vertical_pane = Layout::default()
                .direction(Direction::Vertical)
                .constraints(left_vertical_pane_constraints)
                .split(outer_horizontal_pane[0]);

            let right_vertical_pane = Layout::default()
                .direction(Direction::Vertical)
                .constraints(vec![Constraint::Length(5), Constraint::Min(5)])
                .split(outer_horizontal_pane[1]);

            status.render(&mut frame, left_vertical_pane[0]);
            property_table.render(&mut frame, left_vertical_pane[1]);
            retry_window.render(&mut frame, right_vertical_pane[0]);
            build_table.render(&mut frame, right_vertical_pane[1]);

            if let Some(error) = last_error {
                error.render(&mut frame, left_vertical_pane[2]);
            }
        })?;
        Ok(())
    }

    pub fn reset_retry_window(&mut self, due_time: Instant) {
        self.retry_window.start_time = Instant::now();
        self.retry_window.due_time = due_time;
    }

    pub fn record_build(&mut self, sha: &str, status: Status) {
        self.status = status;

        let mut has_seen_build = false;
        for build in &mut self.build_table.builds {
            if build.sha == sha {
                build.status = status;
                has_seen_build = true;
            }
        }

        if has_seen_build {
            return;
        }

        let builds = &mut self.build_table.builds;

        if builds.len() >= 10 {
            builds.remove(0);
        }

        builds.push(BuildResult {
            sha: sha.to_string(),
            status,
        });
    }

    pub fn record_error(&mut self, error: Error) {
        self.last_error = Some(LastError { error });
    }
}
