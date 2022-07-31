use crate::ui::{EditMode, EditState, WindowMode};
use crate::App;
use crossterm::event;
use crossterm::event::{Event, KeyCode};

use anyhow::Result;
use unicode_width::UnicodeWidthStr;

pub enum Status {
    ExitApp,
    Ignore,
}

impl App {
    pub fn event(&mut self) -> Result<Status> {
        if let Event::Key(key) = event::read()? {
            match self.mode {
                WindowMode::List => {
                    match key.code {
                        KeyCode::Char(' ') => {} // Mark task as done
                        KeyCode::Char('n') => {
                            self.new_task = true;
                            self.mode = WindowMode::Task(EditMode::Edit(EditState::Title))
                        } // New Task
                        KeyCode::Down => self.tasks.next(),
                        KeyCode::Up => self.tasks.previous(),
                        KeyCode::Enter => {
                            if let Some(i) = self.tasks.state.selected() {
                                let task = self.get_task(i)?;

                                self.task.title = task.title;
                                self.task.description = task.description;
                                self.cursor_pos_x =
                                    self.task.description.split('\n').last().unwrap().len() as u16;
                                self.cursor_pos_y =
                                    self.task.description.split('\n').count().saturating_sub(1)
                                        as u16;

                                self.new_task = false;
                                self.mode = WindowMode::Task(EditMode::View)
                            }
                        }
                        KeyCode::Esc => return Ok(Status::ExitApp),
                        _ => {}
                    }
                }
                WindowMode::Task(EditMode::View) => match key.code {
                    KeyCode::Esc => {
                        self.task.title.clear();
                        self.task.description.clear();
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
                        if self.new_task {
                            self.add_to_db()?;
                        } else {
                            self.update_db()?;
                        }

                        self.update_tasks()?;

                        self.mode = WindowMode::List;
                    }
                    _ => {}
                },
                WindowMode::Task(EditMode::Edit(EditState::Title)) => match key.code {
                    KeyCode::Esc => self.mode = WindowMode::Task(EditMode::View),
                    KeyCode::Char(n) => {
                        if self.task.title.width() as u16 == self.width.saturating_sub(3) {
                            return Ok(Status::Ignore);
                        }

                        self.task.title.push(n);
                    }
                    KeyCode::Backspace => {
                        self.task.title.pop();
                    }
                    _ => {}
                },
                WindowMode::Task(EditMode::Edit(EditState::Task)) => match key.code {
                    KeyCode::Esc => self.mode = WindowMode::Task(EditMode::View),
                    KeyCode::Char(n) => {
                        self.task.description.push(n);
                        if self.cursor_pos_x as u16 == self.width.saturating_sub(3) {
                            self.task.description.push('\n');
                            self.cursor_pos_y += 1;
                            return Ok(Status::Ignore);
                        }
                    }
                    KeyCode::Enter => {
                        self.task.description.push('\n');
                        self.cursor_pos_y += 1;
                    }
                    KeyCode::Backspace => {
                        self.task.description.pop();

                        if self.cursor_pos_x == 0 {
                            self.cursor_pos_y = self.cursor_pos_y.saturating_sub(1);
                            self.task.description.pop();
                            return Ok(Status::Ignore);
                        }
                    }
                    _ => {}
                },
            }
        }
        Ok(Status::Ignore)
    }
}
