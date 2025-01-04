

pub fn encode(input: u128) -> String {
    const CHARACTERS: [char; 62] = [
        '0', '1', '2', '3', '4', '5', '6', '7', '8', '9',
        'a', 'b', 'c', 'd', 'e', 'f', 'g', 'h', 'i', 'j',
        'k', 'l', 'm', 'n', 'o', 'p', 'q', 'r', 's', 't',
        'u', 'v', 'w', 'x', 'y', 'z', 'A', 'B', 'C', 'D',
        'E', 'F', 'G', 'H', 'I', 'J', 'K', 'L', 'M', 'N',
        'O', 'P', 'Q', 'R', 'S', 'T', 'U', 'V', 'W', 'X',
        'Y', 'Z'
    ];

    let mut input = input;
    let mut result = String::new();

    while input > 0 {
        let remainder = (input % 62) as usize;
        let character = CHARACTERS[remainder];
        result.push(character);
        input /= 62;
    }

    result.chars().rev().collect()
}


pub fn decode(input: &str) -> Option<u128> {
    const CHARACTERS: &str = "0123456789abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ";

    let mut result: u128 = 0;
    for (index, char) in input.chars().rev().enumerate() {
        if let Some(position) = CHARACTERS.find(char) {
            let position_value = (position as u128).checked_mul(62_u128.pow(index as u32))?;
            result = result.checked_add(position_value)?;
        } else {
            return None;
        }
    }

    Some(result)
}