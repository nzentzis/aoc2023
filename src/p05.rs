use anyhow::{Result, Context};

macro_rules! newtypes {
    { $($i:ident),* } => {
        $(
        #[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord)]
        #[repr(transparent)]
        struct $i(u64);

        impl From<u64> for $i {
            fn from(x: u64) -> Self { Self(x) }
        }
        impl From<$i> for u64 {
            fn from(x: $i) -> u64 { x.0 }
        }
        impl std::ops::Add<u64> for $i {
            type Output = $i;

            fn add(self, other: u64) -> Self {
                Self(self.0 + other)
            }
        }
        impl std::ops::Add<$i> for u64 {
            type Output = $i;

            fn add(self, other: $i) -> $i {
                $i(self + other.0)
            }
        }
        impl Mappable for $i {
            fn offset(&self, u: u64) -> Self {
                Self(self.0 + u)
            }

            fn offset_mut(&mut self, u: u64) {
                self.0 += u;
            }

            fn dist_after(&self, other: &Self) -> u64 {
                self.0 - other.0
            }
        }
        )*
    };
}

trait Mappable: std::fmt::Debug + PartialEq + Eq + PartialOrd + Ord + Copy + From<u64> + Into<u64> {
    fn offset(&self, u: u64) -> Self;
    fn offset_mut(&mut self, u: u64);
    fn dist_after(&self, other: &Self) -> u64;

    fn direct<D: Mappable>(&self) -> D {
        let x: u64 = (*self).into();
        D::from(x)
    }
}

newtypes! { Seed, Soil, Fertilizer, Water, Light, Temp, Humidity, Location }

struct MapEntry<S, D> {
    /// Source start value
    src: S,

    /// Destination start value
    dst: D,

    /// Total number of range entries
    len: u64,
}

impl<S: Mappable, D: Mappable> MapEntry<S, D> {
    fn max_src(&self) -> S {
        let n: u64 = self.src.into();
        (n + self.len).into()
    }
}

struct Map<S: Mappable, D: Mappable> {
    /// Source-ordered list
    ranges: Vec<MapEntry<S, D>>,
}

struct Problem {
    seeds: Vec<Seed>,
    seed_soil: Map<Seed, Soil>,
    soil_fert: Map<Soil, Fertilizer>,
    fert_water: Map<Fertilizer, Water>,
    water_light: Map<Water, Light>,
    light_temp: Map<Light, Temp>,
    temp_humid: Map<Temp, Humidity>,
    humid_loc: Map<Humidity, Location>,
}

fn load_input(input: &mut dyn std::io::BufRead) -> Result<Input> {
    use std::io::{self, BufRead};

    fn parse_map<S: Mappable, D: Mappable>(lines: &[String]) -> Result<Map<S, D>> {
        anyhow::ensure!(!lines.is_empty(), "Invalid map format");
        anyhow::ensure!(lines[0].ends_with(" map:"), "Invalid map format");

        let mut entries = lines[1..].iter()
                         .map(|line| {
                             let mut ns = line.split(' ').map(|s| s.parse::<u64>());
                             let d_start = ns.next().context("Missing line part")??;
                             let s_start = ns.next().context("Missing line part")??;
                             let length = ns.next().context("Missing line part")??;
                             anyhow::ensure!(ns.next().is_none(), "Invalid line syntax");

                             Ok(MapEntry::<S, D> {
                                 src: s_start.into(),
                                 dst: d_start.into(),
                                 len: length,
                             })
                         })
                         .collect::<Result<Vec<_>>>()?;
        entries.sort_unstable_by_key(|e| e.src);

        Ok(Map { ranges: entries })
    }

    let lines = input.lines().collect::<io::Result<Vec<String>>>()?;
    let mut parts = lines.split(|l| l.is_empty());

    let seeds = parts.next().context("Missing seeds")?;
    let seeds = seeds.first().context("Missing seed line")?
               .split_once(' ').context("Missing seed separator")?.1
               .split_whitespace()
               .map(|s| Ok(s.parse::<u64>()?.into()))
               .collect::<Result<_>>()?;

    Ok(Problem {
        seeds,
        seed_soil: parts.next().context("Missing required map").and_then(parse_map)?,
        soil_fert: parts.next().context("Missing required map").and_then(parse_map)?,
        fert_water: parts.next().context("Missing required map").and_then(parse_map)?,
        water_light: parts.next().context("Missing required map").and_then(parse_map)?,
        light_temp: parts.next().context("Missing required map").and_then(parse_map)?,
        temp_humid: parts.next().context("Missing required map").and_then(parse_map)?,
        humid_loc: parts.next().context("Missing required map").and_then(parse_map)?,
    })
}

impl<S: Mappable, D: Mappable> Map<S, D> {
    /// Given a value in the source domain, map it to the destination domain
    fn map_one(&self, x: S) -> D {
        match self.ranges.binary_search_by_key(&x, |r| r.src) {
            Ok(idx) => {
                // easy case
                self.ranges[idx].dst
            }
            Err(0) => { // smaller than first range
                let s_u64: u64 = x.into();
                s_u64.into()
            },
            Err(idx) => {
                let range = &self.ranges[idx-1];
                let delta = x.into() - range.src.into();
                if delta < range.len {
                    let s_u64: u64 = range.dst.into();
                    (s_u64 + delta).into()
                } else {
                    let s_u64: u64 = x.into();
                    s_u64.into()
                }
            }
        }
    }

    /// Given a start index and range length in the source domain, iterate over the resulting
    /// mapped ranges.
    fn map_range<'a>(&'a self, range: (S, u64)) -> impl Iterator<Item=(D, u64)> + 'a {
        use std::mem::take;

        // first relevant range index
        let start_idx = match self.ranges.binary_search_by_key(&range.0, |r| r.src) {
            Err(0) => 0,
            Ok(idx) => idx,
            Err(idx) => idx-1,
        };

        let mut current_src = range.0;
        let mut remaining = range.1;

        let mut current_idx = start_idx; // current map range index
        let ranges = &self.ranges;
        std::iter::from_fn(move || {
            if remaining == 0 {
                return None;
            } else if current_idx == ranges.len() {
                return Some((current_src.direct(), take(&mut remaining)));
            }

            let range = &ranges[current_idx];

            // space from current cursor to range start
            if range.src > current_src { // emit range before we hit the mapping
                let out = (current_src.direct(),
                           range.src.dist_after(&current_src)
                                    .min(remaining));
                remaining -= out.1;
                current_src.offset_mut(out.1);
                Some(out)
            } else if current_src >= range.max_src() { // emit range after mapping
                // are there any more ranges we might run into?
                if let Some(next) = ranges.get(current_idx + 1) {
                    if next.src >= current_src.offset(remaining) {
                        Some((current_src.direct(), take(&mut remaining)))
                    } else {
                        // we're going to hit the next range at some point
                        let out = (current_src.direct(),
                                   next.src.dist_after(&current_src).min(remaining));
                        remaining -= out.1;
                        current_src.offset_mut(out.1);
                        current_idx += 1;
                        Some(out)
                    }
                } else {
                    Some((current_src.direct(), take(&mut remaining)))
                }
            } else {
                assert!(current_src >= range.src);
                // some part at the start of the remaining source range is being mapped by the
                // current mapping range
                let offset = current_src.dist_after(&range.src); // how far into the source are we?
                let n = remaining.min(range.len - offset); // how large is the emitted range?
                current_src.offset_mut(n);
                remaining -= n;
                current_idx += 1;
                Some((range.dst.offset(offset), n))
            }
        })
    }
}

fn solve1(input: &Problem) -> Result<u64> {
    let loc_nums = input.seeds.iter().cloned()
                  .map(|x| input.seed_soil.map_one(x))
                  .map(|x| input.soil_fert.map_one(x))
                  .map(|x| input.fert_water.map_one(x))
                  .map(|x| input.water_light.map_one(x))
                  .map(|x| input.light_temp.map_one(x))
                  .map(|x| input.temp_humid.map_one(x))
                  .map(|x| input.humid_loc.map_one(x));
    Ok(loc_nums.min().context("No input")?.into())
}

fn solve2(input: &Problem) -> Result<u64> {
    anyhow::ensure!(input.seeds.len() % 2 == 0);

    let out = input.seeds.chunks_exact(2)
             .map(|chunk| (chunk[0], chunk[1].0))
             .flat_map(|x| input.seed_soil.map_range(x).collect::<Vec<_>>())
             .flat_map(|x| input.soil_fert.map_range(x).collect::<Vec<_>>())
             .flat_map(|x| input.fert_water.map_range(x).collect::<Vec<_>>())
             .flat_map(|x| input.water_light.map_range(x).collect::<Vec<_>>())
             .flat_map(|x| input.light_temp.map_range(x).collect::<Vec<_>>())
             .flat_map(|x| input.temp_humid.map_range(x).collect::<Vec<_>>())
             .flat_map(|x| input.humid_loc.map_range(x).collect::<Vec<_>>())
             .map(|span| span.0)
             .min();

    out.map(|x| x.into()).context("No ranges")
}

problem!(load_input => Problem => (solve1, solve2));
