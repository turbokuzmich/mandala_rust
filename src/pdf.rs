use std::path::PathBuf;

use glam::{Mat2, Vec2};
use printpdf::*;

static ROBOTO_FONT: &[u8] = include_bytes!("./Roboto-Light.ttf");

const PAGE_WIDTH: f32 = 210.0;
const PAGE_HEIGHT: f32 = 297.0;
const SKETCH_OFFSET: f32 = 10.0;

static DIGITS: [&str; 9] = ["1", "2", "3", "4", "5", "6", "7", "8", "9"];

struct Sizes {
    segment_size: f32,
    half_segment_size: f32,
    line_offset: f32,
    half_line_offset: f32,
    quarter_segment_size: f32,
    translation_vector: Vec2,
}

pub async fn save_mandala_pdf(
    calculation: Vec<Vec<u16>>,
    input: String,
) -> Result<PathBuf, String> {
    let mut document = PdfDocument::new("Mandala");

    let segment_size = PAGE_WIDTH / 33.0;
    let half_segment_size = segment_size / 2.0;
    let line_offset = ((half_segment_size * 2.0).powf(2.0) - half_segment_size.powf(2.0)).sqrt();

    let sizes = Sizes {
        segment_size: segment_size,
        half_segment_size: segment_size / 2.0,
        line_offset: ((half_segment_size * 2.0).powf(2.0) - half_segment_size.powf(2.0)).sqrt(),
        half_line_offset: line_offset / 2.0,
        quarter_segment_size: half_segment_size / 2.0,
        translation_vector: Vec2::new(
            PAGE_WIDTH / 2.0,
            PAGE_HEIGHT - (PAGE_WIDTH / 2.0) - SKETCH_OFFSET,
        ),
    };

    let roboto_font = ParsedFont::from_bytes(ROBOTO_FONT, 0, &mut Vec::new())
        .ok_or("Failed to parse Roboto font")?;
    let roboto_font_id = document.add_font(&roboto_font);

    let mut contents = vec![
        Op::SetFillColor {
            col: printpdf::Color::Rgb(printpdf::Rgb {
                r: 0.0,
                g: 0.0,
                b: 0.0,
                icc_profile: None,
            }),
        },
        Op::SetOutlineColor {
            col: Color::Rgb(Rgb::new(0.0, 0.0, 0.0, None)),
        },
        Op::SetOutlineThickness { pt: Pt(0.5) },
    ];

    generate_lines(&mut contents, &sizes);
    generate_digits(
        calculation,
        &roboto_font,
        &roboto_font_id,
        &mut contents,
        &sizes,
    );
    generate_text(&input, &roboto_font, &roboto_font_id, &mut contents);

    let page = PdfPage::new(Mm(PAGE_WIDTH), Mm(PAGE_HEIGHT), contents);
    let pdf_bytes: Vec<u8> = document
        .with_pages(vec![page])
        .save(&PdfSaveOptions::default(), &mut Vec::new());

    let file_name = input.chars().take(100).collect::<String>();
    let export_path = dirs::download_dir().unwrap().join(&format!("Мандала {}.pdf", file_name));

    std::fs::write(&export_path, pdf_bytes).map_err(|_| "Failed to write PDF file")?;

    Ok(export_path)
}

fn generate_text(text: &str, font: &ParsedFont, font_id: &FontId, contents: &mut Vec<Op>) {
    let text_options = TextShapingOptions {
        max_width: Some(Mm(190.0).into_pt()),
        align: TextAlign::Center,
        ..TextShapingOptions::default()
    };
    let text = font.shape_text(text, &text_options, font_id);

    contents.extend(text.get_ops(Point::new(Mm(10.0), Mm(50.0))));
}

fn generate_digits(
    digits: Vec<Vec<u16>>,
    font: &ParsedFont,
    font_id: &FontId,
    contents: &mut Vec<Op>,
    sizes: &Sizes,
) {
    let text_options = TextShapingOptions {
        ..TextShapingOptions::default()
    };

    (0..6).for_each(|segment| {
        (0..16).for_each(|row| {
            (0..row + 1).for_each(|col| {
                let digit = digits[15 - row as usize][col as usize];
                let text = font.shape_text(DIGITS[digit as usize - 1], &text_options, font_id);

                let x = (col as f32) * sizes.line_offset + sizes.half_line_offset;
                let y = (row as f32) * sizes.segment_size + sizes.half_segment_size
                    - (col as f32) * sizes.half_segment_size
                    + sizes.quarter_segment_size;

                let width: Mm = Pt(text.width).into();
                let height: Mm = Pt(text.height).into();

                let bounds_vector = Vec2::new(-width.0 / 2.0, height.0 / 2.0);
                let initial_position_vector = Vec2::new(x, y);
                let rotation_matrix = Mat2::from_angle((segment as f32) * 60.0_f32.to_radians());
                let position = rotation_matrix * initial_position_vector
                    + sizes.translation_vector
                    + bounds_vector;

                contents.extend(text.get_ops(Point::new(Mm(position.x), Mm(position.y))));
            });
        });
    });
}

fn generate_lines(contents: &mut Vec<Op>, sizes: &Sizes) {
    (0..6).for_each(|sketch_segment| {
        (0..17).for_each(|line_index| {
            let x = sizes.line_offset * line_index as f32;
            let y1 = sizes.half_segment_size * (line_index as f32);
            let idx = if line_index == 0 { 0 } else { line_index - 1 };
            let count = 16 - idx;
            let y2 = y1 + (count as f32) * sizes.half_segment_size * 2.0;

            let rotation = Mat2::from_angle(sketch_segment as f32 * 60.0_f32.to_radians());

            let line_a_start = rotation * Vec2::new(x, y1) + sizes.translation_vector;
            let line_a_end = rotation * Vec2::new(x, y2) + sizes.translation_vector;
            let line_b_start = rotation * Vec2::new(-x, y1) + sizes.translation_vector;
            let line_b_end = rotation * Vec2::new(-x, y2) + sizes.translation_vector;

            contents.push(Op::DrawLine {
                line: Line {
                    points: vec![
                        LinePoint {
                            p: Point::new(Mm(line_a_start.x), Mm(line_a_start.y)),
                            bezier: false,
                        },
                        LinePoint {
                            p: Point::new(Mm(line_a_end.x), Mm(line_a_end.y)),
                            bezier: false,
                        },
                    ],
                    is_closed: true,
                },
            });

            contents.push(Op::DrawLine {
                line: Line {
                    points: vec![
                        LinePoint {
                            p: Point::new(Mm(line_b_start.x), Mm(line_b_start.y)),
                            bezier: false,
                        },
                        LinePoint {
                            p: Point::new(Mm(line_b_end.x), Mm(line_b_end.y)),
                            bezier: false,
                        },
                    ],
                    is_closed: true,
                },
            });
        });
    });
}
