use std::path::PathBuf;

use svg::Document;
use svg::node::Text as SvgTextNode;
use svg::node::element::{Group, Line, Text as SvgText};

use glam::{Mat2, Vec2};

use printpdf::{
    Mm, Op, ParsedFont, PdfDocument, PdfPage, PdfSaveOptions, Point as PrintPdfPoint, Pt, Svg,
    TextAlign, TextShapingOptions, XObjectTransform,
};

static ROBOTO_FONT: &[u8] = include_bytes!("./Roboto-Light.ttf");

const SKETCH_SIZE: f32 = 2480.0;

pub async fn save_mandala_pdf(
    calculation: Vec<Vec<u16>>,
    input: String,
) -> Result<PathBuf, String> {
    let svg = generate_mandala_svg(&calculation);
    let pdf_bytes = generate_mandala_pdf(svg, &input)?;

    let export_path = dirs::download_dir().unwrap().join("mandala.pdf");

    std::fs::write(&export_path, pdf_bytes).map_err(|_| "Failed to write PDF file")?;

    Ok(export_path)
}

fn generate_mandala_pdf(svg: Document, text: &str) -> Result<Vec<u8>, String> {
    let mut document = PdfDocument::new("Mandala");

    let sketch_svg = Svg::parse(&svg.to_string(), &mut Vec::new())?;
    let sketch_id = document.add_xobject(&sketch_svg);
    let roboto_font = ParsedFont::from_bytes(ROBOTO_FONT, 0, &mut Vec::new())
        .ok_or("Failed to parse Roboto font")?;
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

fn generate_mandala_svg(digits: &Vec<Vec<u16>>) -> Document {
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
            .set("fill", "black")
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
