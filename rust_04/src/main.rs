use clap::Parser;
use colored::*;
use rand::Rng;
use std::cmp::Ordering;
use std::collections::{BinaryHeap, HashMap};
use std::fs;
use std::thread;
use std::time::Duration;

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    #[arg(required_unless_present = "generate")]
    file: Option<String>,

    #[arg(short, long)]
    generate: Option<String>,

    #[arg(short, long)]
    output: Option<String>,

    #[arg(short, long)]
    visualize: bool,

    #[arg(short, long)]
    both: bool,

    #[arg(short, long)]
    animate: bool,
}

#[derive(Copy, Clone, Eq, PartialEq)]
struct State {
    cost: u32,
    position: (usize, usize),
}

impl Ord for State {
    fn cmp(&self, other: &Self) -> Ordering {
        other
            .cost
            .cmp(&self.cost)
            .then_with(|| self.position.0.cmp(&other.position.0))
            .then_with(|| self.position.1.cmp(&other.position.1))
    }
}

impl PartialOrd for State {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

struct Grid {
    width: usize,
    height: usize,
    data: Vec<u8>,
}

impl Grid {
    fn get(&self, x: usize, y: usize) -> u8 {
        self.data[y * self.width + x]
    }
}

fn main() {
    let args = Args::parse();

    let grid = if let Some(size_str) = args.generate {
        let parts: Vec<&str> = size_str.split('x').collect();
        if parts.len() != 2 {
            eprintln!("Format invalide. Utilisez LxH (ex: 12x8)");
            return;
        }
        let w: usize = parts[0].parse().expect("Largeur invalide");
        let h: usize = parts[1].parse().expect("Hauteur invalide");

        println!("Generating {}x{} hexadecimal grid...", w, h);
        let map = generate_map(w, h);

        if let Some(path) = &args.output {
            save_map(&map, w, h, path);
            println!("Map saved to: {}", path);
        }

        if args.visualize {
            print_colored_grid(&map, w, h, &[], false);
        } else {
            print_raw_grid(&map, w, h);
        }

        Grid {
            width: w,
            height: h,
            data: map,
        }
    } else if let Some(path) = args.file {
        println!("Analyzing hexadecimal grid...");
        let (map, w, h) = load_map(&path);

        if args.visualize {
            println!("HEXADECIMAL GRID (rainbow gradient):");
            println!("======================================");
            print_colored_grid(&map, w, h, &[], false);
            return;
        }

        Grid {
            width: w,
            height: h,
            data: map,
        }
    } else {
        return;
    };

    println!("\nMINIMUM COST PATH:");
    println!("==================");
    let min_path = solve_dijkstra(&grid, false, args.animate);
    print_path_result(&grid, &min_path, "minimum");

    if args.both {
        println!("\nMAXIMUM COST PATH:");
        println!("==================");
        let max_path = solve_dijkstra(&grid, true, false);
        print_path_result(&grid, &max_path, "maximum");
    }
}

fn solve_dijkstra(grid: &Grid, maximize: bool, animate: bool) -> Vec<(usize, usize)> {
    let start = (0, 0);
    let end = (grid.width - 1, grid.height - 1);

    let mut dist: HashMap<(usize, usize), u32> = HashMap::new();
    let mut came_from: HashMap<(usize, usize), (usize, usize)> = HashMap::new();
    let mut heap = BinaryHeap::new();

    dist.insert(start, 0);
    heap.push(State {
        cost: 0,
        position: start,
    });

    let mut steps = 0;

    if animate {
        print!("\x1B[2J");
    }

    while let Some(State { cost, position }) = heap.pop() {
        steps += 1;

        if animate && steps % 5 == 0 {
            print!("\x1B[1;1H");
            println!("Searching for minimum cost path...\n");
            println!("Step {}: Exploring {:?} - cost: {}", steps, position, cost);
            print_animated_grid(grid, &dist, position);
            thread::sleep(Duration::from_millis(50));
        }

        if position == end {
            if animate {
                print!("\x1B[1;1H");
                println!("\nStep {}: Path found!                 \n", steps);
                print_animated_grid(grid, &dist, position);
            }
            return reconstruct_path(came_from, end);
        }

        if cost > *dist.get(&position).unwrap_or(&u32::MAX) {
            continue;
        }

        let (x, y) = position;
        let neighbors = [
            if x > 0 { Some((x - 1, y)) } else { None },
            if x < grid.width - 1 {
                Some((x + 1, y))
            } else {
                None
            },
            if y > 0 { Some((x, y - 1)) } else { None },
            if y < grid.height - 1 {
                Some((x, y + 1))
            } else {
                None
            },
        ];

        for neighbor in neighbors.iter().flatten() {
            let val = grid.get(neighbor.0, neighbor.1) as u32;
            let weight = if maximize { 255 - val } else { val };
            let next_cost = cost + weight;

            if next_cost < *dist.get(neighbor).unwrap_or(&u32::MAX) {
                heap.push(State {
                    cost: next_cost,
                    position: *neighbor,
                });
                dist.insert(*neighbor, next_cost);
                came_from.insert(*neighbor, position);
            }
        }
    }

    vec![]
}

fn reconstruct_path(
    came_from: HashMap<(usize, usize), (usize, usize)>,
    current: (usize, usize),
) -> Vec<(usize, usize)> {
    let mut path = vec![current];
    let mut curr = current;
    while let Some(&prev) = came_from.get(&curr) {
        path.push(prev);
        curr = prev;
    }
    path.reverse();
    path
}

fn generate_map(w: usize, h: usize) -> Vec<u8> {
    let mut rng = rand::rng();
    let mut map = vec![0u8; w * h];
    for i in 0..map.len() {
        map[i] = rng.random();
    }

    let last_idx = map.len() - 1;
    map[0] = 0x00;
    map[last_idx] = 0xFF;
    map
}

fn save_map(map: &[u8], w: usize, h: usize, path: &str) {
    let mut content = String::new();
    for y in 0..h {
        for x in 0..w {
            content.push_str(&format!("{:02X} ", map[y * w + x]));
        }
        content.push('\n');
    }
    fs::write(path, content).expect("Erreur écriture fichier");
}

fn load_map(path: &str) -> (Vec<u8>, usize, usize) {
    let content = fs::read_to_string(path).expect("Fichier introuvable");
    let mut map = Vec::new();
    let mut width = 0;
    let mut height = 0;

    for line in content.lines() {
        let parts: Vec<u8> = line
            .split_whitespace()
            .map(|s| u8::from_str_radix(s, 16).expect("Hex invalide"))
            .collect();

        if parts.is_empty() {
            continue;
        }
        if width == 0 {
            width = parts.len();
        }
        height += 1;
        map.extend(parts);
    }
    (map, width, height)
}

fn get_color(val: u8) -> Color {
    match val {
        0..=40 => Color::TrueColor {
            r: 255,
            g: val * 5,
            b: 0,
        },

        41..=128 => Color::TrueColor {
            r: 255 - (val - 40) * 2,
            g: 255,
            b: 0,
        },

        129..=180 => Color::TrueColor {
            r: 0,
            g: 255,
            b: (val - 128) * 4,
        },

        _ => {
            let r_calc = (val as u16 - 180) * 4;
            let r_final = if r_calc > 255 { 255 } else { r_calc as u8 };

            Color::TrueColor {
                r: r_final,
                g: 0,
                b: 255,
            }
        }
    }
}

fn print_colored_grid(map: &[u8], w: usize, h: usize, path: &[(usize, usize)], is_anim: bool) {
    let path_set: HashMap<_, _> = path.iter().map(|&p| (p, true)).collect();

    for y in 0..h {
        for x in 0..w {
            let val = map[y * w + x];
            let s = format!("{:02X}", val);

            if path_set.contains_key(&(x, y)) {
                print!("{} ", s.white().on_black().bold());
            } else if is_anim {
                print!("{} ", "[]".dimmed());
            } else {
                print!("{} ", s.color(get_color(val)));
            }
        }
        println!();
    }
}

fn print_raw_grid(map: &[u8], w: usize, h: usize) {
    for y in 0..h {
        for x in 0..w {
            print!("{:02X} ", map[y * w + x]);
        }
        println!();
    }
}

fn print_animated_grid(
    grid: &Grid,
    visited: &HashMap<(usize, usize), u32>,
    current: (usize, usize),
) {
    for y in 0..grid.height {
        print!("[");
        for x in 0..grid.width {
            if (x, y) == current {
                print!("*");
            } else if visited.contains_key(&(x, y)) {
                print!("v");
            } else {
                print!(" ");
            }
            if x < grid.width - 1 {
                print!("][");
            }
        }
        println!("]");
    }
}

fn print_path_result(grid: &Grid, path: &[(usize, usize)], label: &str) {
    if path.is_empty() {
        println!("No path found!");
        return;
    }

    let mut total_cost = 0;
    for (i, &(x, y)) in path.iter().enumerate() {
        if i > 0 {
            total_cost += grid.get(x, y) as u32;
        }
    }

    println!("Total cost: 0x{:X} ({} decimal)", total_cost, total_cost);
    println!("Path length: {} steps", path.len());

    println!("Path:");
    let path_str: Vec<String> = path.iter().map(|(x, y)| format!("({},{})", x, y)).collect();
    println!("{}", path_str.join("->"));

    println!("\n{} COST PATH (shown in WHITE):", label.to_uppercase());
    println!("================================");
    print_colored_grid(&grid.data, grid.width, grid.height, path, false);
}
