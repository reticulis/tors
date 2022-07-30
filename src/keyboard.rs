use crossterm::event;
use crossterm::event::{Event, KeyCode};
use crate::App;
use crate::ui::{EditMode, EditState, WindowMode};

use anyhow::Result;
use unicode_width::UnicodeWidthStr;

pub enum Status {
    ExitApp,
    Ignore
}

impl App {
    pub fn event(&mut self) -> Result<Status> {
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
                        KeyCode::Enter => {
                            if let Some(i) = self.tasks.state.selected() {

                                let task = self.get_task(i)?;

                                self.title_input = task.title;
                                self.task_input = task.description;

                                self.mode = WindowMode::Task(EditMode::View)
                            }

                        },
                        KeyCode::Esc => return Ok(Status::ExitApp),
                        _ => {}
                    }
                }
                WindowMode::Task(EditMode::View) => match key.code {
                    KeyCode::Esc => {
                        self.task_input.clear();
                        self.title_input.clear();
                        self.cursor_pos_y = 0;
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
                        self.update_database()?;

                        self.update_tasks()?;

                        self.mode = WindowMode::List;
                    }
                    _ => {}
                },
                WindowMode::Task(EditMode::Edit(EditState::Title)) => match key.code {
                    KeyCode::Esc => self.mode = WindowMode::Task(EditMode::View),
                    KeyCode::Char(n) => {
                        self.title_line_width = self.title_input.width() as u8;

                        if self.title_line_width as u16 == self.width.saturating_sub(3) {
                            return Ok(Status::Ignore)
                        }

                        self.title_input.push(n);
                    }
                    KeyCode::Backspace => {
                        self.title_input.pop();
                    }
                    _ => {}
                },
                WindowMode::Task(EditMode::Edit(EditState::Task)) => match key.code {
                    KeyCode::Esc => self.mode = WindowMode::Task(EditMode::View),
                    KeyCode::Char(n) => {
                        self.task_input.push(n);
                        if self.description_line_width as u16 == self.width.saturating_sub(3) {
                            self.task_input.push('\n');
                            self.cursor_pos_y += 1;
                            return Ok(Status::Ignore)
                        }
                    }
                    KeyCode::Enter => {
                        self.task_input.push('\n');
                        self.cursor_pos_y += 1;
                    }
                    KeyCode::Backspace => {
                        self.task_input.pop();

                        if self.cursor_pos_x == 0 {
                            self.cursor_pos_y = self.cursor_pos_y.saturating_sub(1);
                            self.task_input.pop();
                            return Ok(Status::Ignore)
                        }
                    }
                    _ => {}
                },
            }
        }
        Ok(Status::Ignore)
    }
}