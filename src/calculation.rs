use std::collections::HashMap;
use std::sync::LazyLock;

// The source string containing UTF-8 symbols
const CYR_SYMBOL_STRING: &str = "абвгдеёжзийклмнопрстуфхцчшщъыьэюя";
const LAT_SYMBOL_STRING: &str = "abcdefghijklmnopqrstuvwxyz";

fn index_generator() -> impl Iterator<Item = u16> {
    (1..=9).cycle()
}

// Static immutable map of UTF-8 symbols to u16 values
static SYMBOL_MAP: LazyLock<HashMap<char, u16>> = LazyLock::new(|| {
    let cyr_symbols = CYR_SYMBOL_STRING.chars().zip(index_generator());
    let lat_symbols = LAT_SYMBOL_STRING.chars().zip(index_generator());

    cyr_symbols.chain(lat_symbols).collect()
});

fn get_sum(index_a: u16, index_b: u16) -> u16 {
    let sum = index_a + index_b;
    if sum > 9 { sum - 9 } else { sum }
}

pub fn calculate_mandala(text: &str) -> Result<(), String> {
    let mut indexes: Vec<u16> = text.chars().fold(vec![], |mut acc, letter| {
        let lower_letter = letter.to_lowercase().next().unwrap();
        let index = SYMBOL_MAP.get(&lower_letter);
        if let Some(&index) = index {
            acc.push(index);
        }
        acc
    });

    if indexes.len() < 2 {
        indexes.push(0);
        indexes.push(0);
    }

    while indexes.len() < 8 {
        let last_index = indexes.last().unwrap();
        let second_last_index = indexes.get(indexes.len() - 2).unwrap();
        let sum = get_sum(*last_index, *second_last_index);
        indexes.push(sum);
    }

    let mut result: Vec<Vec<u16>> = Vec::with_capacity(indexes.len());
    let size = indexes.len();

    result.push(indexes);

    for index in 1..size {
        let previous = result.get(index - 1).unwrap();

        let mut row: Vec<u16> = Vec::with_capacity(previous.len() - 1);

        let row_iter = previous
            .iter()
            .zip(previous.iter().skip(1))
            .map(|(a, b)| get_sum(*a, *b));

        row.extend(row_iter);

        result.push(row);
    }

    print!("{:?}", result);

    Ok(())
}
