use crossterm::event;
use crossterm::event::{Event, KeyCode};
use std::io;
use tui::backend::Backend;
use tui::layout::{Constraint, Layout};
use tui::text::{Span, Spans};
use tui::widgets::{Block, Borders, List, ListItem};
use tui::{Frame, Terminal};

#[derive(Default)]
pub struct App {
    mode: WindowMode,
    tasks: Vec<String>,
    input: String,
}

pub enum WindowMode {
    List,
    NewTask(EditMode),
    EditTask(EditMode),
}

impl Default for WindowMode {
    fn default() -> Self {
        WindowMode::List
    }
}

pub enum EditMode {
    View,
    Edit,
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
                            KeyCode::Char('n') => self.mode = WindowMode::NewTask(EditMode::Edit), // New Task
                            KeyCode::Enter => self.mode = WindowMode::EditTask(EditMode::View),
                            KeyCode::Esc => break,
                            _ => {}
                        }
                    }
                    WindowMode::EditTask(EditMode::View) => match key.code {
                        KeyCode::Esc => self.mode = WindowMode::List,
                        KeyCode::Char('e') => self.mode = WindowMode::EditTask(EditMode::Edit),
                        _ => {}
                    },
                    WindowMode::EditTask(EditMode::Edit) => {
                        match key.code {
                            KeyCode::Esc => self.mode = WindowMode::List,
                            KeyCode::Char(n) => self.input.push(n), // Input text
                            KeyCode::Enter => {
                                let task: String = self.input.drain(..).collect();

                                // Todo
                                // Write into the file

                                self.tasks.push(task);
                            } // Save
                            _ => {}
                        }
                    }
                    WindowMode::NewTask(EditMode::View) => match key.code {
                        KeyCode::Esc => self.mode = WindowMode::List,
                        KeyCode::Char('e') => self.mode = WindowMode::NewTask(EditMode::Edit),
                        _ => {}
                    },
                    WindowMode::NewTask(EditMode::Edit) => {
                        match key.code {
                            KeyCode::Esc => self.mode = WindowMode::List,
                            KeyCode::Char(n) => self.input.push(n), // Input text
                            KeyCode::Enter => {
                                let task: String = self.input.drain(..).collect();

                                // Todo
                                // Write into the file

                                self.tasks.push(task);
                            } // Save
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
            WindowMode::List => self.list_view(f),
            _ => {}
        }
    }

    fn list_view<B: Backend>(&mut self, f: &mut Frame<B>) {
        let layout = Layout::default()
            .margin(2)
            .constraints([Constraint::Percentage(100)])
            .split(f.size());

        let tasks: Vec<ListItem> = self
            .tasks
            .iter()
            .map(|c| {
                let content = vec![Spans::from(Span::raw(c))];
                ListItem::new(content)
            })
            .collect();

        let tasks = List::new(tasks).block(Block::default().borders(Borders::ALL).title(" Tasks "));

        f.render_widget(tasks, layout[0])
    }
}
