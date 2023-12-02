use anyhow::{anyhow, Result};

type Turn = [u32; 3];

#[derive(Debug)]
struct Game {
    id: u32,
    records: Vec<Turn>,
}

fn load_input(input: &mut dyn std::io::BufRead) -> Result<Input> {
    crate::util::read_lines(input, |line| {
        let (head, tail) = line.split_once(':').ok_or_else(|| anyhow!("Input missing colon"))?;
        let (_, game) = head.split_once(' ').ok_or_else(|| anyhow!("Input missing game ID"))?;
        let game = game.parse()?;

        let mut out = Vec::new();
        for part in tail.split(';').map(|p| p.trim()) {
            let mut turn = [0; 3];
            for subpart in part.split(',').map(|p| p.trim()) {
                let (n, t) = subpart.split_once(' ')
                            .ok_or_else(|| anyhow!("Input missing turn information"))?;
                let n = n.parse::<u32>()?;

                match t {
                    "red" => { turn[0] += n; }
                    "green" => { turn[1] += n; }
                    "blue" => { turn[2] += n; }
                    _ => { anyhow::bail!("Invalid entry type"); }
                }
            }
            out.push(turn);
        }

        Ok(Game { id: game, records: out })
    })
}

impl Game {
    fn plausible_for_start(&self, start: Turn) -> bool {
        self.records.iter().all(|turn| {
            turn[0] <= start[0] && turn[1] <= start[1] && turn[2] <= start[2]
        })
    }

    fn min_cubes(&self) -> Turn {
        let mut t = [0; 3];
        for x in &self.records {
            t[0] = t[0].max(x[0]);
            t[1] = t[1].max(x[1]);
            t[2] = t[2].max(x[2]);
        }
        t
    }
}

fn solve1(lines: &Input) -> Result<u64> {
    Ok(lines.iter()
      .filter(|g| g.plausible_for_start([12, 13, 14]))
      .map(|g| g.id)
      .sum::<u32>() as u64)
}

fn solve2(lines: &Input) -> Result<u64> {
    Ok(lines.iter()
      .map(|g| g.min_cubes().into_iter().product::<u32>())
      .sum::<u32>() as u64)
}

problem!(load_input => Vec<Game> => (solve1, solve2));
