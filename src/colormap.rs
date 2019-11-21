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
    pub fn color(m: &Map) -> Option<Self> {
        Some(ColorMap {
            possible_colors: Self::color_constrained(m, vec![0xf; m.regions.len()])?,
        })
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

    fn color_constrained(map: &Map, mut possible_colors: Vec<u8>) -> Option<Vec<u8>> {
        // first find regions with a single possible colors and remove that color from its
        // neighbors until no regions change its respective colors. If any of the regions cannot be
        // colored then this map cannot be colored. On the other hand, if all the regions have a
        // single possible color then that's the solution.
        loop {
            let mut stalled = true;
            let mut solved = true;

            for rid in 0..possible_colors.len() {
                let c = possible_colors[rid];
                if c == 0 {
                    return None;
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

            if solved {
                return Some(possible_colors);
            }

            if stalled {
                break;
            }
        }

        // pick the region with the smallest amount of possible colors to choose from so that we
        // have to explore less space
        let (candidate_i, _) = possible_colors
            .iter()
            .enumerate()
            .map(|(i, &pc)| {
                (
                    i,
                    ((pc >> 3) & 1) + ((pc >> 2) & 1) + ((pc >> 1) & 1) + (pc & 1),
                )
            })
            .filter(|(_, pcl)| *pcl != 1)
            .min_by_key(|(_, pcl)| *pcl)?;

        // try all the possible colors for the candidate and recursively find a solution
        let pcs = possible_colors[candidate_i];
        for i in 0..4 {
            if (pcs >> i) & 1 == 0 {
                continue;
            }

            let mut new_possible_colors = possible_colors.clone();
            new_possible_colors[candidate_i] = 1 << i;

            let sol = Self::color_constrained(map, new_possible_colors);
            if sol.is_some() {
                return sol;
            }
        }

        None
    }
}
