use crate::ui::{EditMode, EditState, Preferences, Task, WindowMode};
use crate::App;
use crossterm::event;
use crossterm::event::{Event, KeyCode};
use std::error::Error;
use std::fmt::{Display, Formatter};

use anyhow::Result;
use unicode_width::UnicodeWidthStr;
use crate::ui::Preferences::{Expire, Repeat};

#[derive(Debug)]
pub struct ExitApp;

impl Display for ExitApp {
    fn fmt(&self, _: &mut Formatter<'_>) -> std::fmt::Result {
        Ok(())
    }
}

impl Error for ExitApp {}

impl App {
    pub fn event(&mut self) -> Result<()> {
        if let Event::Key(key) = event::read()? {
            match self.mode {
                WindowMode::List => match key.code {
                    KeyCode::Char(' ') => self.mark_task()?,
                    KeyCode::Char('n') => self.new_task(),
                    KeyCode::Char('d') => self.delete_task()?,
                    KeyCode::Down => self.tasks.next(),
                    KeyCode::Up => self.tasks.previous(),
                    KeyCode::Enter => self.edit_task()?,
                    KeyCode::Esc => return Err(ExitApp)?,
                    _ => {}
                },
                WindowMode::Task(EditMode::View) => match key.code {
                    KeyCode::Esc => self.back_to_list(),
                    KeyCode::Char('t') => {
                        self.mode = WindowMode::Task(EditMode::Edit(EditState::Title))
                    }
                    KeyCode::Char('e') => {
                        self.mode = WindowMode::Task(EditMode::Edit(EditState::Task))
                    }
                    KeyCode::Char('s') => self.save_task()?,
                    KeyCode::Char('p') => {
                        self.mode = WindowMode::Preferences(false);
                    }
                    _ => {}
                },
                WindowMode::Task(EditMode::Edit(EditState::Title)) => match key.code {
                    KeyCode::Esc => self.mode = WindowMode::Task(EditMode::View),
                    KeyCode::Char(n) => self.title_input(n),
                    KeyCode::Backspace => {
                        self.task.title.pop();
                    }
                    _ => {}
                },
                WindowMode::Task(EditMode::Edit(EditState::Task)) => match key.code {
                    KeyCode::Esc => self.mode = WindowMode::Task(EditMode::View),
                    KeyCode::Char(n) => self.description_input(n),
                    KeyCode::Enter => self.description_enter(),
                    KeyCode::Backspace => self.description_bs(),
                    _ => {}
                },
                WindowMode::Preferences(false) => {
                    match key.code {
                        KeyCode::Char('e') => self.preferences_edit(),
                        KeyCode::Esc => self.mode = WindowMode::Task(EditMode::View),
                        KeyCode::Up => self.preferences.previous(),
                        KeyCode::Down => self.preferences.next(),
                        _ => {}
                    }
                }
                WindowMode::Preferences(true) => {
                    match key.code {
                        KeyCode::Esc => self.mode = WindowMode::Preferences(false),
                        KeyCode::Char(c) => self.preferences_input.push(c),
                        KeyCode::Backspace => {
                            self.preferences_input.pop();
                        },
                        _ => {}
                    }
                }
            }
        }
        Ok(())
    }

    fn mark_task(&mut self) -> Result<()> {
        if let Some(id) = self.tasks.state.selected() {
            let mut task = self.get_task(id)?;
            task.done = !task.done;

            self.task = task;

            self.update_db()?;
            self.update_tasks()?;

            self.task = Task::default();
        }

        Ok(())
    }

    fn new_task(&mut self) {
        self.new_task = true;
        self.mode = WindowMode::Task(EditMode::Edit(EditState::Title));
    }

    fn delete_task(&mut self) -> Result<()> {
        self.rm_from_db()?;
        self.update_tasks()
    }

    fn edit_task(&mut self) -> Result<()> {
        if let Some(id) = self.tasks.state.selected() {
            let task = self.get_task(id)?;

            self.task = task;

            self.cursor_pos_x = self.task.description.split('\n').last().unwrap().len() as u16;
            self.cursor_pos_y = self.task.description.split('\n').count().saturating_sub(1) as u16;

            self.new_task = false;
            self.mode = WindowMode::Task(EditMode::View);
        }

        Ok(())
    }

    fn save_task(&mut self) -> Result<()> {
        if self.task.title.is_empty() {
            return Ok(())
        }
        
        if self.new_task {
            self.add_to_db()?;
        } else {
            self.update_db()?;
        }

        self.update_tasks()?;
        self.back_to_list();

        Ok(())
    }

    fn back_to_list(&mut self) {
        self.task = Task::default();
        self.mode = WindowMode::List;
    }

    fn title_input(&mut self, n: char) {
        if self.task.title.width() as u16 == self.width.saturating_sub(3) {
            return;
        }

        self.task.title.push(n);
    }

    fn description_input(&mut self, n: char) {
        self.task.description.push(n);

        if self.cursor_pos_x as u16 == self.width.saturating_sub(3) {
            self.task.description.push('\n');
            self.cursor_pos_y += 1;
        }
    }

    fn description_enter(&mut self) {
        self.task.description.push('\n');
        self.cursor_pos_y += 1;
    }

    fn description_bs(&mut self) {
        self.task.description.pop();

        if self.cursor_pos_x == 0 {
            self.cursor_pos_y = self.cursor_pos_y.saturating_sub(1);
            self.task.description.pop();
        }
    }

    fn preferences_edit(&mut self) {
        if let Some(i) = self.preferences.state.selected() {
            match self.preferences.items[i] {
                (_, Repeat(b)) => self.task.daily_repeat = !b,
                (_, Expire(_)) => self.mode = WindowMode::Preferences(true),
                _ => {}
            }
        }
    }
}
