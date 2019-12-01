use crate::map::{Map, RegionId};

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ColorMap {
    // possible colors by regionid. A color is represented as a 4 bit bitmask and each element
    // contains the possible colors for 2 regions.
    possible_colors: Vec<u8>,

    // number of regions of this colormap.
    regions: usize,
}

#[derive(Debug, Clone, Copy)]
#[repr(u8)]
pub enum Color {
    C1 = 1,
    C2 = 1 << 1,
    C3 = 1 << 2,
    C4 = 1 << 3,
}

enum SolutionState {
    CannotSolve,
    Solved,
    Unknown,
}

impl ColorMap {
    pub fn color(m: &Map) -> Option<ColorMap> {
        Self::all_possible_colorings(m).next()
    }

    pub fn all_possible_colorings<'s>(m: &'s Map) -> impl Iterator<Item = ColorMap> + 's {
        SolutionIter::new(m)
    }

    pub fn color_of_region(&self, rid: RegionId) -> Color {
        let c = self.at(rid);

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

    fn at(&self, rid: RegionId) -> u8 {
        assert!(rid < self.regions);

        let pcs = self.possible_colors[rid / 2];
        if rid & 1 != 0 {
            pcs >> 4
        } else {
            pcs & 0xf
        }
    }

    fn set(&mut self, rid: RegionId, v: u8) {
        assert!(rid < self.regions);

        let old = self.possible_colors[rid / 2];
        let pcs = if rid & 1 != 0 {
            (v << 4) | (old & 0xf)
        } else {
            (old & 0xf0) | (v & 0xf)
        };

        self.possible_colors[rid / 2] = pcs;
    }

    fn remove_conflicts(&mut self, map: &Map) -> SolutionState {
        loop {
            // first find regions with a single possible colors and remove that color from its
            // neighbors until no regions change its respective colors. If any of the regions cannot be
            // colored then this map cannot be colored. On the other hand, if all the regions have a
            // single possible color then that's the solution.
            let mut stalled = true;
            let mut solved = true;

            for rid in 0..self.regions {
                let c = self.at(rid);
                if c == 0 {
                    return SolutionState::CannotSolve;
                }

                if c.count_ones() != 1 {
                    solved = false;
                    continue;
                }

                for &neigh in &map.regions[rid].neighbors {
                    let old = self.at(neigh);
                    let new = old & !c;

                    self.set(neigh, new);
                    if old != new {
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
}

#[derive(Debug)]
struct SolutionIter<'m> {
    stack: Vec<ColorMap>,
    map: &'m Map,
}

impl<'m> SolutionIter<'m> {
    fn new(map: &'m Map) -> Self {
        let cm = vec![0xff; map.regions.len() / 2 + (map.regions.len() & 1)];

        SolutionIter {
            map,
            stack: vec![ColorMap {
                possible_colors: cm,
                regions: map.regions.len(),
            }],
        }
    }
}

impl Iterator for SolutionIter<'_> {
    type Item = ColorMap;

    fn next(&mut self) -> Option<Self::Item> {
        let has_color = |pc: u8, cid: Color| (pc & cid as u8) != 0;
        let possible_colors_len =
            |pc: u8| ((pc >> 3) & 1) + ((pc >> 2) & 1) + ((pc >> 1) & 1) + (pc & 1);

        while let Some(mut color_map) = self.stack.pop() {
            let state = color_map.remove_conflicts(self.map);

            match state {
                SolutionState::Solved => return Some(color_map),
                SolutionState::CannotSolve => continue,
                SolutionState::Unknown => {
                    // pick the region with the smallest amount of possible colors to choose from so that we
                    // have to explore less space
                    let (candidate_rid, _) = (0..color_map.regions)
                        .map(|rid| (rid, possible_colors_len(color_map.at(rid))))
                        .filter(|(_, pcl)| {
                            // regions that have a single possible color are fixed and cannot be
                            // changed, aka they're not candidates
                            *pcl != 1
                        })
                        .min_by_key(|(_, pcl)| *pcl)?;

                    // try all the possible colors for the candidate and recursively find a solution
                    let pcs = color_map.at(candidate_rid);
                    self.stack.extend(
                        [Color::C1, Color::C2, Color::C3, Color::C4]
                            .iter()
                            .rev()
                            .filter(|&&c| has_color(pcs, c))
                            .map(|&c| {
                                let mut new_color_map = color_map.clone();
                                new_color_map.set(candidate_rid, c as u8);
                                new_color_map
                            }),
                    );
                }
            }
        }

        None
    }
}
