use arboard::Clipboard;
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

    pub fn handler(&mut self, key: KeyEvent) -> Option<String> {
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
                KeyCode::Char('h') | KeyCode::Left => {
                    self.move_cursor_left();
                }
                KeyCode::Char('l') | KeyCode::Right => {
                    self.move_cursor_right();
                }
                KeyCode::Char('p') => {
                    self.paste_from_clipboard();
                }
                KeyCode::Char('q') | KeyCode::Esc => {
                    self.is_active = false;
                    self.input = "".to_string();
                }
                KeyCode::Char('[') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                    self.is_active = false;
                    self.input = "".to_string();
                }
                KeyCode::Enter => {
                    self.is_active = false;
                    self.input_mode = InputMode::Normal;
                    let input = self.input.clone();
                    self.input = "".to_string();
                    return Some(input);
                }
                _ => {}
            },
            InputMode::Editing => match key.code {
                KeyCode::Enter => {
                    self.is_active = false;
                    self.input_mode = InputMode::Normal;
                    let input = self.input.clone();
                    self.input = "".to_string();
                    return Some(input);
                }
                KeyCode::Char('w') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                    self.delete_word()
                }
                KeyCode::Char('v') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                    self.paste_from_clipboard();
                }
                KeyCode::Char('[') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                    self.input_mode = InputMode::Normal;
                }
                KeyCode::Char(to_insert) => self.enter_char(to_insert),
                KeyCode::Backspace => self.delete_char(),
                KeyCode::Left => self.move_cursor_left(),
                KeyCode::Right => self.move_cursor_right(),
                KeyCode::Esc => self.input_mode = InputMode::Normal,
                _ => {}
            },
        }
        None
    }

    fn paste_from_clipboard(&mut self) {
        if let Ok(mut clipboard) = Clipboard::new()
            && let Ok(text) = clipboard.get_text()
        {
            let sanitized: String = text.chars().filter(|c| *c != '\n' && *c != '\r').collect();
            for c in sanitized.chars() {
                self.enter_char(c);
            }
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
                "Enter".bold(),
                " to record the message,".into(),
                " ctrl+shift+v".bold(),
                " to paste".into(),
            ],
        };

        let text = Text::from(Line::from(msg)).style(style);
        let help_message = Paragraph::new(text);
        frame.render_widget(help_message, layout[0]);

        // Inner width = total width minus 2 border chars, reserve 2 spaces for cursor room
        let inner_width = layout[1].width as usize;
        let cursor_reserved: usize = 2;
        let visible_width = inner_width.saturating_sub(cursor_reserved);

        // Scroll offset: if cursor is beyond visible area, shift so cursor stays near the end
        let scroll_offset = if self.character_index >= visible_width {
            self.character_index - visible_width + cursor_reserved
        } else {
            0
        };

        let visible_input = self
            .input
            .char_indices()
            .skip(scroll_offset)
            .take(inner_width)
            .map(|(_, c)| c)
            .collect::<String>();

        let input =
            Paragraph::new(visible_input.as_str())
                .style(style)
                .block(Block::bordered().title(match self.input_mode {
                    InputMode::Editing => "Insert mode",
                    InputMode::Normal => "Normal mode",
                }));
        frame.render_widget(input, layout[1]);

        // Cursor position relative to the visible slice
        let cursor_visible_x = self.character_index.saturating_sub(scroll_offset);

        match self.input_mode {
            #[expect(clippy::cast_possible_truncation)]
            InputMode::Normal => {
                frame.set_cursor_position(Position::new(
                    layout[1].x + cursor_visible_x as u16 + 1,
                    layout[1].y + 1,
                ));
            }

            #[expect(clippy::cast_possible_truncation)]
            InputMode::Editing => frame.set_cursor_position(Position::new(
                layout[1].x + cursor_visible_x as u16 + 1,
                layout[1].y + 1,
            )),
        }
    }
}
