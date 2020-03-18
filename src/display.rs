use std::collections::HashMap;
use std::io::{stdout, Error};
use termion::{event::Key, input::MouseTerminal, raw::IntoRawMode, screen::AlternateScreen};
use tokio::sync::mpsc::error::TryRecvError;
use tui::backend::TermionBackend;
use tui::layout::{Constraint, Direction, Layout};
use tui::style::{Color, Style};
use tui::widgets::{Block, Borders, Gauge};
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
                        .constraints(
                            [
                                Constraint::Length(3),
                                Constraint::Length(7),
                                Constraint::Percentage(40),
                            ]
                            .as_ref(),
                        )
                        .split(f.size());

                    let mut gauge = Gauge::default()
                        .block(Block::default().title("Progress").borders(Borders::ALL))
                        .style(Style::default().fg(Color::Black).bg(Color::Green))
                        .ratio(all.len() as f64 / 5 as f64);
                    f.render(&mut gauge, chunks[0]);
                })
                .unwrap();
        }

        Ok(all)
    }
}
