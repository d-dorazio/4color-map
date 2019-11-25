use std::fs;
use std::io;
use std::io::Write;

mod colormap;
mod map;

use colored::Colorize;
use rand::prelude::*;

use crate::colormap::*;
use crate::map::Map;

fn main() -> io::Result<()> {
    let dim = (80, 24);
    let npivots = 10;

    let m = Map::voronoi_like(Map::random_pivots(&mut thread_rng(), npivots, dim), dim);
    let cm = ColorMap::color(&m).unwrap();
    let regions = &m.regions;

    let c1 = " ".on_red().dimmed().to_string();
    let c2 = " ".on_blue().dimmed().to_string();
    let c3 = " ".on_green().dimmed().to_string();
    let c4 = " ".on_yellow().dimmed().to_string();
    let p1 = "X".on_red().to_string();
    let p2 = "X".on_blue().to_string();
    let p3 = "X".on_green().to_string();
    let p4 = "X".on_yellow().to_string();

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

    // print boundaries
    // let b1 = "*".on_red().to_string();
    // let b2 = "*".on_blue().to_string();
    // let b3 = "*".on_green().to_string();
    // let b4 = "*".on_yellow().to_string();
    // for (rid, r) in regions.iter().enumerate() {
    //     for (x, y) in &r.boundary {
    //         dbg_display[usize::from(*y)][usize::from(*x)] = match cm.color_of_region(rid) {
    //             Color::C1 => &b1,
    //             Color::C2 => &b2,
    //             Color::C3 => &b3,
    //             Color::C4 => &b4,
    //         };
    //     }
    // }

    for (rid, r) in regions.iter().enumerate() {
        let x = usize::from(r.pivot.0);
        let y = usize::from(r.pivot.1);

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

    {
        let mut f = fs::File::create("map.svg")?;
        writeln!(
            f,
            r#"<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE svg PUBLIC "-//W3C//DTD SVG 1.1//EN" "http://www.w3.org/Graphics/SVG/1.1/DTD/svg11.dtd">
<svg xmlns="http://www.w3.org/2000/svg" version="1.1" viewBox="0 0 {width} {height}" >
<rect width="{width}" height="{height}" stroke="none" fill="black" />"#,
            width = dim.0,
            height = dim.1
        )?;
        for (rid, r) in regions.iter().enumerate() {
            let points = r
                .connected_boundary()
                .into_iter()
                .map(|(x, y)| format!("{},{} ", x, y))
                .collect::<String>();
            writeln!(
                f,
                r#"<polygon points="{}" stroke="none" fill="{}" />"#,
                points,
                match cm.color_of_region(rid) {
                    Color::C1 => "red",
                    Color::C2 => "blue",
                    Color::C3 => "green",
                    Color::C4 => "yellow",
                }
            )?
        }
        writeln!(f, "</svg>")?;
    }

    Ok(())
}
