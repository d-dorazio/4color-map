mod colormap;
mod map;

use colored::Colorize;
use rand::prelude::*;

use crate::colormap::*;
use crate::map::Map;

fn main() {
    let dim = (80, 24);
    let npivots = 10;

    let m = Map::voronoi_like(&mut thread_rng(), dim, npivots);
    let cm = ColorMap::color(&m).unwrap();
    let regions = &m.regions;

    let c1 = " ".on_red().to_string();
    let c2 = " ".on_blue().to_string();
    let c3 = " ".on_green().to_string();
    let c4 = " ".on_yellow().to_string();
    let p1 = "X".on_red().to_string();
    let p2 = "X".on_blue().to_string();
    let p3 = "X".on_green().to_string();
    let p4 = "X".on_yellow().to_string();
    let pb = "X".on_black().to_string();
    let b = " ".on_black().to_string();

    let mut dbg_display = vec![vec![" "; usize::from(dim.0)]; usize::from(dim.1)];
    for (y, row) in m.raster.iter().enumerate() {
        for (x, rid) in row.iter().enumerate() {
            dbg_display[y][x] = match cm.color_of_region(*rid) {
                Color::C1 => &c1,
                Color::C2 => &c2,
                Color::C3 => &c3,
                Color::C4 => &c4,
            };
        }
    }

    for r in regions.iter() {
        for (x, y) in &r.boundary {
            dbg_display[y.floor() as usize][x.floor() as usize] = &b;
        }
    }
    for (rid, r) in regions.iter().enumerate() {
        let x = usize::from(r.pivot.0);
        let y = usize::from(r.pivot.1);

        if dbg_display[y][x] == b {
            dbg_display[y][x] = &pb;
            continue;
        }

        dbg_display[y][x] = match cm.color_of_region(rid) {
            Color::C1 => &p1,
            Color::C2 => &p2,
            Color::C3 => &p3,
            Color::C4 => &p4,
        };
    }
    for r in dbg_display {
        let s = r.into_iter().collect::<String>();
        println!("{}", s);
    }
}
