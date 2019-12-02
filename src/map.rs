use std::collections::{BTreeSet, HashSet};

pub type Point = (u16, u16);
pub type RegionId = usize;

#[derive(Debug)]
pub struct Map {
    pub regions: Vec<Region>,
    pub raster: Vec<Vec<RegionId>>,
}

#[derive(Debug)]
pub struct Region {
    /// seed point of the region
    pub pivot: Point,

    /// the farthest points of the region that are still part of the region
    pub boundary: HashSet<Point>,

    /// all the regions that share a border this region
    pub neighbors: HashSet<RegionId>,
}

impl Map {
    /// generate a new voronoi like `Map` using the given `pivots` points as the seeds.
    pub fn voronoi_like(pivots: HashSet<Point>, (w, h): (u16, u16)) -> Self {
        let mut regions = pivots
            .into_iter()
            .map(|pivot| Region {
                pivot,
                boundary: HashSet::new(),
                neighbors: HashSet::new(),
            })
            .collect::<Vec<_>>();

        let mut canvas = vec![vec![regions.len(); usize::from(w)]; usize::from(h)];
        let mut boundaries = Vec::with_capacity(regions.len());

        for (region_id, r) in regions.iter().enumerate() {
            let x = usize::from(r.pivot.0);
            let y = usize::from(r.pivot.1);
            boundaries.push(vec![r.pivot]);
            canvas[y][x] = region_id;
        }

        loop {
            let mut changed = false;

            for (region_id, bs) in boundaries.iter_mut().enumerate() {
                let mut newbs = vec![];

                for p in bs.iter() {
                    let neighbors = {
                        let ox = i32::from(p.0);
                        let oy = i32::from(p.1);

                        [(ox - 1, oy), (ox, oy - 1), (ox, oy + 1), (ox + 1, oy)]
                    };

                    for &(x, y) in &neighbors {
                        if x < 0 || y < 0 || x >= i32::from(w) || y >= i32::from(h) {
                            continue;
                        }
                        let x = x as u16;
                        let y = y as u16;

                        let on_boundary = x == 0 || x == w - 1 || y == 0 || y == h - 1;

                        let mut closest_rid = canvas[usize::from(y)][usize::from(x)];
                        if closest_rid >= regions.len() {
                            canvas[usize::from(y)][usize::from(x)] = region_id;
                            closest_rid = region_id;
                            newbs.push((x, y));
                        }

                        if closest_rid != region_id {
                            regions[region_id].neighbors.insert(closest_rid);
                            regions[closest_rid].neighbors.insert(region_id);
                            regions[region_id].boundary.insert(*p);
                        } else if on_boundary {
                            regions[region_id].boundary.insert((x, y));
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

impl Region {
    /// Return the boundary as a polyline.
    pub fn boundary_polyline(&self) -> Vec<Point> {
        // the idea here to connect all the points in the boundary is to pick a random point and
        // start following it along a direction until we reach the starting point or there are no
        // more points to consider.

        let mut cb = vec![];
        if self.boundary.is_empty() {
            return cb;
        }

        // collecting into a btreeset allows to quickly find the minimum which is used as the
        // starting point of the polyline
        let mut bs = self
            .boundary
            .iter()
            .map(|(x, y)| (i32::from(*x), i32::from(*y)))
            .collect::<BTreeSet<_>>();

        const DIRS: [(i32, i32); 8] = [
            (0, -1),
            (-1, -1),
            (-1, 0),
            (-1, 1),
            (0, 1),
            (1, 1),
            (1, 0),
            (1, -1),
        ];

        let start_p = *bs.iter().next().unwrap();
        let mut cur_p = start_p;
        let mut cur_dir = 0;

        loop {
            cb.push((cur_p.0 as u16, cur_p.1 as u16));
            bs.remove(&cur_p);

            let next = (0..DIRS.len())
                .map(|i| {
                    let i = (cur_dir + i) % DIRS.len();
                    let d = DIRS[i];
                    (i, (cur_p.0 + d.0, cur_p.1 + d.1))
                })
                .find(|(_, p)| bs.contains(p));

            let (new_dir, new_p) = match next {
                None => break,
                Some(p) => p,
            };

            if new_p == start_p {
                break;
            }

            // if the direction didn't change then the new point is colinear with the previous and
            // we can replace the previous point with the new one. However, make sure to always
            // keep the first point of the polyline.
            if cur_p != start_p && cur_dir == new_dir {
                cb.pop();
            }

            cur_p = new_p;
            cur_dir = new_dir;
        }

        cb
    }
}
