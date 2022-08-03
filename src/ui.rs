use anyhow::Result;
use chrono::{DateTime, Local};
use serde::{Deserialize, Serialize};
use tui::backend::Backend;
use tui::layout::{Constraint, Layout};
use tui::style::{Color, Modifier, Style};
use tui::text::{Span, Spans};
use tui::widgets::{Block, BorderType, Borders, List, ListItem, ListState, Paragraph};
use tui::{Frame, Terminal};
use unicode_width::UnicodeWidthStr;
use crate::database::Database;

#[derive(Default)]
pub(crate) struct StatefulList {
    pub(crate) state: ListState,
    pub(crate) items: Vec<(String, Task)>,
}

impl StatefulList {
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
    pub(crate) tasks: StatefulList,
    pub(crate) task: Task,
    pub(crate) new_task: bool,
    pub(crate) cursor_pos_x: u16,
    pub(crate) cursor_pos_y: u16,
    pub(crate) width: u16,
}

pub enum WindowMode {
    List,
    Task(EditMode),
}

impl Default for WindowMode {
    fn default() -> Self {
        WindowMode::List
    }
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
    pub(crate) expire: String,
    // TODO
    // another parameters
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
        self.tasks.items = self
            .database
            .database
            .iter()
            .map(|v| {
                let (id, task) = v?;

                let (task, _) =
                    bincode::serde::decode_from_slice::<Task, _>(&task, self.database.config)?;
                let id = String::from_utf8_lossy(&id).parse::<String>()?;

                Ok((id, task))
            })
            .collect::<Result<Vec<(String, Task)>>>()?;

        Ok(())
    }

    fn ui<B: Backend>(&mut self, f: &mut Frame<B>) {
        match self.mode {
            WindowMode::List => self.tasks_window(f),
            WindowMode::Task(_) => self.view_window(f),
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
            .iter()
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
}
