use std::collections::HashSet;

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
    pub fn connected_boundary(&self) -> Vec<Point> {
        let mut b = self.boundary.iter().cloned().collect::<Vec<_>>();

        for i in 1..b.len() {
            let p = b[i - 1];
            let closest_i = i + b[i..]
                .iter()
                .enumerate()
                .min_by_key(|(_, pp)| {
                    (i32::from(pp.0) - i32::from(p.0)).abs()
                        + (i32::from(pp.1) - i32::from(p.1)).abs()
                })
                .map(|(i, _)| i)
                .unwrap();

            b.swap(i, closest_i);
        }

        b
    }
}
