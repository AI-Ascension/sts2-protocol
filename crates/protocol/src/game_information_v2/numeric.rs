// SPDX-License-Identifier: MIT

use super::{Rejection, Result};

const MAX_SAFE: u64 = 9_007_199_254_740_991;

/// Preserve string bytes and normalize exact mathematical integers before any float parser.
/// Original input is bounded by decode; each normalized number has at most 17 bytes.
pub(super) fn normalize(input: &[u8]) -> Result<Vec<u8>> {
    std::str::from_utf8(input).map_err(|_| Rejection::Malformed)?;
    let mut output = Vec::with_capacity(input.len());
    let mut at = 0;
    while at < input.len() {
        let start = at;
        if input[at] == b'"' {
            at = string_end(input, at + 1)?;
            output.extend_from_slice(&input[start..at]);
        } else if input[at] == b'-' || input[at].is_ascii_digit() {
            at += 1;
            while at < input.len() && (input[at].is_ascii_digit() || b".eE+-".contains(&input[at]))
            {
                at += 1;
            }
            output.extend_from_slice(integer(&input[start..at])?.as_bytes());
        } else {
            output.push(input[at]);
            at += 1;
        }
    }
    Ok(output)
}

fn string_end(input: &[u8], mut at: usize) -> Result<usize> {
    while at < input.len() {
        match input[at] {
            b'"' => return Ok(at + 1),
            b'\\' => at += 2,
            _ => at += 1,
        }
    }
    Err(Rejection::Malformed)
}

struct Decimal<'a> {
    negative: bool,
    coefficient: &'a [u8],
    fraction_digits: usize,
    exponent: isize,
}

fn decimal(token: &[u8]) -> Result<Decimal<'_>> {
    let negative = token.first() == Some(&b'-');
    let start = usize::from(negative);
    let mut at = start;
    match token.get(at) {
        Some(b'0') => at += 1,
        Some(b'1'..=b'9') => {
            while token.get(at).is_some_and(u8::is_ascii_digit) {
                at += 1;
            }
        }
        _ => return Err(Rejection::Malformed),
    }
    let mut fraction_digits = 0;
    if token.get(at) == Some(&b'.') {
        at += 1;
        let fraction_start = at;
        while token.get(at).is_some_and(u8::is_ascii_digit) {
            at += 1;
        }
        fraction_digits = at - fraction_start;
        if fraction_digits == 0 {
            return Err(Rejection::Malformed);
        }
    }
    let coefficient = &token[start..at];
    let exponent = exponent(token, &mut at)?;
    if at != token.len() {
        return Err(Rejection::Malformed);
    }
    Ok(Decimal {
        negative,
        coefficient,
        fraction_digits,
        exponent,
    })
}

fn exponent(token: &[u8], at: &mut usize) -> Result<isize> {
    if !matches!(token.get(*at), Some(b'e' | b'E')) {
        return Ok(0);
    }
    *at += 1;
    let negative = token.get(*at) == Some(&b'-');
    if matches!(token.get(*at), Some(b'+' | b'-')) {
        *at += 1;
    }
    let start = *at;
    // A nonzero coefficient cannot offset an exponent larger than its entire token length.
    // Saturation is used only for rejecting that impossible range, never to accept a value.
    let bound = token.len() + 17;
    let mut magnitude = 0;
    while let Some(digit) = token.get(*at).filter(|digit| digit.is_ascii_digit()) {
        magnitude = (magnitude * 10 + usize::from(*digit - b'0')).min(bound);
        *at += 1;
    }
    if *at == start {
        return Err(Rejection::Malformed);
    }
    let magnitude = magnitude as isize;
    Ok(if negative { -magnitude } else { magnitude })
}

fn integer(token: &[u8]) -> Result<String> {
    let decimal = decimal(token)?;
    let mut leading = 0;
    let mut trailing = 0;
    let mut digits = 0;
    let mut nonzero = false;
    for digit in decimal
        .coefficient
        .iter()
        .filter(|digit| digit.is_ascii_digit())
    {
        digits += 1;
        if !nonzero && *digit == b'0' {
            leading += 1;
        } else {
            nonzero = true;
            trailing = if *digit == b'0' { trailing + 1 } else { 0 };
        }
    }
    if !nonzero {
        return Ok("0".to_owned());
    }
    let shift = decimal.exponent - decimal.fraction_digits as isize;
    let significant = digits - leading;
    let width = significant as isize + shift;
    let dropped = if shift < 0 { (-shift) as usize } else { 0 };
    if !(1..=16).contains(&width) || dropped > trailing {
        return Err(Rejection::Malformed);
    }
    let mut magnitude = 0_u64;
    for digit in decimal
        .coefficient
        .iter()
        .filter(|digit| digit.is_ascii_digit())
        .skip(leading)
        .take(significant - dropped)
    {
        magnitude = magnitude
            .checked_mul(10)
            .and_then(|value| value.checked_add(u64::from(*digit - b'0')))
            .ok_or(Rejection::Malformed)?;
    }
    for _ in 0..shift.max(0) {
        magnitude = magnitude.checked_mul(10).ok_or(Rejection::Malformed)?;
    }
    if magnitude > MAX_SAFE {
        return Err(Rejection::Malformed);
    }
    Ok(if decimal.negative {
        format!("-{magnitude}")
    } else {
        magnitude.to_string()
    })
}

#[cfg(test)]
mod tests {
    use super::normalize;

    #[test]
    fn exact_integer_lexemes_preserve_bounds_and_string_bytes() {
        for (raw, expected) in [
            ("1.0", "1"),
            ("1e0", "1"),
            ("100e-2", "1"),
            ("9007199254740991.0", "9007199254740991"),
            ("90071992547409910e-1", "9007199254740991"),
            ("-2147483648.0", "-2147483648"),
            ("2147483647e0", "2147483647"),
            ("-0e999999999999999999999", "0"),
            (r#""1.0 and 1e0""#, r#""1.0 and 1e0""#),
        ] {
            assert_eq!(
                normalize(raw.as_bytes()).unwrap(),
                expected.as_bytes(),
                "{raw}"
            );
        }
        for raw in [
            "1.5",
            "1.0000000000000001",
            "9007199254740991.1",
            "9007199254740992.0",
            "1e-99999",
            "1e99999",
            "01",
            "1.",
            "1e+",
            "--1",
        ] {
            assert!(normalize(raw.as_bytes()).is_err(), "{raw}");
        }
    }
}
