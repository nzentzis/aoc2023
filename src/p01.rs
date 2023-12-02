use anyhow::{anyhow, Result};

use std::iter::DoubleEndedIterator;
use std::str::FromStr;

struct Line(String);

impl FromStr for Line {
    type Err = std::convert::Infallible;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(Self(s.to_owned()))
    }
}

impl Line {
    fn digits1<'a>(&'a self) -> impl DoubleEndedIterator<Item=u8> + Clone + 'a {
        self.0.as_bytes().iter()
            .cloned().filter(u8::is_ascii_digit)
            .map(|d| d - b'0')
    }

    /// Original implementation of part 2 digit recognition
    #[allow(dead_code)]
    fn digits2(&self) -> impl DoubleEndedIterator<Item=u8> + Clone {
        const DIGITS: &[(&[u8], u8)] = &[
            (b"one", 1), (b"two", 2), (b"three", 3), (b"four", 4), (b"five", 5), (b"six", 6),
            (b"seven", 7), (b"eight", 8), (b"nine", 9),
        ];

        let mut out = Vec::new();
        let mut data = self.0.as_bytes();

        while !data.is_empty() {
            if data[0].is_ascii_digit() {
                out.push(data[0] - b'0');
                data = &data[1..];
                continue;
            }

            if !matches!(data[0], b'e'..=b'i' | b'n' | b'o' | b'r'..=b't' | b'u' ..=b'x') {
                data = &data[1..];
                continue;
            }

            if let Some((pat, num)) = DIGITS.iter().find(|(s, _)| data.starts_with(s)) {
                data = &data[pat.len() - 1..];
                out.push(*num);
                continue;
            }

            data = &data[1..];
        }

        out.into_iter()
    }

    /// Optimized digit recognizer using a finite state machine
    ///
    /// Runs in O(n) for any input. This implementation uses a hand-built DFA to recognize input
    /// bytes. Partial overlaps are handled automatically by having the last recognized character
    /// of one number jump to the corresponding start state of another.
    fn digits3(&self) -> impl DoubleEndedIterator<Item=u8> + Clone {
        const STATE_TABLE: &[[u8; 14]] = &[
        //   0  1  2  3  4  5  6  7  8  9  10 11 12 13
        //   o  e  r  x  n  t  w  h  f  u  i  v  s  g
            [1, 5, 0, 0, 6, 2, 0, 0, 3, 0, 0, 0, 4, 0], // 0
            [1, 5, 0, 0, 7, 2, 0, 0, 3, 0, 0, 0, 4, 0], // 1
            [1, 5, 0, 0, 6, 2, 8, 9, 3, 0, 0, 0, 4, 0], // 2
            [10,5, 0, 0, 6, 2, 0, 0, 3, 0, 11,0, 4, 0], // 3
            [1, 13,0, 0, 6, 2, 0, 0, 3, 0, 12,0, 4, 0], // 4
            [1, 5, 0, 0, 6, 2, 0, 0, 3, 0, 14,0, 4, 0], // 5
            [1, 5, 0, 0, 6, 2, 0, 0, 3, 0, 15,0, 4, 0], // 6
            [1, 25,0, 0, 6, 2, 0, 0, 3, 0, 15,0, 4, 0], // 7
            [26,5, 0, 0, 6, 2, 0, 0, 3, 0, 0, 0, 4, 0], // 8
            [1, 5, 16,0, 6, 2, 0, 0, 3, 0, 0, 0, 4, 0], // 9
            [1, 5, 0, 0, 7, 2, 0, 0, 3, 17,0, 0, 4, 0], // 10
            [1, 5, 0, 0, 6, 2, 0, 0, 3, 0, 0, 18,4, 0], // 11
            [1, 5, 0, 30,6, 2, 0, 0, 3, 0, 0, 0, 4, 0], // 12
            [1, 5, 0, 0, 6, 2, 0, 0, 3, 0, 14,19,4, 0], // 13
            [1, 5, 0, 0, 6, 2, 0, 0, 3, 0, 0, 0, 4, 20],// 14
            [1, 5, 0, 0, 21,2, 0, 0, 3, 0, 0, 0, 4, 0], // 15
            [1, 22,0, 0, 6, 2, 0, 0, 3, 0, 0, 0, 4, 0], // 16
            [1, 5, 28,0, 6, 2, 0, 0, 3, 0, 0, 0, 4, 0], // 17
            [1, 29,0, 0, 6, 2, 0, 0, 3, 0, 0, 0, 4, 0], // 18
            [1, 23,0, 0, 6, 2, 0, 0, 3, 0, 0, 0, 4, 0], // 19
            [1, 5, 0, 0, 6, 2, 0, 24,3, 0, 0, 0, 4, 0], // 20
            [1, 33,0, 0, 6, 2, 0, 0, 3, 0, 15,0, 4, 0], // 21
            [1, 27,0, 0, 6, 2, 0, 0, 3, 0, 14,0, 4, 0], // 22
            [1, 5, 0, 0, 31,2, 0, 0, 3, 0, 14,0, 4, 0], // 23
            [1, 5, 0, 0, 6, 32,0, 0, 3, 0, 0, 0, 4, 0], // 24
        ];

        // fake emit states are 25-33
        const EMIT_START: u8 = 25;
        const EMIT_STATES_NEXT: &[u8] = &[5, 1, 5, 0, 5, 0, 6, 2, 5];

                                  //zyxwvutsrqponmlkjihgfedcba
        const CHARSET_MASK: u32 = 0b00111111100110000111110000;
        const TOKEN_TABLE: [u8; 26] = [
        //  a  b  c  d  e  f  g  h  i  j  k  l  m  n  o  p  q  r  s  t  u  v  w  x  y  z
            0, 0, 0, 0, 1, 8, 13,7, 10,0, 0, 0, 0, 4, 0, 0, 0, 2, 12,5, 9, 11,6, 3, 0, 0
        ];

        let mut out = Vec::new();
        let mut state = 0;
        for c in self.0.as_bytes() {
            if c.is_ascii_digit() {
                out.push(c - b'0');
                state = 0;
                continue;
            } else if !c.is_ascii_lowercase() {
                state = 0;
                continue;
            }

            let char_idx = (c - b'a') as usize;
            if (1 << char_idx) & CHARSET_MASK == 0 {
                state = 0;
                continue;
            }
            let token = TOKEN_TABLE[char_idx] as usize;
            let next = STATE_TABLE[state][token] as usize;

            if next >= EMIT_START as usize {
                let next = next as u8;
                out.push(next - EMIT_START + 1);
                state = EMIT_STATES_NEXT[(next - EMIT_START) as usize] as usize;
            } else {
                state = next;
            }
        }

        out.into_iter()
    }
}

fn calibration<I: DoubleEndedIterator<Item=u8> + Clone>(iter: I) -> Option<u64> {
    let d0 = iter.clone().next()? as u64;
    let d1 = iter.rev().next()? as u64;

    Some((d0 * 10) + d1)
}

fn solve1(lines: &Input) -> Result<u64> {
    lines.iter()
         .map(|l| calibration(l.digits1()).ok_or_else(|| anyhow!("invalid input line")))
         .sum()
}

fn solve2(lines: &Input) -> Result<u64> {
    lines.iter()
         .map(|l| calibration(l.digits3()).ok_or_else(|| anyhow!("invalid input line")))
         .sum()
}

problem!(crate::util::load_lines => Vec<Line> => (solve1, solve2));
