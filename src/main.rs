mod calculation;

use calculation::calculate_mandala;
use iced::widget::canvas::{Cache, Canvas, Geometry, Program, Text};
use iced::widget::{TextInput, button, column, container, text};
use iced::{
    Color, Element, Fill, Pixels, Point, Rectangle, Renderer, Result as IcedResult, Size, Theme,
    alignment, color,
};
use iced::{Vector, mouse};

static COLORS: [Color; 9] = [
    color!(0xE6194B),
    color!(0xF58231),
    color!(0xFFE119),
    color!(0xBFEF45),
    color!(0x3CB44B),
    color!(0x42D4F4),
    color!(0x4363D8),
    color!(0x911EB4),
    color!(0xF032E6),
];

static TEXT_SIZE: Pixels = Pixels(24.0);

#[derive(Debug, Clone)]
enum Message {
    Type(String),
    Submit,
    Return,
}

#[derive(Debug, Copy, Clone)]
enum Screen {
    Input,
    Result,
}

struct Mandala<'a> {
    result: &'a Vec<Vec<u16>>,
    cache: Cache<Renderer>,
}

impl<'a> Mandala<'a> {
    fn new(result: &'a Vec<Vec<u16>>) -> Self {
        Self {
            result,
            cache: Cache::default(),
        }
    }
}

impl<'a> Program<Message> for Mandala<'a> {
    type State = ();

    fn draw(
        &self,
        _state: &Self::State,
        renderer: &Renderer,
        _theme: &Theme,
        bounds: Rectangle,
        _cursor: mouse::Cursor,
    ) -> Vec<Geometry<Renderer>> {
        let geometry = self.cache.draw(renderer, bounds.size(), |frame| {
            let side = f32::min(frame.width(), frame.height());
            let padding_x = (frame.width() - side) / 2.0;
            let padding_y = (frame.height() - side) / 2.0;
            let block_size = side / 16.0;
            let block_half_size = block_size / 2.0;
            let text_offset: Vector = [block_half_size, block_half_size].into();

            for row in 0..16 {
                let block_padding = (side - (row as f32 + 1.0) * block_size) / 2.0;
                for col in 0..row + 1 {
                    let x = padding_x + block_padding + (col as f32) * block_size;
                    let y = padding_y + ((15 - row) as f32) * block_size;
                    let index = self.result[15 - row][col];
                    let color = COLORS.get((index - 1) as usize).unwrap();
                    let top_left_point = Point::new(x, y);

                    let text = Text {
                        content: index.to_string(),
                        position: top_left_point + text_offset,
                        color: Color::BLACK,
                        horizontal_alignment: alignment::Horizontal::Center,
                        vertical_alignment: alignment::Vertical::Center,
                        size: TEXT_SIZE,
                        ..Text::default()
                    };

                    frame.fill_rectangle(top_left_point, Size::new(block_size, block_size), *color);
                    frame.fill_text(text);
                }
            }
        });

        vec![geometry]
    }
}

#[derive(Debug, Clone)]
struct State {
    screen: Screen,
    input: String,
    calculation: Result<Vec<Vec<u16>>, String>,
}

impl State {
    fn update(&mut self, message: Message) {
        match message {
            Message::Type(text) => {
                self.input = text;
            }
            Message::Submit => {
                self.screen = Screen::Result;
                self.calculation = calculate_mandala(&self.input);
            }
            Message::Return => {
                self.screen = Screen::Input;
                self.input = "".to_string();
                self.calculation = Err("Введите текст для мандалы".to_string());
            }
        }
    }
    fn view(&self) -> Element<'_, Message> {
        match self.screen {
            Screen::Input => {
                let text = self.input.trim();

                let text_input =
                    TextInput::new("Пожалуйста, введите текст для мандалы", self.input.as_str())
                        .on_input(Message::Type)
                        .on_submit_maybe(if text.is_empty() {
                            None
                        } else {
                            Some(Message::Submit)
                        });

                let submit_button = button("Создать мандалу").on_press_maybe(if text.is_empty() {
                    None
                } else {
                    Some(Message::Submit)
                });

                container(
                    column![text_input, submit_button]
                        .align_x(alignment::Horizontal::Right)
                        .spacing(10),
                )
                .height(Fill)
                .align_y(alignment::Vertical::Center)
                .padding(20)
                .into()
            }
            Screen::Result => match &self.calculation {
                Ok(result) => container(
                    column![
                        text(&self.input).size(20),
                        Canvas::new(Mandala::new(result)).width(Fill).height(Fill),
                        button("Назад").on_press(Message::Return),
                    ]
                    .width(Fill)
                    .spacing(10)
                    .align_x(alignment::Horizontal::Center),
                )
                .height(Fill)
                .align_y(alignment::Vertical::Center)
                .padding(20)
                .into(),
                Err(error) => text(error).into(),
            },
        }
    }
}

impl Default for State {
    fn default() -> Self {
        State {
            screen: Screen::Input,
            input: "".to_string(),
            calculation: Err("Введите текст для мандалы".to_string()),
        }
    }
}

fn main() -> IcedResult {
    iced::run("Мандала", State::update, State::view)
}
