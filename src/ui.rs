use crossterm::event;
use crossterm::event::{Event, KeyCode};
use std::io;
use tui::backend::Backend;
use tui::layout::{Constraint, Layout};
use tui::style::{Color, Modifier, Style};
use tui::text::{Span, Spans};
use tui::widgets::{Block, BorderType, Borders, List, ListItem, ListState, Paragraph};
use tui::{Frame, Terminal};
use unicode_width::UnicodeWidthStr;

#[derive(Default)]
struct StatefulList {
    state: ListState,
    items: Vec<String>,
}

impl StatefulList {
    fn next(&mut self) {
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

    fn previous(&mut self) {
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
    mode: WindowMode,
    tasks: StatefulList,
    title_input: String,
    task_input: String,
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

impl App {
    pub fn run<B: Backend>(&mut self, terminal: &mut Terminal<B>) -> io::Result<()> {
        loop {
            terminal.draw(|f| self.ui(f))?;

            if let Event::Key(key) = event::read()? {
                match self.mode {
                    WindowMode::List => {
                        match key.code {
                            KeyCode::Char(' ') => {} // Mark task as done
                            KeyCode::Char('n') => {
                                self.mode = WindowMode::Task(EditMode::Edit(EditState::Title))
                            } // New Task
                            KeyCode::Down => self.tasks.next(),
                            KeyCode::Up => self.tasks.previous(),
                            KeyCode::Enter => self.mode = WindowMode::Task(EditMode::View),
                            KeyCode::Esc => break,
                            _ => {}
                        }
                    }
                    WindowMode::Task(EditMode::View) => match key.code {
                        KeyCode::Esc => {
                            self.task_input.clear();
                            self.title_input.clear();
                            self.mode = WindowMode::List;
                        }
                        KeyCode::Char('t') => {
                            // Edit title
                            self.mode = WindowMode::Task(EditMode::Edit(EditState::Title))
                        }
                        KeyCode::Char('e') => {
                            // Edit task content
                            self.mode = WindowMode::Task(EditMode::Edit(EditState::Task))
                        }
                        KeyCode::Char('s') => {
                            // Save task
                            todo!()
                        }
                        _ => {}
                    },
                    // Todo
                    WindowMode::Task(EditMode::Edit(EditState::Title)) => {
                        match key.code {
                            KeyCode::Esc => self.mode = WindowMode::Task(EditMode::View),
                            KeyCode::Char(n) => self.title_input.push(n), // Input title text
                            KeyCode::Backspace => {
                                self.title_input.pop();
                            },
                            _ => {}
                        }
                    }
                    WindowMode::Task(EditMode::Edit(EditState::Task)) => {
                        match key.code {
                            KeyCode::Esc => self.mode = WindowMode::Task(EditMode::View),
                            KeyCode::Char(n) => self.task_input.push(n), // Input task text
                            KeyCode::Backspace => {
                                self.task_input.pop();
                            }
                            _ => {}
                        }
                    }
                }
            }
        }

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
            .map(|c| {
                let content = vec![Spans::from(Span::raw(c))];
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
        let layout = Layout::default()
            .margin(2)
            .constraints([Constraint::Percentage(10), Constraint::Percentage(90)])
            .split(f.size());

        let title_block = Paragraph::new(self.title_input.as_ref())
            .style(match self.mode {
                WindowMode::Task(EditMode::Edit(EditState::Title)) => {
                    Style::default().fg(Color::Cyan)
                }
                _ => Style::default(),
            })
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .border_type(BorderType::Rounded),
            );

        let task_block = Paragraph::new(self.task_input.as_ref())
            .style(match self.mode {
                WindowMode::Task(EditMode::Edit(EditState::Task)) => {
                    Style::default().fg(Color::Green)
                }
                _ => Style::default(),
            })
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .border_type(BorderType::Rounded),
            );

        match self.mode {
            WindowMode::Task(EditMode::Edit(EditState::Title)) => f.set_cursor(
                layout[0].x + self.title_input.width() as u16 + 1,
                layout[0].y + 1,
            ),
            WindowMode::Task(EditMode::Edit(EditState::Task)) => f.set_cursor(
                layout[1].x + self.task_input.width() as u16 + 1,
                layout[1].y + 1,
            ),
            _ => {}
        }

        f.render_widget(title_block, layout[0]);
        f.render_widget(task_block, layout[1]);
    }
}
