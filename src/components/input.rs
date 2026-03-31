/// A simple example demonstrating how to handle user input.
///
/// This is a bit out of the scope of
/// the library as it does not provide any input handling out of the box. However, it may helps
/// some to get started.
///
/// This is a very simple example:
///   * An input box always focused. Every character you type is registered here.
///   * An entered character is inserted at the cursor position.
///   * Pressing Backspace erases the left character before the cursor position
///   * Pressing Enter pushes the current input in the history of previous messages.
///
/// **Note:** as this is a relatively simple example unicode characters are unsupported and
/// their use will result in undefined behaviour.
///
/// See also <https://github.com/ratatui/ratatui-textarea> and <https://github.com/sayanarijit/tui-input>/
///
/// This example runs with the Ratatui library code in the branch that you are currently
/// reading. See the [`latest`] branch for the code which works with the most recent Ratatui
/// release.
///
/// [`latest`]: https://github.com/ratatui/ratatui/tree/latest
use ratatui::Frame;
use ratatui::layout::{Constraint, Direction, Layout, Position, Rect};
use ratatui::style::{Color, Modifier, Style, Stylize};
use ratatui::text::{Line, Text};
use ratatui::widgets::{Block, Paragraph};

/// App holds the state of the application
pub struct Input {
    pub is_active: bool,
    /// Current value of the input box
    pub input: String,
    /// Position of cursor in the editor area.
    pub character_index: usize,
    /// Current input mode
    pub input_mode: InputMode,
}

pub enum InputMode {
    Normal,
    Editing,
}

impl Input {
    pub const fn new(is_active: bool) -> Self {
        Self {
            is_active,
            input: String::new(),
            input_mode: InputMode::Editing,
            character_index: 0,
        }
    }

    pub fn move_cursor_left(&mut self) {
        let cursor_moved_left = self.character_index.saturating_sub(1);
        self.character_index = self.clamp_cursor(cursor_moved_left);
    }

    pub fn move_cursor_right(&mut self) {
        let cursor_moved_right = self.character_index.saturating_add(1);
        self.character_index = self.clamp_cursor(cursor_moved_right);
    }

    pub fn enter_char(&mut self, new_char: char) {
        let index = self.byte_index();
        self.input.insert(index, new_char);
        self.move_cursor_right();
    }

    /// Returns the byte index based on the character position.
    ///
    /// Since each character in a string can contain multiple bytes, it's necessary to calculate
    /// the byte index based on the index of the character.
    pub fn byte_index(&self) -> usize {
        self.input
            .char_indices()
            .map(|(i, _)| i)
            .nth(self.character_index)
            .unwrap_or(self.input.len())
    }

    pub fn delete_char(&mut self) {
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

    pub fn clamp_cursor(&self, new_cursor_pos: usize) -> usize {
        new_cursor_pos.clamp(0, self.input.chars().count())
    }

    pub const fn reset_cursor(&mut self) {
        self.character_index = 0;
    }

    // fn run(mut self, terminal: &mut DefaultTerminal) -> Result<()> {
    //     loop {
    //         terminal.draw(|frame| self.render(frame))?;
    //
    //         if let Some(key) = event::read()?.as_key_press_event() {
    //             match self.input_mode {
    //                 InputMode::Normal => match key.code {
    //                     KeyCode::Char('e') => {
    //                         self.input_mode = InputMode::Editing;
    //                     }
    //                     KeyCode::Char('q') => {
    //                         return Ok(());
    //                     }
    //                     _ => {}
    //                 },
    //                 InputMode::Editing if key.kind == KeyEventKind::Press => match key.code {
    //                     KeyCode::Enter => self.submit_message(),
    //                     KeyCode::Char(to_insert) => self.enter_char(to_insert),
    //                     KeyCode::Backspace => self.delete_char(),
    //                     KeyCode::Left => self.move_cursor_left(),
    //                     KeyCode::Right => self.move_cursor_right(),
    //                     KeyCode::Esc => self.input_mode = InputMode::Normal,
    //                     _ => {}
    //                 },
    //                 InputMode::Editing => {}
    //             }
    //         }
    //     }
    // }

    pub fn render(&self, frame: &mut Frame, area: Rect) {
        let layout = Layout::default()
            .direction(Direction::Vertical)
            .constraints(vec![Constraint::Length(1), Constraint::Length(3)])
            .split(area);

        let (msg, style) = match self.input_mode {
            InputMode::Normal => (
                vec![
                    "Press ".into(),
                    "q".bold(),
                    " to exit, ".into(),
                    "e".bold(),
                    " to start editing.".bold(),
                ],
                Style::default().add_modifier(Modifier::RAPID_BLINK),
            ),
            InputMode::Editing => (
                vec![
                    "Press ".into(),
                    "Esc".bold(),
                    " to stop editing, ".into(),
                    "Enter".bold(),
                    " to record the message".into(),
                ],
                Style::default(),
            ),
        };
        let text = Text::from(Line::from(msg)).patch_style(style);
        let help_message = Paragraph::new(text);
        frame.render_widget(help_message, layout[0]);

        let input = Paragraph::new(self.input.as_str())
            .style(match self.input_mode {
                InputMode::Normal => Style::default(),
                InputMode::Editing => Style::default().fg(Color::Yellow),
            })
            .block(Block::bordered().title("Input"));
        frame.render_widget(input, layout[1]);
        match self.input_mode {
            // Hide the cursor. `Frame` does this by default, so we don't need to do anything here
            InputMode::Normal => {}

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
