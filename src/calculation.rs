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

pub fn calculate_mandala(text: &str) -> Result<Vec<Vec<u16>>, String> {
    let mut indexes_a = text
        .chars()
        .map(|letter| SYMBOL_MAP.get(&letter.to_lowercase().next().unwrap()))
        .filter(|index| index.is_some())
        .map(|index| *(index.unwrap()))
        .collect::<Vec<u16>>();

    if indexes_a.len() < 2 {
        return Err("Слишком короткий текст".to_string());
    }

    while indexes_a.len() < 8 {
        let last_index = indexes_a.last().unwrap();
        let second_last_index = indexes_a.get(indexes_a.len() - 2).unwrap();
        let sum = get_sum(*last_index, *second_last_index);
        indexes_a.push(sum);
    }

    let mut size = indexes_a.len();
    let mut iteration: u16 = 0;
    let mut indexes_b: Vec<u16> = vec![0; size];

    while size > 8 {
        let (from, to) = if iteration % 2 == 0 {
            (&mut indexes_a, &mut indexes_b)
        } else {
            (&mut indexes_b, &mut indexes_a)
        };

        for i in 1..size {
            to[i - 1] = get_sum(from[i - 1], from[i]);
        }

        size -= 1;
        iteration += 1;
    }

    let reduced = if iteration % 2 == 0 {
        &indexes_a
    } else {
        &indexes_b
    };

    let line = reduced
        .iter()
        .take(8)
        .chain(reduced.iter().take(8).rev())
        .map(|&index| index)
        .collect::<Vec<u16>>();

    let mut result: Vec<Vec<u16>> = Vec::with_capacity(16);

    result.push(line);

    for index in 1..16 {
        let previous = result.get(index - 1).unwrap();

        let mut row: Vec<u16> = Vec::with_capacity(previous.len() - 1);

        let row_iter = previous
            .iter()
            .zip(previous.iter().skip(1))
            .map(|(a, b)| get_sum(*a, *b));

        row.extend(row_iter);

        result.push(row);
    }

    Ok(result)
}

fn calculate_mandala_optimized(text: &str) -> Result<(), String> {
    // let mut arr: [u8; 8] = [0; 8];

    // for i in 0..9 {
    //     arr[i] = i as u8;
    // }

    Ok(())
}
