use lobsters::url::{self, Url};

pub fn as_usize((x, y): (u16, u16)) -> (usize, usize) {
    (usize::from(x), usize::from(y))
}

pub fn parse_url(src: &str) -> Result<Url, url::ParseError> {
    src.parse()
}

pub fn count_digits(num: i32) -> usize {
    match num {
        0 => 1,
        num if num.is_negative() => (f64::from(num.abs()).log10() + 1.).floor() as usize + 1,
        _ => (f64::from(num).log10() + 1.).floor() as usize,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_count_digits() {
        assert_eq!(count_digits(0), 1);
        assert_eq!(count_digits(1), 1);
        assert_eq!(count_digits(10), 2);
        assert_eq!(count_digits(50), 2);
        assert_eq!(count_digits(99), 2);
        assert_eq!(count_digits(101), 3);
        assert_eq!(count_digits(-101), 4);
        assert_eq!(count_digits(-99), 3);
        assert_eq!(count_digits(-50), 3);
        assert_eq!(count_digits(-10), 3);
        assert_eq!(count_digits(-1), 2);
    }
}
