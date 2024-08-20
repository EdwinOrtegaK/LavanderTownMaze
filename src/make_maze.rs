use std::fs::File;
use std::io::{self, Write};
use rand::seq::SliceRandom;
use rand::Rng;

pub fn make_maze(w: usize, h: usize) -> Vec<Vec<char>> {
    let mut vis = vec![vec![0; w]; h];
    let mut ver = vec![vec!['|'; w]; h];
    let mut hor = vec![vec!['+'; w + 1]; h + 1];

    for y in 0..h {
        ver[y].push('|');
    }

    for x in 0..=w {
        hor[h].push('+');
    }

    let mut rng = rand::thread_rng();
    let mut d = vec![(0, 0); 4];

    fn walk(x: usize, y: usize, vis: &mut Vec<Vec<u8>>, hor: &mut Vec<Vec<char>>, ver: &mut Vec<Vec<char>>, rng: &mut impl Rng, d: &mut Vec<(usize, usize)>) {
        vis[y][x] = 1;

        d[0] = (x.wrapping_sub(1), y);
        d[1] = (x, y + 1);
        d[2] = (x + 1, y);
        d[3] = (x, y.wrapping_sub(1));
        d.shuffle(rng);

        for i in 0..d.len() {
            let (xx, yy) = d[i];
            if yy < vis.len() && xx < vis[0].len() && vis[yy][xx] != 0 { continue; }
            if xx == x && yy < hor.len() {
                hor[std::cmp::max(y, yy)][x] = ' ';
            }
            if yy == y && xx < ver[y].len() {
                ver[y][std::cmp::max(x, xx)] = ' ';
            }
            if yy < vis.len() && xx < vis[0].len() {
                walk(xx, yy, vis, hor, ver, rng, d);
            }
        }
    }

    walk(rng.gen_range(0..w), rng.gen_range(0..h), &mut vis, &mut hor, &mut ver, &mut rng, &mut d);

    // Convertimos las líneas a un array 2D de caracteres
    let mut maze = Vec::new();

    for (a, b) in hor.iter().zip(ver.iter()) {
        let mut line_a = a.clone();
        let mut line_b = b.clone();
        line_a.push('\n');
        line_b.push('\n');
        maze.push(line_a);
        maze.push(line_b);
    }

    // Colocar 'p' y 'g' en las posiciones adecuadas
    let last_row_index = maze.len() - 2;
    let last_col_index = maze[0].len() - 2;
    maze[1][1] = 'p';
    maze[last_row_index][last_col_index] = 'g';

    maze
}

pub fn save_maze_to_file(filename: &str, maze: &Vec<Vec<char>>) -> io::Result<()> {
    let mut file = File::create(filename)?;

    for line in maze {
        for &ch in line {
            write!(file, "{}", ch)?;
        }
    }

    Ok(())
}
