use std::{io, vec};

use crossterm::event::{self, Event, KeyCode, KeyEventKind, KeyModifiers};
use ports::netstat::{NetStat, NetStatEntry, Protocol, truncate_path};
use ratatui::{
    DefaultTerminal, Frame,
    layout::{Constraint, Flex, Layout, Margin, Rect},
    style::{Style, Stylize},
    symbols::border,
    text::Line,
    widgets::{Block, Cell, Clear, Row, Table},
};
use sysinfo::{Pid, System};

pub struct Tui {
    netstat: Box<dyn NetStat>,
    proto: Protocol,
    ports: Vec<NetStatEntry>,
    selected: usize,
    confirm: bool,
    exit: bool,
}

impl Tui {
    pub fn new(netstat: Box<dyn NetStat>, proto: Protocol) -> Self {
        Self {
            netstat,
            proto,
            ports: Vec::new(),
            exit: false,
            selected: 0,
            confirm: false,
        }
    }

    pub fn run(&mut self, terminal: &mut DefaultTerminal) -> io::Result<()> {
        self.refresh_ports()?;
        while !self.exit {
            terminal.draw(|frame| self.draw(frame))?;
            self.handle_events()?;
        }
        Ok(())
    }

    fn draw(&self, frame: &mut Frame) {
        let title = Line::from(" Ports ".bold());
        let instructions = Line::from(vec![
            " Refresh ".into(),
            "<R>".blue().bold(),
            " Kill".into(),
            " <Enter>".red().bold(),
            " Quit ".into(),
            "<Q> ".blue().bold(),
        ]);
        let block = Block::bordered()
            .title(title.centered())
            .title_bottom(
                if self.confirm
                    && let Some(port) = self.ports.get(self.selected)
                {
                    Line::from(vec![
                        " Are you sure you want to ".into(),
                        "kill".red().bold(),
                        " PID ".into(),
                        port.pid.to_string().into(),
                        "? [".into(),
                        "Y".bold(),
                        "/n] ".into(),
                    ])
                } else {
                    instructions
                }
                .centered(),
            )
            .border_set(border::ROUNDED);

        let table_width: u16 = self
            .get_table_constraints()
            .iter()
            .map(|c| match c {
                Constraint::Length(l) => *l,
                Constraint::Max(l) => *l,
                Constraint::Min(l) => *l,
                _ => 0,
            })
            .sum();

        let [area] = Layout::horizontal([Constraint::Length(table_width + 4)])
            .flex(Flex::Center)
            .areas(frame.area());
        let [area] = Layout::vertical([Constraint::Length(self.ports.len() as u16 + 5)])
            .flex(Flex::Center)
            .areas(area);

        let inner = block.inner(area);

        frame.render_widget(Clear, frame.area());
        frame.render_widget(block, area);
        self.render_table(frame, inner);
    }

    fn get_table_constraints(&self) -> [Constraint; 5] {
        [
            Constraint::Length(38),
            Constraint::Min(7),
            Constraint::Min(19),
            Constraint::Min(19),
            Constraint::Length(8),
        ]
    }

    fn render_table(&self, frame: &mut Frame, area: Rect) {
        let header = ["Exe", "PID", "Local Address", "Remote Address", "Protocol"]
            .into_iter()
            .map(Cell::from)
            .collect::<Row>()
            .height(1)
            .bold();
        let table_constraints = self.get_table_constraints();

        let table = Table::new(
            self.ports.iter().enumerate().map(|(i, port)| {
                Row::new(vec![
                    format!("...{}", truncate_path(&port.exe, 32)),
                    // port.exe.to_owned(),
                    port.pid.to_string(),
                    port.local_addr.to_string(),
                    port.remote_addr.to_string(),
                    port.proto.to_string(),
                ])
                .style(if i == self.selected {
                    Style::default().bold().blue()
                } else {
                    Style::default().dim()
                })
            }),
            table_constraints,
        )
        .header(header);

        frame.render_widget(table, area.inner(Margin::new(1, 1)));
    }

    fn handle_events(&mut self) -> io::Result<()> {
        match event::read()? {
            Event::Key(key_event) if key_event.kind == KeyEventKind::Press => {
                self.handle_key_event(key_event)?
            }
            _ => {}
        };
        Ok(())
    }

    fn handle_key_event(&mut self, key_event: event::KeyEvent) -> io::Result<()> {
        match key_event.code {
            KeyCode::Char('q') => self.exit(),
            KeyCode::Char('c') => {
                if key_event.modifiers == KeyModifiers::CONTROL {
                    self.exit()
                }
            }

            KeyCode::Char('r') => {
                if !self.confirm {
                    self.refresh_ports()?;
                }
            }

            // Navigation keybinds
            KeyCode::Char('j') => {
                if !self.confirm {
                    self.select_next();
                }
            }
            KeyCode::Char('k') => {
                if !self.confirm {
                    self.select_prev();
                }
            }
            KeyCode::Down => {
                if !self.confirm {
                    self.select_next();
                }
            }
            KeyCode::Up => {
                if !self.confirm {
                    self.select_prev();
                }
            }

            KeyCode::Enter => {
                if self.confirm {
                    self.kill_selected()?;
                } else {
                    self.confirm = true;
                }
            }
            KeyCode::Char('y') => {
                if self.confirm {
                    self.kill_selected()?;
                }
            }
            KeyCode::Char('n') => {
                self.confirm = false;
            }
            _ => {}
        }

        Ok(())
    }

    fn kill_selected(&mut self) -> io::Result<()> {
        self.confirm = false;

        if self.ports.is_empty() {
            return Ok(());
        }

        let port = &self.ports[self.selected];

        let s = System::new_all();
        if let Some(process) = s.process(Pid::from(port.pid)) {
            process.kill_and_wait().map_err(|ref e| {
                io::Error::other(match e {
                    sysinfo::KillError::SignalDoesNotExist => "Kill signal does not exist.",
                    sysinfo::KillError::FailedToSendSignal => "Failed to send kill signal.",
                })
            })?;
        }
        self.refresh_ports()
    }

    fn select_next(&mut self) {
        if !self.ports.is_empty() {
            self.selected = (self.selected + 1).rem_euclid(self.ports.len())
        }
    }

    fn select_prev(&mut self) {
        if self.ports.is_empty() {
            return;
        }

        if self.selected == 0 {
            self.selected = self.ports.len() - 1
        } else {
            self.selected -= 1
        }
    }

    fn exit(&mut self) {
        self.exit = true;
    }

    fn refresh_ports(&mut self) -> io::Result<()> {
        self.ports = self.netstat.get_ports(&self.proto)?;
        self.selected = self.selected.min(self.ports.len().saturating_sub(1));
        Ok(())
    }
}
