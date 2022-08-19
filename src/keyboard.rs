use crate::ui::{EditMode, EditState, Task, WindowMode};
use crate::App;
use anyhow::Result;
use crossterm::event;
use crossterm::event::{Event, KeyCode};
use std::error::Error;
use std::fmt::{Display, Formatter};
use unicode_width::UnicodeWidthStr;

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
                    KeyCode::Char('n') => self.new_task()?,
                    KeyCode::Char('d') => self.delete_task()?,
                    KeyCode::Char('s') => self.mode = WindowMode::Stats,
                    // KeyCode::Char('a') => self.config.add_exp(30)?,
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
                WindowMode::Task(EditMode::Edit(EditState::Title)) => {
                    let (_, task) = self.task().unwrap();
                    let task = &mut *task.borrow_mut();

                    match key.code {
                        KeyCode::Esc => self.mode = WindowMode::Task(EditMode::View),
                        KeyCode::Char(c) => input(&mut task.title, self.width, c),
                        KeyCode::Backspace => {
                            task.title.pop();
                        }
                        _ => {}
                    }
                }
                WindowMode::Task(EditMode::Edit(EditState::Task)) => match key.code {
                    KeyCode::Esc => self.mode = WindowMode::Task(EditMode::View),
                    KeyCode::Char(n) => self.description_input(n),
                    KeyCode::Enter => self.description_enter(),
                    KeyCode::Backspace => self.description_bs(),
                    _ => {}
                },
                WindowMode::Preferences(false) => match key.code {
                    KeyCode::Char('e') => self.preferences_edit()?,
                    KeyCode::Esc => self.back_to_task(),
                    KeyCode::Up => self.preferences.previous(),
                    KeyCode::Down => self.preferences.next(),
                    _ => {}
                },
                WindowMode::Preferences(true) => match key.code {
                    KeyCode::Esc => self.back_to_pref(),
                    KeyCode::Char(c) => input(&mut self.preferences_input, self.width, c),
                    KeyCode::Backspace => {
                        self.preferences_input.pop();
                    }
                    KeyCode::Enter => self.preferences_edit()?,
                    _ => {}
                },
                WindowMode::Stats => match key.code {
                    KeyCode::Esc => self.back_to_list(),
                    _ => {}
                },
            }
        }
        Ok(())
    }

    fn mark_task(&mut self) -> Result<()> {
        if let Some((id, task)) = self.task() {
            let task = &mut *task.borrow_mut();

            task.done = !task.done;

            self.database.insert(id, task)?;
            self.update_tasks()?;
        }

        Ok(())
    }

    fn new_task(&mut self) -> Result<()> {
        let task = Task {
            title: "New task".to_string(),
            creation_date: chrono::Local::now().naive_local(),
            ..Default::default()
        };
        self.database.add(&task)?;
        self.update_tasks()?;

        Ok(())
    }

    fn delete_task(&mut self) -> Result<()> {
        if let Some((id, _)) = self.task() {
            self.database.remove(id)?;
            self.update_tasks()?;
        }

        Ok(())
    }

    fn edit_task(&mut self) -> Result<()> {
        if let Some((_, task)) = self.task() {
            let task = &mut *task.borrow_mut();

            self.cursor_pos_x = task.description.lines().last().unwrap_or_default().len() as u16;
            self.cursor_pos_y = task.description.lines().count().saturating_sub(1) as u16;

            self.mode = WindowMode::Task(EditMode::View);
        }

        Ok(())
    }

    fn save_task(&mut self) -> Result<()> {
        let (id, task) = self.task().unwrap();
        let task = &mut *task.borrow_mut();

        if task.title.is_empty() {
            return Ok(());
        } else {
            self.database.insert(id, task)?;
        }

        self.update_tasks()?;
        self.back_to_list();

        Ok(())
    }

    fn back_to_list(&mut self) {
        self.mode = WindowMode::List;
    }

    fn back_to_task(&mut self) {
        self.preferences_input.clear();
        self.mode = WindowMode::Task(EditMode::View);
    }

    fn back_to_pref(&mut self) {
        self.mode = WindowMode::Preferences(false);
        self.preferences_input.clear();
    }

    fn description_input(&mut self, n: char) {
        let (_, task) = self.task().unwrap();
        let task = &mut *task.borrow_mut();

        task.description.push(n);

        if self.cursor_pos_x as u16 == self.width.saturating_sub(3) {
            task.description.push('\n');
            self.cursor_pos_y += 1;
        }
    }

    fn description_enter(&mut self) {
        let (_, task) = self.task().unwrap();
        let task = &mut *task.borrow_mut();

        task.description.push('\n');
        self.cursor_pos_y += 1;
    }

    fn description_bs(&mut self) {
        let (_, task) = self.task().unwrap();
        let task = &mut *task.borrow_mut();

        task.description.pop();

        if self.cursor_pos_x == 0 {
            self.cursor_pos_y = self.cursor_pos_y.saturating_sub(1);
            task.description.pop();
        }
    }

    fn preferences_edit(&mut self) -> Result<()> {
        let (_, task) = self.task().unwrap();

        let task = &mut *task.borrow_mut();

        if let Some(i) = self.preferences.state.selected() {
            if i == 0 {
                task.preferences.daily_repeat = !task.preferences.daily_repeat;

                return Ok(())
            } else if self.mode == WindowMode::Preferences(false) {
                self.mode = WindowMode::Preferences(true);

                return Ok(());
            }

            match i {
                1 => {
                    if let Ok(date) = chrono::NaiveDateTime::parse_from_str(
                        &self.preferences_input,
                        "%Y-%m-%d %H:%M:%S",
                    ) {
                        task.preferences.expire = date;
                        self.back_to_pref();
                    }
                }
                2 => {
                    if let Ok(exp) = self.preferences_input.parse::<u32>() {
                        task.preferences.exp = exp;
                        self.back_to_pref();
                    }
                }
                _ => {}
            }
        }

        Ok(())
    }
}

fn input(input: &mut String, width: u16, n: char) {
    if input.width() as u16 == width.saturating_sub(3) {
        return;
    }

    input.push(n);
}
