use crate::ui::{EditMode, EditState, Task, WindowMode};
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
                WindowMode::List => match key.code {
                    KeyCode::Char(' ') => {
                        let mut task = self.get_task()?;
                        task.done = !task.done;

                        self.task = task;

                        self.update_db()?;
                        self.update_tasks()?;

                        self.task = Task::default();
                    }
                    KeyCode::Char('n') => {
                        self.new_task = true;
                        self.mode = WindowMode::Task(EditMode::Edit(EditState::Title))
                    }
                    KeyCode::Char('d') => {
                        self.rm_from_db()?;
                        self.update_tasks()?;
                    }
                    KeyCode::Down => self.tasks.next(),
                    KeyCode::Up => self.tasks.previous(),
                    KeyCode::Enter => {
                        let task = self.get_task()?;

                        self.task = task;

                        self.cursor_pos_x =
                            self.task.description.split('\n').last().unwrap().len() as u16;
                        self.cursor_pos_y =
                            self.task.description.split('\n').count().saturating_sub(1) as u16;

                        self.new_task = false;
                        self.mode = WindowMode::Task(EditMode::View)
                    }
                    KeyCode::Esc => return Ok(Status::ExitApp),
                    _ => {}
                },
                WindowMode::Task(EditMode::View) => match key.code {
                    KeyCode::Esc => {
                        self.task = Task::default();

                        self.cursor_pos_y = 0;
                        self.mode = WindowMode::List;
                    }
                    KeyCode::Char('t') => {
                        self.mode = WindowMode::Task(EditMode::Edit(EditState::Title))
                    }
                    KeyCode::Char('e') => {
                        self.mode = WindowMode::Task(EditMode::Edit(EditState::Task))
                    }
                    KeyCode::Char('s') => {
                        if self.new_task {
                            self.add_to_db()?;
                        } else {
                            self.update_db()?;
                        }

                        self.update_tasks()?;

                        self.task = Task::default();

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
