mod calculation;

use calculation::calculate_mandala;
use iced::mouse;
use iced::widget::canvas::{Cache, Canvas, Geometry, Program};
use iced::widget::{TextInput, button, column, container, text};
use iced::{
    Color, Element, Fill, Point, Rectangle, Renderer, Result as IcedResult, Theme, alignment,
};

#[derive(Debug, Clone)]
enum Message {
    Type(String),
    Submit,
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
        println!("new: {:?}", result);
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
            frame.fill_rectangle(Point::new(0.0, 0.0), frame.size(), Color::WHITE);
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
                        Canvas::new(Mandala::new(result)).width(Fill).height(Fill)
                    ]
                    .width(Fill)
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
