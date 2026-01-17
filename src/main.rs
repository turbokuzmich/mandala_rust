mod calculation;
// mod export;
mod pdf;

use calculation::calculate_mandala;
// use export::save_mandala_pdf;
use iced::widget::canvas::{Cache, Canvas, Geometry, Program, Text};
use iced::widget::{TextInput, button, column, container, row, text};
use iced::{
    Color, Element, Fill, Pixels, Point, Rectangle, Renderer, Result as IcedResult, Size, Task,
    Theme, Vector, alignment, application, color, mouse,
};
use opener::reveal;
use pdf::save_mandala_pdf;
use std::path::PathBuf;

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
    Export,
    Open,
    Exported(Result<PathBuf, String>),
}

#[derive(Debug, Copy, Clone)]
enum Screen {
    Input,
    Result,
}

#[derive(Debug, Clone)]
enum ExportState {
    Idle,
    Saving,
    Completed(Result<PathBuf, String>),
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
    export: ExportState,
}

impl State {
    fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            Message::Type(text) => {
                self.input = text;

                Task::none()
            }
            Message::Submit => {
                self.screen = Screen::Result;
                self.calculation = calculate_mandala(&self.input);

                Task::none()
            }
            Message::Return => {
                self.screen = Screen::Input;
                self.input = "".to_string();
                self.calculation = Err("Введите текст для мандалы".to_string());
                self.export = ExportState::Idle;

                Task::none()
            }
            Message::Export => {
                self.export = ExportState::Saving;

                let calculation = self.calculation.to_owned().unwrap();
                let input = self.input.to_owned();

                Task::perform(save_mandala_pdf(calculation, input), Message::Exported)
            }
            Message::Exported(result) => {
                self.export = ExportState::Completed(result);

                Task::none()
            }
            Message::Open => {
                let path = match self.export {
                    ExportState::Completed(ref result) => result.as_ref().ok(),
                    _ => None,
                };

                if let Some(path) = path {
                    let _ = reveal(path);
                }

                self.export = ExportState::Idle;

                Task::none()
            }
        }
    }
    fn view(&self) -> Element<'_, Message> {
        match self.screen {
            Screen::Input => {
                let text = self.input.trim();

                let input =
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
                    column![input, submit_button]
                        .align_x(alignment::Horizontal::Right)
                        .spacing(10),
                )
                .height(Fill)
                .align_y(alignment::Vertical::Center)
                .padding(20)
                .into()
            }
            Screen::Result => match &self.calculation {
                Ok(result) => {
                    let notification: Element<'_, Message> = match self.export {
                        ExportState::Idle => {
                            text("Нажмите «Сохранить» для сохранения мандалы в PDF").into()
                        }
                        ExportState::Saving => text("Сохранение...").into(),
                        ExportState::Completed(ref result) => match result {
                            Ok(path) => text(format!("Сохранено в {}", path.display())).into(),
                            Err(error) => text(format!("Ошибка: {}", error)).into(),
                        },
                    };

                    let save_button = match self.export {
                        ExportState::Completed(ref result) if result.is_ok() => {
                            button("Открыть").on_press(Message::Open)
                        }
                        _ => button("Сохранить").on_press(Message::Export),
                    };

                    container(
                        column![
                            text(&self.input).size(20),
                            Canvas::new(Mandala::new(result)).width(Fill).height(Fill),
                            row![button("Назад").on_press(Message::Return), save_button,]
                                .spacing(10),
                            notification,
                        ]
                        .width(Fill)
                        .spacing(10)
                        .align_x(alignment::Horizontal::Center),
                    )
                    .height(Fill)
                    .align_y(alignment::Vertical::Center)
                    .padding(20)
                    .into()
                }
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
            export: ExportState::Idle,
        }
    }
}

fn main() -> IcedResult {
    let app = application("Мандала", State::update, State::view);

    app.run()
}
