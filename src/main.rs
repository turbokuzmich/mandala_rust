mod calculation;

use calculation::calculate_mandala;
use iced::widget::{TextInput, button, column, container, text};
use iced::{Element, Fill, Result as IcedResult, alignment};

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

#[derive(Debug, Clone)]
struct State {
    screen: Screen,
    input: String,
    calculation: Option<Result<Vec<Vec<u16>>, String>>,
}

impl State {
    fn update(&mut self, message: Message) {
        match message {
            Message::Type(text) => {
                self.input = text;
            }
            Message::Submit => {
                self.screen = Screen::Result;
                self.calculation = Some(calculate_mandala(&self.input));
                println!("{:?}", self.calculation);
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
            _ => text(self.input.trim()).into(),
        }
    }
}

impl Default for State {
    fn default() -> Self {
        State {
            screen: Screen::Input,
            input: "".to_string(),
            calculation: None,
        }
    }
}

fn main() -> IcedResult {
    // let text = "хуй".to_string();
    // let _ = calculate_mandala(text.as_str());
    iced::run("Мандала", State::update, State::view)
}
