use std::collections::HashMap;
use std::io::{stdout, Error};
use termion::{event::Key, input::MouseTerminal, raw::IntoRawMode, screen::AlternateScreen};
use tokio::sync::mpsc::error::TryRecvError;
use tui::backend::TermionBackend;
use tui::layout::{Constraint, Direction, Layout};
use tui::style::{Color, Modifier, Style};
use tui::widgets::{Block, Borders, Gauge, List, Text};
use tui::Terminal;

use crate::RequestResult;

pub struct Monitor {
    pub report_receiver: tokio::sync::mpsc::Receiver<anyhow::Result<RequestResult>>,
    pub start: std::time::Instant,
    pub fps: usize,
}

impl Monitor {
    pub async fn monitor(mut self) -> Result<Vec<anyhow::Result<RequestResult>>, Error> {
        let stdout = stdout().into_raw_mode()?;
        let stdout = MouseTerminal::from(stdout);
        let stdout = AlternateScreen::from(stdout);
        let backend = TermionBackend::new(stdout);
        let mut terminal = Terminal::new(backend)?;
        terminal.hide_cursor()?;

        let mut all: Vec<anyhow::Result<RequestResult>> = Vec::new();
        let mut status_dist: HashMap<hyper::StatusCode, usize> = HashMap::new();
        'outer: loop {
            loop {
                match self.report_receiver.try_recv() {
                    Ok(report) => {
                        if let Ok(report) = report.as_ref() {
                            *status_dist.entry(report.status).or_default() += 1;
                        }
                        all.push(report);
                    }
                    Err(TryRecvError::Empty) => {
                        break;
                    }
                    Err(TryRecvError::Closed) => {
                        break 'outer;
                    }
                }
            }

            terminal
                .draw(|mut f| {
                    let chunks = Layout::default()
                        .direction(Direction::Vertical)
                        .margin(2)
                        .constraints([Constraint::Percentage(100)].as_ref())
                        .split(f.size());

                    let tasks = status_dist
                        .iter()
                        .map(|(status, _count)| Text::raw(format!("{}", status)));
                    let mut task_list = List::new(tasks)
                        .block(Block::default().borders(Borders::ALL).title("List"));
                    f.render(&mut task_list, chunks[0]);
                })
                .unwrap();

            // maybe just keep looping until Event::Input matches Key::Char('q')
        }

        Ok(all)
    }
}
