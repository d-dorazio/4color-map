use crate::map::RegionId;
use crate::Map;

#[derive(Debug)]
pub struct ColorMap {
    // possible colors by regionid. A color is represented as a 4 bit bitmask.
    possible_colors: Vec<u8>,
}

#[derive(Debug, Clone, Copy)]
#[repr(u8)]
pub enum Color {
    C1 = 1,
    C2 = 1 << 1,
    C3 = 1 << 2,
    C4 = 1 << 3,
}

impl ColorMap {
    pub fn color(m: &Map) -> Option<ColorMap> {
        Self::all_possible_colorings(m).next()
    }

    pub fn all_possible_colorings<'s>(m: &'s Map) -> impl Iterator<Item = ColorMap> + 's {
        SolutionIter::new(m)
    }

    pub fn color_of_region(&self, rid: RegionId) -> Color {
        let c = self.possible_colors[rid];
        if c == Color::C1 as u8 {
            return Color::C1;
        }
        if c == Color::C2 as u8 {
            return Color::C2;
        }
        if c == Color::C3 as u8 {
            return Color::C3;
        }
        if c == Color::C4 as u8 {
            return Color::C4;
        }

        if cfg!(debug_assertions) {
            unreachable!()
        } else {
            // might want to use the unchecked version for greater speed ups
            // use std::hint::unreachable_unchecked;
            // unsafe { unreachable_unchecked() }

            unreachable!()
        }
    }
}

#[derive(Debug)]
struct SolutionIter<'m> {
    stack: Vec<Vec<u8>>,
    map: &'m Map,
}

impl<'m> SolutionIter<'m> {
    fn new(map: &'m Map) -> Self {
        SolutionIter {
            map,
            stack: vec![vec![0xf; map.regions.len()]],
        }
    }
}

enum SolutionState {
    CannotSolve,
    Solved,
    Unknown,
}

impl Iterator for SolutionIter<'_> {
    type Item = ColorMap;

    fn next(&mut self) -> Option<Self::Item> {
        let has_color = |pc: u8, cid: Color| (pc & cid as u8) != 0;
        let possible_colors_len =
            |pc: u8| ((pc >> 3) & 1) + ((pc >> 2) & 1) + ((pc >> 1) & 1) + (pc & 1);

        while let Some(mut possible_colors) = self.stack.pop() {
            let state = remove_conflicts(&mut possible_colors, &self.map);

            match state {
                SolutionState::Solved => return Some(ColorMap { possible_colors }),
                SolutionState::CannotSolve => continue,
                SolutionState::Unknown => {
                    // pick the region with the smallest amount of possible colors to choose from so that we
                    // have to explore less space
                    let (candidate_i, _) = possible_colors
                        .iter()
                        .enumerate()
                        .map(|(i, &pc)| (i, possible_colors_len(pc)))
                        .filter(|(_, pcl)| {
                            // regions that have a single possible color are fixed and cannot be
                            // changed, aka they're not candidates
                            *pcl != 1
                        })
                        .min_by_key(|(_, pcl)| *pcl)?;

                    // try all the possible colors for the candidate and recursively find a solution
                    let pcs = possible_colors[candidate_i];
                    self.stack.extend(
                        [Color::C1, Color::C2, Color::C3, Color::C4]
                            .iter()
                            .rev()
                            .filter(|&&c| has_color(pcs, c))
                            .map(|&c| {
                                let mut new_possible_colors = possible_colors.clone();
                                new_possible_colors[candidate_i] = c as u8;
                                new_possible_colors
                            }),
                    );
                }
            }
        }

        None
    }
}

fn remove_conflicts(possible_colors: &mut [u8], map: &Map) -> SolutionState {
    loop {
        // first find regions with a single possible colors and remove that color from its
        // neighbors until no regions change its respective colors. If any of the regions cannot be
        // colored then this map cannot be colored. On the other hand, if all the regions have a
        // single possible color then that's the solution.
        let mut stalled = true;
        let mut solved = true;

        for rid in 0..possible_colors.len() {
            let c = possible_colors[rid];
            if c == 0 {
                return SolutionState::CannotSolve;
            }

            if c.count_ones() != 1 {
                solved = false;
                continue;
            }

            for &neigh in &map.regions[rid].neighbors {
                let old = possible_colors[neigh];
                possible_colors[neigh] &= !c;

                if old != possible_colors[neigh] {
                    stalled = false;
                    solved = false;
                }
            }
        }

        if stalled {
            break if solved {
                SolutionState::Solved
            } else {
                SolutionState::Unknown
            };
        }
    }
}
