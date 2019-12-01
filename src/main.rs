use std::collections::HashSet;
use std::fs;
use std::io;
use std::io::Write;
use std::path;

use colored::Colorize;
use rand::prelude::*;
use structopt::StructOpt;

use map_4col::colormap::{Color, ColorMap};
use map_4col::map::{Map, Point};

/// Simple program to generate maps colored following the 4 color theorem.
#[derive(Debug, StructOpt)]
struct Opts {
    /// Width of the map
    #[structopt(short, long)]
    width: Option<u16>,

    /// Height of the map
    #[structopt(short, long)]
    height: Option<u16>,

    /// Maximum number of regions in the map
    #[structopt(short, long, default_value = "5")]
    nregions: u16,

    #[structopt(subcommand)]
    format: Format,
}

#[derive(Debug, StructOpt)]
enum Format {
    /// Print the map directly to the terminal
    Terminal,

    /// Render the map as an svg
    Svg {
        /// Path where to write the map as svg
        #[structopt(short, long, default_value = "map.svg")]
        filename: path::PathBuf,
    },
}

impl Format {
    pub fn default_dimensions(&self) -> (u16, u16) {
        match self {
            Format::Terminal => (80, 24),
            Format::Svg { .. } => (400, 800),
        }
    }
}

fn main() -> io::Result<()> {
    let opts = Opts::from_args();

    let (default_w, default_h) = opts.format.default_dimensions();
    let dim = (
        opts.width.unwrap_or(default_w),
        opts.height.unwrap_or(default_h),
    );

    let pivots = random_pivots(opts.nregions, dim);
    let m = Map::voronoi_like(pivots, dim);
    let cm = ColorMap::color(&m).unwrap();

    match opts.format {
        Format::Terminal => dump_terminal(dim, &m, &cm),
        Format::Svg { filename } => dump_svg(&filename, dim, &m, &cm)?,
    }

    Ok(())
}

fn dump_terminal(dim: (u16, u16), m: &Map, cm: &ColorMap) {
    let c1 = " ".on_red().dimmed().to_string();
    let c2 = " ".on_blue().dimmed().to_string();
    let c3 = " ".on_green().dimmed().to_string();
    let c4 = " ".on_yellow().dimmed().to_string();

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
    // for (rid, r) in m.regions.iter().enumerate() {
    //     for (x, y) in &r.boundary {
    //         dbg_display[usize::from(*y)][usize::from(*x)] = match cm.color_of_region(rid) {
    //             Color::C1 => &b1,
    //             Color::C2 => &b2,
    //             Color::C3 => &b3,
    //             Color::C4 => &b4,
    //         };
    //     }
    // }

    // let p1 = "X".on_red().to_string();
    // let p2 = "X".on_blue().to_string();
    // let p3 = "X".on_green().to_string();
    // let p4 = "X".on_yellow().to_string();
    // for (rid, r) in m.regions.iter().enumerate() {
    //     let x = usize::from(r.pivot.0);
    //     let y = usize::from(r.pivot.1);

    //     dbg_display[y][x] = match cm.color_of_region(rid) {
    //         Color::C1 => &p1,
    //         Color::C2 => &p2,
    //         Color::C3 => &p3,
    //         Color::C4 => &p4,
    //     };
    // }

    for r in dbg_display {
        let s = r.into_iter().collect::<String>();
        println!("{}", s);
    }
}

fn dump_svg(filename: &path::PathBuf, dim: (u16, u16), m: &Map, cm: &ColorMap) -> io::Result<()> {
    let mut f = fs::File::create(filename)?;

    let stroke_width = ((f32::from(dim.0) * f32::from(dim.1)) / (400.0 * 800.0) * 2.5).min(5.0);

    writeln!(
        f,
        r#"<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE svg PUBLIC "-//W3C//DTD SVG 1.1//EN" "http://www.w3.org/Graphics/SVG/1.1/DTD/svg11.dtd">
<svg xmlns="http://www.w3.org/2000/svg" version="1.1" viewBox="0 0 {width} {height}" >
<rect width="{width}" height="{height}" stroke="none" fill="black" />"#,
        width = dim.0,
        height = dim.1
    )?;

    for (rid, r) in m.regions.iter().enumerate() {
        let points = r
            .connected_boundary()
            .into_iter()
            .map(|(x, y)| format!("{},{} ", x, y))
            .collect::<String>();

        writeln!(
            f,
            r#"<polygon points="{}" stroke-width="{}" stroke="black" fill="{}" />"#,
            points,
            stroke_width,
            match cm.color_of_region(rid) {
                Color::C1 => "#3604ff",
                Color::C2 => "#ffde00",
                Color::C3 => "#ff0041",
                Color::C4 => "#00ffed",
            }
        )?
    }

    writeln!(f, "</svg>")?;

    Ok(())
}

fn random_pivots(npivots: u16, (width, height): (u16, u16)) -> HashSet<Point> {
    let mut rng = thread_rng();
    (0..npivots)
        .map(|_| {
            let x = rng.gen_range(0, width);
            let y = rng.gen_range(0, height);
            (x, y)
        })
        .collect::<HashSet<_>>()
}
