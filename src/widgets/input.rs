use crossterm::event::KeyCode;
use crossterm::event::KeyEvent;
use crossterm::event::KeyModifiers;
use ratatui::Frame;

use ratatui::layout::Constraint;
use ratatui::layout::Direction;
use ratatui::layout::Layout;
use ratatui::layout::Position;
use ratatui::layout::Rect;

use ratatui::style::Style;
use ratatui::style::Stylize;

use ratatui::text::Line;
use ratatui::text::Text;

use ratatui::widgets::Block;
use ratatui::widgets::Paragraph;

use crate::config::Theme;

/// App holds the state of the application
pub struct Input {
    /// Current value of the input box
    pub input: String,
    /// Position of cursor in the editor area.
    character_index: usize,
    /// Current input mode
    pub input_mode: InputMode,
    pub is_active: bool,
}

pub enum InputMode {
    Normal,
    Editing,
}

impl Input {
    pub const fn new() -> Self {
        Self {
            input: String::new(),
            input_mode: InputMode::Editing,
            character_index: 0,
            is_active: false,
        }
    }

    fn move_cursor_left(&mut self) {
        let cursor_moved_left = self.character_index.saturating_sub(1);
        self.character_index = self.clamp_cursor(cursor_moved_left);
    }

    fn move_cursor_right(&mut self) {
        let cursor_moved_right = self.character_index.saturating_add(1);
        self.character_index = self.clamp_cursor(cursor_moved_right);
    }

    fn move_cursor_to_end(&mut self) {
        self.character_index = self.input.chars().count();
    }

    fn enter_char(&mut self, new_char: char) {
        let index = self.byte_index();
        self.input.insert(index, new_char);
        self.move_cursor_right();
    }

    /// Returns the byte index based on the character position.
    ///
    /// Since each character in a string can contain multiple bytes, it's necessary to calculate
    /// the byte index based on the index of the character.
    fn byte_index(&self) -> usize {
        self.input
            .char_indices()
            .map(|(i, _)| i)
            .nth(self.character_index)
            .unwrap_or(self.input.len())
    }

    fn delete_char(&mut self) {
        let is_not_cursor_leftmost = self.character_index != 0;
        if is_not_cursor_leftmost {
            // Method "remove" is not used on the saved text for deleting the selected char.
            // Reason: Using remove on String works on bytes instead of the chars.
            // Using remove would require special care because of char boundaries.

            let current_index = self.character_index;
            let from_left_to_current_index = current_index - 1;

            // Getting all characters before the selected character.
            let before_char_to_delete = self.input.chars().take(from_left_to_current_index);
            // Getting all characters after selected character.
            let after_char_to_delete = self.input.chars().skip(current_index);

            // Put all characters together except the selected one.
            // By leaving the selected one out, it is forgotten and therefore deleted.
            self.input = before_char_to_delete.chain(after_char_to_delete).collect();
            self.move_cursor_left();
        }
    }

    fn delete_word(&mut self) {
        if self.character_index == 0 {
            return;
        }

        let chars: Vec<char> = self.input.chars().collect();
        let mut end_pos = self.character_index;

        // Move backwards to the end of the word
        while end_pos > 0 && chars[end_pos - 1] == ' ' {
            end_pos -= 1;
        }

        // Move backwards to the start of the word
        while end_pos > 0 && chars[end_pos - 1] != ' ' {
            end_pos -= 1;
        }

        // Delete characters from end_pos to character_index
        let before = chars[..end_pos].iter().collect::<String>();
        let after = chars[self.character_index..].iter().collect::<String>();
        self.input = before + &after;
        self.character_index = end_pos;
    }

    fn clamp_cursor(&self, new_cursor_pos: usize) -> usize {
        new_cursor_pos.clamp(0, self.input.chars().count())
    }

    pub const fn reset_cursor(&mut self) {
        self.character_index = 0;
    }

    pub fn handler(&mut self, key: KeyEvent) {
        match self.input_mode {
            InputMode::Normal => match key.code {
                KeyCode::Char('i') => {
                    self.input_mode = InputMode::Editing;
                }
                KeyCode::Char('A') => {
                    self.move_cursor_to_end();
                    self.input_mode = InputMode::Editing;
                }
                KeyCode::Char('a') => {
                    self.move_cursor_right();
                    self.input_mode = InputMode::Editing;
                }
                KeyCode::Char('h') => {
                    self.move_cursor_left();
                }
                KeyCode::Char('l') => {
                    self.move_cursor_right();
                }
                KeyCode::Char('q') | KeyCode::Esc => {
                    self.is_active = false;
                    self.input = "".to_string();
                }
                KeyCode::Enter => {
                    self.is_active = false;
                    self.input_mode = InputMode::Normal;
                    self.input = "".to_string();
                }
                _ => {}
            },
            InputMode::Editing => match key.code {
                KeyCode::Enter => {
                    self.is_active = false;
                    self.input_mode = InputMode::Normal;
                    self.input = "".to_string();
                }
                KeyCode::Char('w') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                    self.delete_word()
                }
                KeyCode::Char(to_insert) => self.enter_char(to_insert),
                KeyCode::Backspace => self.delete_char(),
                KeyCode::Left => self.move_cursor_left(),
                KeyCode::Right => self.move_cursor_right(),
                KeyCode::Esc => self.input_mode = InputMode::Normal,
                _ => {}
            },
        }
    }

    pub fn render(&self, frame: &mut Frame, area: Rect, theme: &Theme) {
        let layout = Layout::default()
            .direction(Direction::Vertical)
            .constraints(vec![Constraint::Length(1), Constraint::Length(3)])
            .split(area);

        let style = Style::default()
            .fg(Theme::color(&theme.general.foreground))
            .bg(Theme::color(&theme.general.background));

        let msg = match self.input_mode {
            InputMode::Normal => vec![
                "Press ".into(),
                "i".bold(),
                " to edit, ".into(),
                "Enter".bold(),
                " to record the message, ".into(),
                "Esc".bold(),
                " to exit.".into(),
            ],
            InputMode::Editing => vec![
                "Press ".into(),
                "Esc".bold(),
                " to stop editing, ".into(),
                "Enter".bold(),
                " to record the message".into(),
            ],
        };

        let text = Text::from(Line::from(msg)).style(style);
        let help_message = Paragraph::new(text);
        frame.render_widget(help_message, layout[0]);

        let input =
            Paragraph::new(self.input.as_str())
                .style(style)
                .block(Block::bordered().title(match self.input_mode {
                    InputMode::Editing => "Insert mode",
                    InputMode::Normal => "Normal mode",
                }));
        frame.render_widget(input, layout[1]);

        match self.input_mode {
            // Show a bar cursor in Normal mode
            #[expect(clippy::cast_possible_truncation)]
            InputMode::Normal => {
                frame.set_cursor_position(Position::new(
                    layout[1].x + self.character_index as u16 + 1,
                    layout[1].y + 1,
                ));
            }

            // Make the cursor visible and ask ratatui to put it at the specified coordinates after
            // rendering
            #[expect(clippy::cast_possible_truncation)]
            InputMode::Editing => frame.set_cursor_position(Position::new(
                // Draw the cursor at the current position in the input field.
                // This position can be controlled via the left and right arrow key
                layout[1].x + self.character_index as u16 + 1,
                // Move one line down, from the border to the input line
                layout[1].y + 1,
            )),
        }
    }
}
