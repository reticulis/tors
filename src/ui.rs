use crate::database::Database;
use anyhow::Result;
use chrono::Local;
use rayon::prelude::*;
use std::cell::RefCell;
use std::env;
use std::rc::Rc;
use tui::backend::Backend;
use tui::layout::{Alignment, Constraint, Layout};
use tui::style::{Color, Modifier, Style};
use tui::text::{Span, Spans};
use tui::widgets::{Block, BorderType, Borders, List, ListItem, ListState, Paragraph};
use tui::{Frame, Terminal};
use unicode_width::UnicodeWidthStr;
use crate::task::Task;
use num_integer::Roots;

#[derive(Default, PartialEq, Eq)]
pub enum WindowMode {
    #[default]
    List,
    Task(EditMode),
    Preferences(bool),
    Stats,
}

#[derive(PartialEq, Eq)]
pub enum EditMode {
    View,
    Edit(EditState),
}

#[derive(PartialEq, Eq)]
pub enum EditState {
    Title,
    Task,
}

pub struct App {
    pub(crate) database: Database,
    pub(crate) mode: WindowMode,
    pub(crate) tasks: StatefulList<(String, Rc<RefCell<Task>>)>,
    pub(crate) preferences: StatefulList<String>,
    pub(crate) preferences_input: String,
    pub(crate) cursor_pos_x: u16,
    pub(crate) cursor_pos_y: u16,
    pub(crate) width: u16,
}

impl App {
    pub fn new() -> Result<Self> {
        let database = Database::new()?;

        Ok(Self {
            database,
            mode: WindowMode::default(),
            tasks: StatefulList::default(),
            preferences: StatefulList::default(),
            preferences_input: String::new(),
            cursor_pos_x: 0,
            cursor_pos_y: 0,
            width: 0,
        })
    }

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
            .iter()
            .filter_map(|v| {
                let (id, task) = v.ok()?;

                let id = String::from_utf8_lossy(&id).to_string();

                let (task, _) =
                    bincode::serde::decode_from_slice::<Task, _>(&task, self.database.config)
                        .ok()?;

                if task.preferences.expire <= Local::now().naive_local() {
                    return None;
                }

                Some((id, Rc::new(RefCell::new(task))))
            })
            .collect::<Vec<(String, Rc<RefCell<Task>>)>>();

        tasks.sort_unstable_by(|(_, time1), (_, time2)| {
            let time1 = time1.borrow();
            let time2 = time2.borrow();

            time1.creation_date.cmp(&time2.creation_date)
        });

        self.tasks.items = tasks;

        Ok(())
    }

    pub(crate) fn task(&self) -> Option<(&String, Rc<RefCell<Task>>)> {
        let (id, task) = self.tasks.items.get(self.tasks.state.selected()?)?;

        Some((id, task.clone()))
    }

    fn ui<B: Backend>(&mut self, f: &mut Frame<B>) {
        match self.mode {
            WindowMode::List => self.tasks_window(f),
            WindowMode::Task(_) => self.view_window(f),
            WindowMode::Preferences(_) => self.preferences_window(f),
            WindowMode::Stats => self.statistics_window(f),
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
                let t = t.clone();
                let t = &mut *t.borrow_mut();
                let (status, style) = if t.done {
                    ("✅ ".to_string(), Style::default().fg(Color::Green))
                } else {
                    ("❌ ".to_string(), Style::default())
                };
                let content = vec![Spans::from(Span::styled(status + &t.title, style))];
                ListItem::new(content)
            })
            .collect();

        if !tasks.is_empty() && self.tasks.state.selected().is_none() {
            self.tasks.state.select(Some(0));
        }

        let tasks = List::new(tasks)
            .block(Block::default().borders(Borders::ALL).title(" Tasks "))
            .highlight_style(
                Style::default()
                    .bg(Color::DarkGray)
                    .add_modifier(Modifier::BOLD),
            );

        f.render_stateful_widget(tasks, layout[0], &mut self.tasks.state)
    }

    fn view_window<B: Backend>(&mut self, f: &mut Frame<B>) {
        let (_, task) = self.task().unwrap();
        let task = &mut *task.borrow_mut();

        self.cursor_pos_x = task.description.split('\n').last().unwrap().width() as u16;

        let layout = Layout::default()
            .margin(2)
            .constraints([Constraint::Percentage(7), Constraint::Percentage(93)])
            .split(f.size());

        self.width = layout[1].width;

        let title_block = Paragraph::new(task.title.as_ref())
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

        let task_block = Paragraph::new(task.description.as_ref())
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

        match self.mode {
            WindowMode::Task(EditMode::Edit(EditState::Title)) => {
                f.set_cursor(layout[0].x + task.title.width() as u16 + 1, layout[0].y + 1)
            }
            WindowMode::Task(EditMode::Edit(EditState::Task)) => f.set_cursor(
                layout[1].x + self.cursor_pos_x + 1,
                layout[1].y + self.cursor_pos_y + 1,
            ),
            _ => {}
        }

        f.render_widget(title_block, layout[0]);
        f.render_widget(task_block, layout[1]);
    }

    fn preferences_window<B: Backend>(&mut self, f: &mut Frame<B>) {
        let layout = Layout::default()
            .margin(2)
            .constraints([Constraint::Percentage(90), Constraint::Percentage(10)])
            .split(f.size());

        self.width = layout[0].width;

        let (_, task) = self.task().unwrap();
        let task = &mut *task.borrow_mut();

        self.preferences.items = vec![
            format!("Repeat: {}", task.preferences.daily_repeat),
            format!(
                "Expire: {}",
                task.preferences.expire.format("%Y-%m-%d %H:%M:%S")
            ),
            format!("Experience: {}", task.preferences.exp),
        ];

        let options: Vec<ListItem> = self
            .preferences
            .items
            .par_iter()
            .map(|f| ListItem::new(vec![Spans::from(Span::raw(f))]))
            .collect();

        let options = List::new(options).highlight_style(
            Style::default()
                .bg(Color::DarkGray)
                .add_modifier(Modifier::BOLD),
        );

        let input = Paragraph::new(self.preferences_input.as_ref()).block(
            Block::default()
                .title(" Edit ")
                .borders(Borders::ALL)
                .border_type(BorderType::Rounded),
        );

        if let WindowMode::Preferences(true) = self.mode {
            f.set_cursor(
                layout[1].x + self.preferences_input.width() as u16 + 1,
                layout[1].y + 1,
            )
        }

        f.render_stateful_widget(options, layout[0], &mut self.preferences.state);
        f.render_widget(input, layout[1]);
    }

    fn statistics_window<B: Backend>(&mut self, f: &mut Frame<B>) {
        let layout = Layout::default()
            .margin(2)
            .constraints([Constraint::Percentage(100)])
            .split(f.size());

        let username = env::var("USER").unwrap_or_default();

        let exp = self.database.get_exp().unwrap();
        let lvl = (exp/10).sqrt().saturating_sub(1);
        let next_lvl_exp = 10*(lvl+2).pow(2);

        let stats = format!(
            "Level: {}\n\
            Exp: {}\n\
            Exp to next level: {}",
            lvl,
            exp,
            next_lvl_exp - exp,
        );

        let stats = Paragraph::new(stats).block(
            Block::default()
                .title(format!(" {}'s stats", &username))
                .title_alignment(Alignment::Center)
                .borders(Borders::ALL)
                .border_type(BorderType::Rounded),
        );

        f.render_widget(stats, layout[0])
    }
}

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
