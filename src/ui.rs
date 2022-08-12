use crate::database::Database;
use crate::ui::Preferences::{Expire, Repeat};
use anyhow::Result;
use chrono::serde::ts_seconds;
use chrono::{DateTime, Datelike, Utc};
use rayon::prelude::*;
use serde::{Deserialize, Serialize};
use tui::backend::Backend;
use tui::layout::{Constraint, Layout};
use tui::style::{Color, Modifier, Style};
use tui::text::{Span, Spans};
use tui::widgets::{Block, BorderType, Borders, List, ListItem, ListState, Paragraph};
use tui::{Frame, Terminal};
use unicode_width::UnicodeWidthStr;

#[derive(Default)]
pub(crate) struct StatefulList<T> {
    pub(crate) state: ListState,
    pub(crate) items: Vec<T>,
}

impl<T> StatefulList<T> {
    pub(crate) fn next(&mut self) {
        let i = match self.state.selected() {
            Some(i) => {
                if i >= self.items.len().saturating_sub(1) {
                    0
                } else {
                    i + 1
                }
            }
            None => 0,
        };
        self.state.select(Some(i));
    }

    pub(crate) fn previous(&mut self) {
        let i = match self.state.selected() {
            Some(i) => {
                if i == 0 {
                    self.items.len().saturating_sub(1)
                } else {
                    i.saturating_sub(1)
                }
            }
            None => 0,
        };
        self.state.select(Some(i));
    }
}

#[derive(Default)]
pub struct App {
    pub(crate) database: Database,
    pub(crate) mode: WindowMode,
    pub(crate) tasks: StatefulList<(String, Task)>,
    pub(crate) preferences: StatefulList<(String, Preferences)>,
    pub(crate) preferences_input: String,
    pub(crate) task: Task,
    pub(crate) new_task: bool,
    pub(crate) cursor_pos_x: u16,
    pub(crate) cursor_pos_y: u16,
    pub(crate) width: u16,
}

#[derive(Default)]
pub enum WindowMode {
    #[default]
    List,
    Task(EditMode),
    Preferences(bool),
}

pub enum EditMode {
    View,
    Edit(EditState),
}

pub enum EditState {
    Title,
    Task,
}

#[derive(Clone, Serialize, Deserialize, Default)]
pub struct Task {
    pub(crate) title: String,
    pub(crate) description: String,
    pub(crate) done: bool,
    pub(crate) daily_repeat: bool,
    pub(crate) expire: Date,
    // TODO
    // another parameters
}

#[derive(Clone, Serialize, Deserialize)]
pub struct Date {
    #[serde(with = "ts_seconds")]
    date: DateTime<Utc>,
}

impl Default for Date {
    fn default() -> Self {
        let now = Utc::now();

        let date = match now.with_day(now.day() + 1) {
            Some(date) => date,
            None => match now.with_month(now.month() + 1) {
                Some(date) => date.with_day(1).unwrap(),
                None => now
                    .with_year(now.year() + 1)
                    .unwrap()
                    .with_month(1)
                    .unwrap()
                    .with_day(1)
                    .unwrap(),
            },
        };

        Self { date }
    }
}

#[derive(Default)]
pub enum Preferences {
    #[default]
    Default,
    Repeat(bool),
    Expire(String),
}

impl Preferences {
    fn value(&self) -> String {
        match self {
            Repeat(b) => b.to_string(),
            Expire(e) => e.to_string(),
            _ => "".to_string(),
        }
    }
}

impl App {
    pub fn run<B: Backend>(&mut self, terminal: &mut Terminal<B>) -> Result<()> {
        self.update_tasks()?;

        loop {
            terminal.draw(|f| self.ui(f))?;

            self.event()?
        }
    }

    pub(crate) fn update_tasks(&mut self) -> Result<()> {
        let mut tasks = self
            .database
            .database
            .iter()
            .par_bridge()
            .filter_map(|d| {
                let (id, task) = d.ok()?;

                let (task, _) =
                    bincode::serde::decode_from_slice::<Task, _>(&task, self.database.config)
                        .ok()?;

                if task.expire.date <= Utc::now() {
                    return None;
                }

                let id = String::from_utf8(id.to_vec()).ok()?;

                Some((id, task))
            })
            .collect::<Vec<(String, Task)>>();

        tasks.par_sort_unstable_by(|(_, time1), (_, time2)| {
            time1.expire.date.cmp(&time2.expire.date)
        });

        self.tasks.items = tasks;

        Ok(())
    }

    fn ui<B: Backend>(&mut self, f: &mut Frame<B>) {
        match self.mode {
            WindowMode::List => self.tasks_window(f),
            WindowMode::Task(_) => self.view_window(f),
            WindowMode::Preferences(_) => self.preferences_window(f),
        }
    }

    fn tasks_window<B: Backend>(&mut self, f: &mut Frame<B>) {
        let layout = Layout::default()
            .margin(2)
            .constraints([Constraint::Percentage(100)])
            .split(f.size());

        let tasks: Vec<ListItem> = self
            .tasks
            .items
            .par_iter()
            .map(|(_, t)| {
                let (status, style) = if t.done {
                    ("✅ ".to_string(), Style::default().fg(Color::Green))
                } else {
                    ("❌ ".to_string(), Style::default())
                };
                let content = vec![Spans::from(Span::styled(status + &t.title, style))];
                ListItem::new(content)
            })
            .collect();

        let tasks = List::new(tasks)
            .block(Block::default().borders(Borders::ALL).title(" Tasks "))
            .highlight_style(
                Style::default()
                    .bg(Color::LightBlue)
                    .add_modifier(Modifier::BOLD),
            );

        f.render_stateful_widget(tasks, layout[0], &mut self.tasks.state)
    }

    fn view_window<B: Backend>(&mut self, f: &mut Frame<B>) {
        self.cursor_pos_x = self.task.description.split('\n').last().unwrap().width() as u16;

        let layout = Layout::default()
            .margin(2)
            .constraints([Constraint::Percentage(7), Constraint::Percentage(93)])
            .split(f.size());

        self.width = layout[1].width;

        let title_block = Paragraph::new(self.task.title.as_ref())
            .style(match self.mode {
                WindowMode::Task(EditMode::Edit(EditState::Title)) => {
                    Style::default().fg(Color::Cyan)
                }
                _ => Style::default(),
            })
            .block(
                Block::default()
                    .title(" Title ")
                    .borders(Borders::ALL)
                    .border_type(BorderType::Rounded),
            );

        match self.mode {
            WindowMode::Task(EditMode::Edit(EditState::Title)) => f.set_cursor(
                layout[0].x + self.task.title.width() as u16 + 1,
                layout[0].y + 1,
            ),
            WindowMode::Task(EditMode::Edit(EditState::Task)) => f.set_cursor(
                layout[1].x + self.cursor_pos_x + 1,
                layout[1].y + self.cursor_pos_y + 1,
            ),
            _ => {}
        }

        let task_block = Paragraph::new(self.task.description.as_ref())
            .style(match self.mode {
                WindowMode::Task(EditMode::Edit(EditState::Task)) => {
                    Style::default().fg(Color::Green)
                }
                _ => Style::default(),
            })
            .block(
                Block::default()
                    .title(" Description ")
                    .borders(Borders::ALL)
                    .border_type(BorderType::Rounded),
            );

        f.render_widget(title_block, layout[0]);
        f.render_widget(task_block, layout[1]);
    }

    fn preferences_window<B: Backend>(&mut self, f: &mut Frame<B>) {
        let layout = Layout::default()
            .margin(2)
            .constraints([Constraint::Percentage(90), Constraint::Percentage(10)])
            .split(f.size());

        self.width = layout[0].width;

        self.preferences.items = vec![
            ("Repeat".to_string(), Repeat(self.task.daily_repeat)),
            (
                "Expire".to_string(),
                Expire(self.task.expire.date.format("%Y/%m/%d %H:%M").to_string()),
            ),
        ];

        let options: Vec<ListItem> = self
            .preferences
            .items
            .par_iter()
            .map(|(title, pref)| {
                let raw = format!("{title}: {}", pref.value());
                ListItem::new(vec![Spans::from(Span::raw(raw))])
            })
            .collect();

        let options = List::new(options).highlight_style(
            Style::default()
                .bg(Color::LightBlue)
                .add_modifier(Modifier::BOLD),
        );

        let input = Paragraph::new(self.preferences_input.as_ref())
            .block(
                Block::default()
                    .title(" Edit ")
                    .borders(Borders::ALL)
                    .border_type(BorderType::Rounded),
            );

        match self.mode {
            WindowMode::Preferences(true) => {
                f.set_cursor(
                    layout[1].x + self.preferences_input.width() as u16 + 1,
                    layout[1].y + 1,
                )
            }
            _ => {}
        }

        f.render_stateful_widget(options, layout[0], &mut self.preferences.state);
        f.render_widget(input, layout[1]);
    }
}
