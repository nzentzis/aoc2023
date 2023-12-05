use anyhow::Result;

/// A single scratch card
///
/// Numbers are always two digits at most, so they're represented as bitsets here for efficiency.
struct Card {
    winning: u128,
    numbers: u128,
}

fn load_input(input: &mut dyn std::io::BufRead) -> Result<Input> {
    crate::util::read_lines_regex(input, r#"^Card +(\d+): +((?:\d+ +)*)\|((?: +\d+)*)$"#, |caps| {
        let winning = caps.get(2).unwrap()
                     .as_str().split_whitespace()
                     .map(|n| n.parse::<usize>())
                     .try_fold(0, |acc, num| num.map(|n| acc | (1u128 << n)))?;
        let numbers = caps.get(3).unwrap()
                     .as_str().split_whitespace()
                     .map(|n| n.parse::<usize>())
                     .try_fold(0, |acc, num| num.map(|n| acc | (1u128 << n)))?;

        Ok(Card{ winning, numbers })
    })
}

impl Card {
    #[inline]
    /// How many matching numbers are on this card?
    fn count_matches(&self) -> usize {
        (self.winning & self.numbers).count_ones() as usize
    }
}

fn solve1(input: &Input) -> Result<u64> {
    Ok(input.iter()
            .map(Card::count_matches).filter(|wins| *wins > 0)
            .map(|wins| 1 << (wins - 1)) // compute value of each card
            .sum())
}

fn solve2(input: &Input) -> Result<u64> {
    // number of copies of each card
    let mut dupes = 1; // copies of the next card to make
    let mut expiring = vec![0; input.len()]; // number of cards expiring at each point

    let mut total_cards = 0; // total number of cards processed
    for (card_idx, card) in input.into_iter().enumerate() {
        dupes -= expiring[card_idx];
        total_cards += dupes;

        // handle drawing extra cards
        let n = card.count_matches();
        if n == 0 {
            continue;
        }

        let copy_end = card_idx + 1 + n; // card index at which the dupes from this card stop
        if copy_end < expiring.len() {
            expiring[copy_end] += dupes; // expire all duplicates we're about to add
        }
        dupes += dupes; // add a duplicate for each instance of the current card
    }

    Ok(total_cards)
}

problem!(load_input => Vec<Card> => (solve1, solve2));
