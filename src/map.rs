use std::collections::HashSet;

use rand::prelude::*;

pub type Point = (f32, f32);
pub type RegionId = usize;

#[derive(Debug)]
pub struct Map {
    pub regions: Vec<Region>,
    pub raster: Vec<Vec<RegionId>>,
}

#[derive(Debug)]
pub struct Region {
    pub pivot: (u16, u16),
    pub boundary: Vec<Point>,
    pub neighbors: HashSet<RegionId>,
}

impl Map {
    pub fn voronoi_like(rng: &mut impl Rng, (w, h): (u8, u8), npivots: usize) -> Self {
        assert!(w > 2);
        assert!(h > 2);

        // TODO: find a better way to generate distinct pivot points
        let mut regions = (0..npivots)
            .map(|_| Region {
                pivot: {
                    let x = rng.gen_range(1, u16::from(w) - 1);
                    let y = rng.gen_range(1, u16::from(h) - 1);
                    (x, y)
                },
                boundary: vec![],
                neighbors: HashSet::new(),
            })
            .collect::<Vec<_>>();
        regions.sort_by_key(|r| r.pivot);
        regions.dedup_by_key(|r| r.pivot);

        let mut canvas = vec![vec![regions.len(); usize::from(w)]; usize::from(h)];
        let mut boundaries = Vec::with_capacity(regions.len());

        for (region_id, r) in regions.iter().enumerate() {
            let x = usize::from(r.pivot.0);
            let y = usize::from(r.pivot.1);
            boundaries.push(vec![r.pivot]);
            canvas[y][x] = region_id;
        }

        // TODO: it seems boundaries are a bit too thick in some cases, try to stick to boundaries
        // of 1pix so that it's easier to generate non piexelated boundaries in case of svg output
        // or similar
        loop {
            let mut changed = false;

            for (region_id, bs) in boundaries.iter_mut().enumerate() {
                let mut newbs = vec![];

                for p in bs.iter() {
                    let neighbors = {
                        let ox = i32::from(p.0);
                        let oy = i32::from(p.1);

                        [
                            // (ox - 1, oy - 1),
                            // (ox - 1, oy + 1),
                            // (ox + 1, oy - 1),
                            // (ox + 1, oy + 1),
                            (ox - 1, oy),
                            (ox, oy - 1),
                            (ox, oy + 1),
                            (ox + 1, oy),
                        ]
                    };

                    for &(x, y) in &neighbors {
                        if x < 0 || y < 0 || x >= i32::from(w) || y >= i32::from(h) {
                            continue;
                        }
                        let x = x as u16;
                        let y = y as u16;

                        let on_boundary =
                            x == 0 || x == u16::from(w) - 1 || y == 0 || y == u16::from(h) - 1;

                        let mut closest_rid = canvas[y as usize][x as usize];
                        if closest_rid >= regions.len() {
                            canvas[usize::from(y)][usize::from(x)] = region_id;
                            closest_rid = region_id;
                            newbs.push((x, y));
                        }

                        if closest_rid != region_id {
                            regions[region_id].neighbors.insert(closest_rid);
                            regions[closest_rid].neighbors.insert(region_id);
                            regions[region_id].boundary.push((
                                (f32::from(x) + f32::from(p.0)) / 2.0,
                                (f32::from(y) + f32::from(p.1)) / 2.0,
                            ));
                        } else if on_boundary {
                            regions[region_id]
                                .boundary
                                .push((f32::from(x), f32::from(y)));
                        }
                    }
                }

                *bs = newbs;
                if !bs.is_empty() {
                    changed = true;
                }
            }

            if !changed {
                break;
            }
        }

        for (id, r) in regions.iter_mut().enumerate() {
            r.neighbors.remove(&id);
        }

        Map {
            regions,
            raster: canvas,
        }
    }
}
