use minifb::{Key, Window};
use std::f32::consts::PI;
use crate::player::Player;

pub fn process_events(window: &Window, player: &mut Player, maze: &Vec<Vec<char>>, block_size: usize) {
    const MOVE_SPEED: f32 = 0.1;
    const ROTATION_SPEED: f32 = PI / 30.0;

    if window.is_key_down(Key::Left) {
        player.a -= ROTATION_SPEED;
    }
    if window.is_key_down(Key::Right) {
        player.a += ROTATION_SPEED;
    }

    let mut next_pos_x = player.pos.x;
    let mut next_pos_y = player.pos.y;

    if window.is_key_down(Key::Up) {
        // Intentar mover hacia adelante en la dirección de vista
        next_pos_x += player.a.cos() * MOVE_SPEED;
        next_pos_y += player.a.sin() * MOVE_SPEED;
    }
    if window.is_key_down(Key::Down) {
        // Intentar mover hacia atrás en la dirección opuesta a la vista
        next_pos_x -= player.a.cos() * MOVE_SPEED;
        next_pos_y -= player.a.sin() * MOVE_SPEED;
    }

    // Convertir la siguiente posición en índices de celda
    let next_cell_x = next_pos_x as usize;
    let next_cell_y = next_pos_y as usize;

    // Verificar si la siguiente posición está dentro del laberinto y no es una pared
    if next_cell_x < maze[0].len() && next_cell_y < maze.len() && maze[next_cell_y][next_cell_x] == ' ' {
        // Si no es una pared, actualizamos la posición del jugador
        player.pos.x = next_pos_x;
        player.pos.y = next_pos_y;
    }
}