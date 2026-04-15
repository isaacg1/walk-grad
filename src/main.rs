use hashbrown::{DefaultHashBuilder, HashTable};
use image::{ImageBuffer, RgbImage};
use rand::prelude::*;
use std::collections::HashSet;
use std::env;
use std::hash::{BuildHasher, Hash, Hasher};

fn remove_random<T, R>(set: &mut HashTable<T>, rng: &mut R) -> Option<T>
where
    R: Rng,
    T: Eq + PartialEq + Hash,
{
    if set.is_empty() {
        return None;
    }
    // If load factor is under 25%, shrink to fit.
    // We need a high load factor to ensure that the sampling succeeds in a reasonable time,
    // and the table doesn't rebalance on removals.
    // Insertions can only cause the load factor to reach as low as 50%,
    // so it's safe to shrink at 25%.
    let hasher = DefaultHashBuilder::default();
    let hasher = |val: &_| hasher.hash_one(val);
    if set.capacity() >= 8 && set.len() < set.capacity() / 4 {
        set.shrink_to_fit(hasher);
    }
    let num_buckets = set.num_buckets();
    // Perform rejection sampling: Pick a random bucket, check if it's full,
    // repeat until a full bucket is found.
    loop {
        let bucket_index = rng.random_range(0..num_buckets);
        let out = set
            .get_bucket_entry(bucket_index)
            .ok()
            .map(|occupied| occupied.remove().0);
        if out.is_some() {
            return out;
        }
    }
}

type BaseColor = [f64; 3];
type Location = [usize; 2];

fn neighbors(location: Location, size: usize) -> [Location; 4] {
    [
        [(location[0] + 1) % size, location[1]],
        [location[0], (location[1] + 1) % size],
        [(location[0] + size - 1) % size, location[1]],
        [location[0], (location[1] + size - 1) % size],
    ]
}
const PERMS: [[usize; 4]; 24] = [
    [0, 1, 2, 3],
    [0, 1, 3, 2],
    [0, 2, 1, 3],
    [0, 2, 3, 1],
    [0, 3, 1, 2],
    [0, 3, 2, 1],
    [1, 0, 2, 3],
    [1, 0, 3, 2],
    [1, 2, 0, 3],
    [1, 2, 3, 0],
    [1, 3, 0, 2],
    [1, 3, 2, 0],
    [2, 0, 1, 3],
    [2, 0, 3, 1],
    [2, 1, 0, 3],
    [2, 1, 3, 0],
    [2, 3, 0, 1],
    [2, 3, 1, 0],
    [3, 0, 1, 2],
    [3, 0, 2, 1],
    [3, 1, 0, 2],
    [3, 1, 2, 0],
    [3, 2, 0, 1],
    [3, 2, 1, 0],
];
const DEBUG: bool = false;
fn run(size: usize, length_alpha: f64, bug: u8, seed: u64) -> RgbImage {
    let mut rng = StdRng::seed_from_u64(seed);
    let mut grid: Vec<Vec<Option<BaseColor>>> = vec![vec![None; size]; size];
    let mut blank: HashTable<Location> = HashTable::new();
    let hasher = |val: &Location| {
        let mut hasher = std::hash::DefaultHasher::default();
        hasher.write_usize(val[0]);
        hasher.write_usize(val[1]);
        hasher.finish()
    };
    for i in 0..size {
        for j in 0..size {
            let val = [i, j];
            blank.insert_unique(hasher(&val), val, hasher);
        }
    }
    let walk_length_cap = (size as f64).powf(length_alpha) as usize;
    'outer: while let Some(start) = remove_random(&mut blank, &mut rng) {
        if grid[start[0]][start[1]].is_some() {
            continue;
        }
        let mut walks = [(vec![start], false), (vec![start], false)];
        let mut seen: HashSet<Location> = HashSet::new();
        seen.insert(start);
        while (walks[0].0.len() < walk_length_cap && !walks[0].1)
            || (walks[1].0.len() < walk_length_cap && !walks[1].1)
        {
            for walk in &mut walks {
                if walk.0.len() < walk_length_cap && !walk.1 {
                    let perm_index = rng.random_range(0..24);
                    let perm = PERMS[perm_index];
                    let neighs = neighbors(*walk.0.last().expect("occupied"), size);
                    let mut inserted = false;
                    for index in perm {
                        let neigh = neighs[index];
                        if !seen.contains(&neigh) {
                            walk.0.push(neigh);
                            seen.insert(neigh);
                            if grid[neigh[0]][neigh[1]].is_some() {
                                walk.1 = true;
                            }
                            inserted = true;
                            break;
                        }
                    }
                    if !inserted {
                        walk.0.pop();
                        if walk.0.is_empty() {
                            continue 'outer;
                        }
                    }
                }
            }
        }
        for walk in &mut walks {
            if !walk.1 {
                let last = walk.0.last().expect("Nonempty");
                if DEBUG {
                    println!("Placing random {:?}", last);
                }
                assert!(grid[last[0]][last[1]].is_none());
                grid[last[0]][last[1]] = Some([rng.random(), rng.random(), rng.random()]);
                blank
                    .find_entry(hasher(last), |val| val == last)
                    .ok()
                    .map(|occupied| occupied.remove());
            }
        }
        let last1 = walks[0].0.last().expect("Nonempty 1");
        let last2 = walks[1].0.last().expect("Nonempty 2");
        let ends1 = grid[last1[0]][last1[1]].expect("Filled 1");
        let ends2 = grid[last2[0]][last2[1]].expect("Filled 2");
        let length = walks[0].0.len() - 1 + walks[1].0.len() - 1;
        let b1 = if bug & 1 == 0 { ends1 } else { ends2 };
        let b2 = if bug & 2 == 0 { ends1 } else { ends2 };
        let b3 = if bug & 4 == 0 { 1.0 } else { -1.0 };
        let diff = [
            b3 * (ends2[0] - ends1[0]) / length as f64,
            b3 * (ends2[1] - ends1[1]) / length as f64,
            b3 * (ends2[2] - ends1[2]) / length as f64,
        ];
        for (i, location) in walks[0].0.iter().rev().skip(1).enumerate() {
            let color = [
                b1[0] + diff[0] * (i + 1) as f64,
                b1[1] + diff[1] * (i + 1) as f64,
                b1[2] + diff[2] * (i + 1) as f64,
            ];
            if DEBUG {
                println!("Placing sequence {}: {:?} - {:?}", i, location, walks[0].0);
            }
            assert!(grid[location[0]][location[1]].is_none());
            grid[location[0]][location[1]] = Some(color);
            blank
                .find_entry(hasher(location), |val| val == location)
                .ok()
                .map(|occupied| occupied.remove());
        }
        for (i, location) in walks[1].0[..walks[1].0.len() - 1]
            .iter()
            .enumerate()
            .skip(1)
        {
            let position = walks[0].0.len() - 1 + i;
            let color = [
                b2[0] + diff[0] * position as f64,
                b2[1] + diff[1] * position as f64,
                b2[2] + diff[2] * position as f64,
            ];
            if DEBUG {
                println!("Placing sequence {}: {:?}", position, location);
            }
            assert!(grid[location[0]][location[1]].is_none());
            grid[location[0]][location[1]] = Some(color);
            blank
                .find_entry(hasher(location), |val| val == location)
                .ok()
                .map(|occupied| occupied.remove());
        }
    }
    let mut img: RgbImage = ImageBuffer::new(size as u32, size as u32);
    for (i, row) in grid.into_iter().enumerate() {
        for (j, cell) in row.into_iter().enumerate() {
            let color = cell.unwrap_or([0.5; 3]).map(|f| (f * 255.999999) as u8);
            img.put_pixel(i as u32, j as u32, image::Rgb(color));
        }
    }
    img
}

fn main() {
    let size = env::args()
        .nth(1)
        .expect("size present")
        .parse()
        .expect("size num");
    let length_alpha = env::args()
        .nth(2)
        .expect("alpha present")
        .parse()
        .expect("alpha num");
    let seed = env::args()
        .nth(3)
        .expect("seed present")
        .parse()
        .expect("seed num");
    for bug in 0..8 {
        let filename = format!("img-{size}-{length_alpha}-{bug}-{seed}.png");
        println!("{filename}");
        let image = run(size, length_alpha, bug, seed);
        image.save(filename).expect("Saved");
    }
}
