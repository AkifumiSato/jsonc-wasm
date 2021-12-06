pub fn is_number_token_char(c: char) -> bool {
    c.is_numeric() | matches!(c, '.' | '-' | 'e' | 'E')
}
