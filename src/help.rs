use ratatui::{Frame, layout::Rect, style::Stylize, text::Line, widgets::Paragraph};

#[derive(Default)]
pub struct Help;

impl Help {
    pub fn draw(&self, frame: &mut Frame, area: Rect) {
        let text = vec![
            Line::from("[↑↓ to move, enter to interact, q to quit]"),
            Line::from(""),
        ];

        let paragraph = Paragraph::new(text).dark_gray();

        frame.render_widget(paragraph, area);
    }
}
