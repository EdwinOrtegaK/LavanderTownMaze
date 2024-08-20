mod maze;
mod player;
mod raycasting;
mod controls;
mod textures;

use player::Player;
use raycasting::cast_ray;
use controls::process_events;
use minifb::{Key, Window, WindowOptions};
use nalgebra as na;
use textures::Texture;
use once_cell::sync::Lazy;
use std::sync::Arc;
use rusttype::{Font, Scale};
use std::time::Instant;

const WIDTH: usize = 1040;
const HEIGHT: usize = 900;

static WALL1: Lazy<Arc<Texture>> = Lazy::new(|| Arc::new(Texture::new("sprites/casaSprite.png")));
static WALL2: Lazy<Arc<Texture>> = Lazy::new(|| Arc::new(Texture::new("sprites/casaSprite2.png")));
static FLOOR: Lazy<Arc<Texture>> = Lazy::new(|| Arc::new(Texture::new("sprites/floorSprite.png")));
static SKY: Lazy<Arc<Texture>> = Lazy::new(|| Arc::new(Texture::new("sprites/skySprite.png")));
static CHARACTER: Lazy<Arc<Texture>> = Lazy::new(|| Arc::new(Texture::new("sprites/ghastSprite.jpg")));

fn cell_to_texture_color(wall_type: char, _is_vertical: bool, tx: f32, ty: f32) -> u32 {
    match wall_type {
        '|' => WALL1.get_pixel_color((tx * WALL1.width as f32) as u32, (ty * WALL1.height as f32) as u32),
        '-' => WALL2.get_pixel_color((tx * WALL2.width as f32) as u32, (ty * WALL2.height as f32) as u32),
        _ => WALL1.get_pixel_color((tx * WALL1.width as f32) as u32, (ty * WALL1.height as f32) as u32),
    }
}

fn render_sky(framebuffer: &mut Vec<u32>) {
    for y in 0..HEIGHT / 2 {
        for x in 0..WIDTH {
            let tx = (x as f32 / WIDTH as f32 * SKY.width as f32) as u32;
            let ty = (y as f32 / (HEIGHT / 2) as f32 * SKY.height as f32) as u32;
            let color = SKY.get_pixel_color(tx, ty);
            framebuffer[y * WIDTH + x] = color;
        }
    }
}

fn render_text(framebuffer: &mut Vec<u32>, text: &str, x: usize, y: usize, scale: Scale, color: u32) {
    let font_data = include_bytes!("../assets/PressStart2P.ttf");
    let font = Font::try_from_bytes(font_data as &[u8]).unwrap();

    let v_metrics = font.v_metrics(scale);

    let glyphs: Vec<_> = font
        .layout(text, scale, rusttype::point(x as f32, y as f32 + v_metrics.ascent))
        .collect();

    for glyph in glyphs {
        if let Some(bb) = glyph.pixel_bounding_box() {
            glyph.draw(|gx, gy, gv| {
                let px = (gx as i32 + bb.min.x) as usize;
                let py = (gy as i32 + bb.min.y) as usize;

                if px < WIDTH && py < HEIGHT {
                    let alpha = (gv * 255.0) as u32;
                    let foreground = if alpha > 128 {
                        color & 0xFFFFFF
                    } else {
                        framebuffer[py * WIDTH + px]
                    };

                    framebuffer[py * WIDTH + px] = foreground;
                }
            });
        }
    }
}

fn draw_cell(framebuffer: &mut Vec<u32>, xo: usize, yo: usize, block_size: usize, cell: char, row: usize, col: usize) {
    let color = match cell {
        '+' | '-' | '|' => {
            if (row + col) % 2 == 0 {
                0xFF5733 // Color 1 (naranja)
            } else {
                0x3498DB // Color 2 (azul)
            }
        }
        'p' => 0xFF0000, // Rojo para el punto de inicio
        'g' => 0x00FF00, // Verde para el punto de meta
        _ => 0x000000,   // Negro para el espacio vacío
    };

    for y in yo..(yo + block_size).min(HEIGHT) {
        for x in xo..(xo + block_size).min(WIDTH) {
            framebuffer[y * WIDTH + x] = color;
        }
    }
}

fn render_floor(framebuffer: &mut Vec<u32>) {
    for y in (HEIGHT / 2)..HEIGHT {
        let ty = (y - HEIGHT / 2) * FLOOR.height as usize / (HEIGHT / 2);

        for x in 0..WIDTH {
            let tx = x * FLOOR.width as usize / WIDTH;
            let color = FLOOR.get_pixel_color(tx as u32, ty as u32);
            framebuffer[y * WIDTH + x] = color;
        }
    }
}

fn render_minimap(framebuffer: &mut Vec<u32>, maze: &Vec<Vec<char>>, player: &Player, block_size: usize, enemy_positions: &Vec<na::Vector2<f32>>) {
    let minimap_scale = 20;
    let minimap_width = maze[0].len() * minimap_scale;
    let minimap_height = maze.len() * minimap_scale;

    // Posición del minimapa en la pantalla (esquina superior izquierda)
    let minimap_x_offset = 10;
    let minimap_y_offset = 10;

    // Dibujar el laberinto en el minimapa
    for (row, line) in maze.iter().enumerate() {
        for (col, &cell) in line.iter().enumerate() {
            let color = match cell {
                '+' | '-' | '|' => 0xFFFFFF, // Color blanco para las paredes
                'p' => 0xFF0000, // Rojo para el punto de inicio
                'g' => 0x00FF00, // Verde para el punto de meta
                _ => 0x000000,   // Negro para el espacio vacío
            };

            for y in 0..minimap_scale {
                for x in 0..minimap_scale {
                    let pixel_x = minimap_x_offset + col * minimap_scale + x;
                    let pixel_y = minimap_y_offset + row * minimap_scale + y;
                    if pixel_x < WIDTH && pixel_y < HEIGHT {
                        framebuffer[pixel_y * WIDTH + pixel_x] = color;
                    }
                }
            }
        }
    }

    // Dibujar al jugador en el minimapa
    let player_minimap_x = minimap_x_offset + (player.pos.x * minimap_scale as f32) as usize;
    let player_minimap_y = minimap_y_offset + (player.pos.y * minimap_scale as f32) as usize;

    let player_minimap_size = 8;

    for y in 0..player_minimap_size {
        for x in 0..player_minimap_size {
            let pixel_x = player_minimap_x + x;
            let pixel_y = player_minimap_y + y;
            if pixel_x < WIDTH && pixel_y < HEIGHT {
                framebuffer[pixel_y * WIDTH + pixel_x] = 0xFF0000;
            }
        }
    }

    /* Dibujar los enemigos en el minimapa
    for enemy_pos in enemy_positions {
        let enemy_minimap_x = minimap_x_offset + (enemy_pos.x * minimap_scale as f32) as usize;
        let enemy_minimap_y = minimap_y_offset + (enemy_pos.y * minimap_scale as f32) as usize;

        for y in 0..player_minimap_size {
            for x in 0..player_minimap_size {
                let pixel_x = enemy_minimap_x + x;
                let pixel_y = enemy_minimap_y + y;
                if pixel_x < WIDTH && pixel_y < HEIGHT {
                    framebuffer[pixel_y * WIDTH + pixel_x] = 0x800080;
                }
            }
        }
    }*/
}

fn render_enemy(framebuffer: &mut Vec<u32>, player: &Player, pos: &na::Vector2<f32>, z_buffer: &mut [f32]) {
    let sprite_dir = na::Vector2::new(
        pos.x - player.pos.x,
        pos.y - player.pos.y,
    );

    let sprite_d = sprite_dir.norm();
    let sprite_angle = (sprite_dir.y).atan2(sprite_dir.x) - player.a;

    // Normalizar el ángulo
    let sprite_angle = if sprite_angle < -std::f32::consts::PI {
        sprite_angle + 2.0 * std::f32::consts::PI
    } else if sprite_angle > std::f32::consts::PI {
        sprite_angle - 2.0 * std::f32::consts::PI
    } else {
        sprite_angle
    };

    // Si el sprite está fuera del campo de visión, no renderizar
    if sprite_angle.abs() > player.fov / 2.0 || sprite_d < 0.5 {
        return;
    }

    let screen_x = (WIDTH as f32 / 2.0) * (1.0 + sprite_angle / player.fov);
    let sprite_height = (HEIGHT as f32 / sprite_d) * 0.4; // Ajuste de tamaño del sprite
    let sprite_width = sprite_height;

    let start_x = screen_x as isize - (sprite_width as isize / 2);
    let start_y = (HEIGHT as isize / 2) - (sprite_height as isize / 2);
    let end_x = start_x + sprite_width as isize;
    let end_y = start_y + sprite_height as isize;

    if start_x >= 0 && end_x < WIDTH as isize && sprite_d < z_buffer[screen_x as usize] {
        for x in start_x..end_x {
            for y in start_y..end_y {
                if x >= 0 && x < WIDTH as isize && y >= 0 && y < HEIGHT as isize {
                    let x = x as usize;
                    let y = y as usize;

                    let tx = ((x - start_x as usize) * CHARACTER.width as usize / sprite_width as usize) as u32;
                    let ty = ((y - start_y as usize) * CHARACTER.height as usize / sprite_height as usize) as u32;
                    let color = CHARACTER.get_pixel_color(tx, ty);

                    // Solo dibujar si el color no es igual al del fondo (asumimos que el fondo es blanco)
                    if color != 0xFFFFFF { // Ajusta este valor si el color de fondo es diferente
                        framebuffer[y * WIDTH + x] = color;
                    }
                }
            }
        }
        z_buffer[screen_x as usize] = sprite_d;
    }
}

fn render_enemies(framebuffer: &mut Vec<u32>, player: &Player, z_buffer: &mut [f32]) {
    let enemies = vec![
        na::Vector2::new(2.0, 5.0),
        na::Vector2::new(11.0, 3.5),
        na::Vector2::new(5.0, 5.0),
        na::Vector2::new(8.0, 7.0),
    ];

    for enemy in &enemies {
        render_enemy(framebuffer, player, enemy, z_buffer);
    }
}

fn render2d(framebuffer: &mut Vec<u32>, maze: &Vec<Vec<char>>, block_size: usize, player: &Player) {
    for (row, line) in maze.iter().enumerate() {
        for (col, &cell) in line.iter().enumerate() {
            draw_cell(
                framebuffer,
                col * block_size,
                row * block_size,
                block_size,
                cell,
                row,
                col,
            );
        }
    }

    // Dibujar al jugador en la vista 2D
    let player_x = (player.pos.x * block_size as f32) as usize;
    let player_y = (player.pos.y * block_size as f32) as usize;
    let player_size = block_size / 4; // Tamaño del punto que representa al jugador

    for y in player_y..(player_y + player_size).min(HEIGHT) {
        for x in player_x..(player_x + player_size).min(WIDTH) {
            framebuffer[y * WIDTH + x] = 0xFF0000; // Rojo para representar al jugador
        }
    }
}

fn render3d(framebuffer: &mut Vec<u32>, maze: &Vec<Vec<char>>, player: &Player, block_size: f32, z_buffer: &mut [f32], enemy_positions: &Vec<na::Vector2<f32>>) {
    render_sky(framebuffer);
    render_floor(framebuffer);
    
    let num_rays = WIDTH;
    let hh = (HEIGHT / 2) as f32;

    for i in 0..num_rays {
        let current_ray = i as f32 / num_rays as f32;
        let a = player.a - (player.fov / 2.0) + (player.fov * current_ray);

        // Llamada a la función cast_ray
        let ray_hit = cast_ray(maze, player, a);

        // Calcular la altura de la stake basada en la distancia
        let stake_height = (hh / ray_hit.distance) as usize;

        // Calcular las posiciones superior e inferior de la stake
        let mut stake_top = (hh - (stake_height as f32 / 2.0)) as usize;
        let mut stake_bottom = (hh + (stake_height as f32 / 2.0)) as usize;

        // Calcular la coordenada de la textura en función del punto de impacto
        let texture_x = if ray_hit.is_vertical {
            ray_hit.hit_y % 1.0
        } else {
            ray_hit.hit_x % 1.0
        };

        // Limitar los valores dentro del rango del framebuffer
        if stake_top >= HEIGHT {
            stake_top = HEIGHT - 1;
        }
        if stake_bottom >= HEIGHT {
            stake_bottom = HEIGHT - 1;
        }

        // Dibujar la stake directamente en el framebuffer
        for y in stake_top..stake_bottom {
            let texture_y = (y as f32 - stake_top as f32) / (stake_bottom as f32 - stake_top as f32);
            let color = cell_to_texture_color(ray_hit.wall_type, ray_hit.is_vertical, texture_x, texture_y);
            framebuffer[y * WIDTH + i] = color;
        }

        // Renderizar el piso
        for y in stake_bottom..HEIGHT {
            let current_distance = hh / (y as f32 - hh);

            let weight = current_distance / ray_hit.distance;

            let floor_x = weight * ray_hit.hit_x + (1.0 - weight) * player.pos.x;
            let floor_y = weight * ray_hit.hit_y + (1.0 - weight) * player.pos.y;

            let texture_x = (floor_x * FLOOR.width as f32) as u32 % FLOOR.width;
            let texture_y = (floor_y * FLOOR.height as f32) as u32 % FLOOR.height;

            let color = FLOOR.get_pixel_color(texture_x, texture_y);
            framebuffer[y * WIDTH + i] = color;
        }

        z_buffer[i] = ray_hit.distance;
    }
    // Renderizar los enemigos
    render_enemies(framebuffer, player, z_buffer);

    // Llamar al render_minimap
    render_minimap(framebuffer, maze, player, block_size as usize, enemy_positions);
}

fn main() {
    let mut window = Window::new(
        "Maze",
        WIDTH,
        HEIGHT,
        WindowOptions::default(),
    )
    .unwrap_or_else(|e| {
        panic!("{}", e);
    });

    let enemy_positions = vec![
        na::Vector2::new(2.0, 5.0),
        na::Vector2::new(11.0, 3.5),
        na::Vector2::new(5.0, 5.0),
        na::Vector2::new(8.0, 7.0),
    ];

    let maze = maze::load_maze("maze.txt");

    let mut framebuffer: Vec<u32> = vec![0; WIDTH * HEIGHT];
    let block_size = 80;

    let mut player = Player {
        pos: na::Vector2::new(1.5, 1.5),
        a: std::f32::consts::FRAC_PI_3,
        fov: std::f32::consts::FRAC_PI_3,
    };

    let mut mode = "3D";

    let mut last_time = Instant::now();
    let mut frame_count = 0;
    let mut fps_text = String::new();

    while window.is_open() && !window.is_key_down(Key::Escape) {
        let frame_start_time = Instant::now();

        process_events(&window, &mut player, &maze, block_size);

        // Limpiar el framebuffer
        framebuffer.iter_mut().for_each(|pixel| *pixel = 0);
        let mut z_buffer: Vec<f32> = vec![std::f32::MAX; WIDTH];

        // Renderizar según el modo
        if mode == "2D" {
            render2d(&mut framebuffer, &maze, block_size, &player);
        } else {
            render3d(&mut framebuffer, &maze, &player, block_size as f32, &mut z_buffer, &enemy_positions);
        }

        // Calcular FPS
        frame_count += 1;
        let current_time = Instant::now();
        let elapsed = current_time.duration_since(last_time);

        if elapsed >= std::time::Duration::from_secs(1) {
            let fps = frame_count as f64 / elapsed.as_secs_f64();
            fps_text = format!("FPS: {:.0}", fps);
            last_time = current_time;
            frame_count = 0;
        }

        // Dibujar FPS en la esquina superior derecha
        let scale = Scale::uniform(24.0);
        let text_width = 200;
        let fps_x = WIDTH.saturating_sub(text_width) - 10;
        let fps_y = 10;
        render_text(&mut framebuffer, &fps_text, fps_x, fps_y, scale, 0x000000);

        window
            .update_with_buffer(&framebuffer, WIDTH, HEIGHT).unwrap();

        if window.is_key_down(Key::M) {
            mode = if mode == "2D" { "3D" } else { "2D" };
        }

        let frame_end_time = Instant::now();
        let frame_duration_actual = frame_end_time.duration_since(frame_start_time);
        if frame_duration_actual < std::time::Duration::from_millis(16) {
            let sleep_duration = std::time::Duration::from_millis(16) - frame_duration_actual;
            if sleep_duration > std::time::Duration::from_millis(0) {
                std::thread::sleep(sleep_duration);
            }
        }
    }
}
