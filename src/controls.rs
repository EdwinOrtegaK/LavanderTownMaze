use minifb::{Key, Window};
use std::f32::consts::PI;
use crate::player::Player;
use crate::audio::AudioPlayer;

pub fn process_events(window: &Window, player: &mut Player, maze: &Vec<Vec<char>>, block_size: usize, steps_player: &AudioPlayer) {
    const MOVE_SPEED: f32 = 0.1;
    const ROTATION_SPEED: f32 = PI / 30.0;
    let mut moved = false;

    // Rotación del jugador con A y D
    if window.is_key_down(Key::A) {
        player.a -= ROTATION_SPEED;
    }
    if window.is_key_down(Key::D) {
        player.a += ROTATION_SPEED;
    }

    let mut next_pos_x = player.pos.x;
    let mut next_pos_y = player.pos.y;

    // Movimiento del jugador con W y S
    if window.is_key_down(Key::W) {
        next_pos_x += player.a.cos() * MOVE_SPEED;
        next_pos_y += player.a.sin() * MOVE_SPEED;
        moved = true;  // El jugador se ha movido
    }
    if window.is_key_down(Key::S) {
        next_pos_x -= player.a.cos() * MOVE_SPEED;
        next_pos_y -= player.a.sin() * MOVE_SPEED;
        moved = true;  // El jugador se ha movido
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

    // Reproducir o pausar el sonido de los pasos dependiendo si el jugador se mueve o no
    if moved {
        steps_player.play();  // Reproducir sonido de pasos si el jugador se mueve
    } else {
        steps_player.pause();  // Pausar sonido de pasos si el jugador no se mueve
    }
}