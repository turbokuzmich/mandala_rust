mod calculation;

use calculation::calculate_mandala;
use iced::widget::canvas::{Cache, Canvas, Geometry, Program, Text};
use iced::widget::{TextInput, button, column, container, row, text};
use iced::{
    Color, Element, Fill, Pixels, Point, Rectangle, Renderer, Result as IcedResult, Size, Task,
    Theme, Vector, alignment, application, color, mouse,
};
use std::path::PathBuf;

use svg::Document;
use svg::node::Text as SvgTextNode;
use svg::node::element::{Group, Line, Text as SvgText};

use glam::{Mat2, Vec2};

use printpdf::{
    Mm, Op, ParsedFont, PdfDocument, PdfPage, PdfSaveOptions, Point as PrintPdfPoint, Pt, Svg,
    TextAlign, TextShapingOptions, XObjectTransform,
};

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

static ROBOTO_FONT: &[u8] = include_bytes!("./Roboto-Light.ttf");

const SKETCH_SIZE: f32 = 2480.0;

#[derive(Debug, Clone)]
enum Message {
    Type(String),
    Submit,
    Return,
    Export,
    Exported(Result<PathBuf, ()>),
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
    export: Result<PathBuf, ()>,
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
                Task::none()
            }
            Message::Export => {
                let calculation = self.calculation.to_owned().unwrap();
                let input = self.input.to_owned();

                Task::perform(
                    async move {
                        let pdf_bytes =
                            save_svg_as_pdf(generate_svg_mandala(&calculation), &input).unwrap();

                        let export_path = dirs::download_dir().unwrap().join("mandala.pdf");

                        std::fs::write(&export_path, pdf_bytes).unwrap();

                        Ok(export_path)
                    },
                    Message::Exported,
                )
            }
            Message::Exported(result) => {
                self.export = result;
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
                Ok(result) => container(
                    column![
                        text(&self.input).size(20),
                        Canvas::new(Mandala::new(result)).width(Fill).height(Fill),
                        row![
                            button("Назад").on_press(Message::Return),
                            button("Сохранить").on_press(Message::Export)
                        ]
                        .spacing(10),
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
            export: Err(()),
        }
    }
}

fn main() -> IcedResult {
    let app = application("Мандала", State::update, State::view);

    app.run()
}

fn save_svg_as_pdf(svg: Document, text: &str) -> Result<Vec<u8>, String> {
    let mut document = PdfDocument::new("Mandala");

    let sketch_svg = Svg::parse(&svg.to_string(), &mut Vec::new()).map_err(|e| e.to_string())?;
    let sketch_id = document.add_xobject(&sketch_svg);
    let roboto_font = ParsedFont::from_bytes(ROBOTO_FONT, 0, &mut Vec::new()).unwrap();
    let roboto_font_id = document.add_font(&roboto_font);

    let text_options = TextShapingOptions {
        max_width: Some(Mm(190.0).into_pt()),
        align: TextAlign::Center,
        ..TextShapingOptions::default()
    };
    let text = roboto_font.shape_text(text, &text_options, &roboto_font_id);

    let mut ops: Vec<Op> = vec![
        Op::UseXobject {
            id: sketch_id,
            transform: XObjectTransform {
                translate_y: Some(Pt(200.0)),
                ..XObjectTransform::default()
            },
        },
        Op::SetFillColor {
            col: printpdf::Color::Rgb(printpdf::Rgb {
                r: 0.0,
                g: 0.0,
                b: 0.0,
                icc_profile: None,
            }),
        },
    ];

    ops.extend(text.get_ops(PrintPdfPoint::new(Mm(10.0), Mm(50.0))));

    let page = PdfPage::new(Mm(210.0), Mm(297.0), ops);

    let bytes = document
        .with_pages(vec![page])
        .save(&PdfSaveOptions::default(), &mut Vec::new());

    Ok(bytes)
}

fn generate_svg_mandala(digits: &Vec<Vec<u16>>) -> Document {
    const SKETCH_HALF_SIZE: f32 = SKETCH_SIZE / 2.0;
    const CELL_HALF_SIZE: f32 = SKETCH_HALF_SIZE / 33.0;
    const CELL_THREE_FOURTHS_SIZE: f32 = CELL_HALF_SIZE + CELL_HALF_SIZE / 2.0;
    let line_offset = ((CELL_HALF_SIZE * 2.0).powf(2.0) - CELL_HALF_SIZE.powf(2.0)).sqrt();
    let half_line_offset = line_offset / 2.0;
    let text_translation = Vec2::new(SKETCH_HALF_SIZE, SKETCH_HALF_SIZE);

    let build_line = |index: u8| -> Line {
        let x = line_offset * index as f32;
        let y1 = if index == 0 {
            CELL_HALF_SIZE
        } else {
            CELL_HALF_SIZE * ((index - 1) as f32)
        };
        let idx = if index == 0 { 0 } else { index - 1 };
        let count = 16 - idx;
        let y2 = y1 + (count as f32) * CELL_HALF_SIZE * 2.0;

        Line::new()
            .set("x1", x)
            .set("y1", y1)
            .set("x2", x)
            .set("y2", y2)
            .set("stroke", "black")
    };

    let group_base = (0..=16).fold(Group::new(), |acc, index| acc.add(build_line(index)));

    let build_group = |index: u8| -> [Group; 2] {
        [
            group_base.clone().set(
                "transform",
                format!(
                    "translate({} {}) rotate({} 0 {})",
                    SKETCH_HALF_SIZE,
                    0.0,
                    (index as u16) * 60,
                    SKETCH_HALF_SIZE,
                ),
            ),
            group_base.clone().set(
                "transform",
                format!(
                    "translate({} {}) scale(-1 1) rotate({} 0 {})",
                    SKETCH_HALF_SIZE,
                    0.0,
                    (index as u16) * 60,
                    SKETCH_HALF_SIZE,
                ),
            ),
        ]
    };

    let build_text = |segment: u8, row: u8, col: u8, digits: &Vec<Vec<u16>>| -> SvgText {
        let x = (col as f32) * line_offset + half_line_offset;
        let y = (row as f32) * CELL_HALF_SIZE * (-2.0) - CELL_THREE_FOURTHS_SIZE
            + col as f32 * CELL_HALF_SIZE;
        let angle = (segment as f32) * 60.0_f32.to_radians();
        let text = digits[15 - row as usize][col as usize].to_string();
        let initial_position = Vec2::new(x, y);
        let rotation = Mat2::from_angle(angle);
        let position = text_translation + rotation * initial_position;

        SvgText::new()
            .set("x", position.x)
            .set("y", position.y)
            .set("font-family", "sans-serif")
            .set("font-size", "40px")
            .set("text-anchor", "middle")
            .set("dominant-baseline", "middle")
            .add(SvgTextNode::new(text))
    };

    let svg = Document::new()
        .set("width", SKETCH_SIZE as i32)
        .set("height", SKETCH_SIZE as i32)
        .set(
            "viewBox",
            format!("0 0 {} {}", SKETCH_SIZE as i32, SKETCH_SIZE as i32),
        )
        .add(build_text(1, 2, 2, &digits));

    (0..6).fold(svg, |acc, segment| {
        let [group1, group2] = build_group(segment);

        (0..16).fold(acc.add(group1).add(group2), |acc, row| {
            (0..row + 1).fold(acc, |acc, col| {
                acc.add(build_text(segment as u8, row as u8, col as u8, &digits))
            })
        })
    })
}
