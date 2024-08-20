use crate::player::Player;

pub struct RayHit {
    pub distance: f32,      // Distancia desde el jugador hasta la pared
    pub hit_x: f32,         // Coordenada x del punto de impacto
    pub hit_y: f32,         // Coordenada y del punto de impacto
    pub wall_type: char,    // Símbolo de la pared golpeada ('-' o '|')
    pub is_vertical: bool,  // Indica si la pared golpeada es vertical
}

pub fn cast_rays(
    framebuffer: &mut Vec<u32>,
    maze: &Vec<Vec<char>>,
    player: &Player,
    block_size: usize,
    screen_width: usize,
) {
    let num_rays = screen_width;
    let delta_angle = player.fov / num_rays as f32;

    for i in 0..num_rays {
        let ray_angle = player.a - (player.fov / 2.0) + (i as f32) * delta_angle;
        cast_ray(maze, player, ray_angle);
    }
}

pub fn cast_ray(
    maze: &Vec<Vec<char>>,
    player: &Player,
    angle: f32
) -> RayHit {
    let sin_a = angle.sin();
    let cos_a = angle.cos();

    let x = player.pos.x;
    let y = player.pos.y;

    let delta_dist_x = (1.0 / cos_a).abs();
    let delta_dist_y = (1.0 / sin_a).abs();

    let mut map_x = x.floor() as isize;
    let mut map_y = y.floor() as isize;

    let step_x = if cos_a >= 0.0 { 1 } else { -1 };
    let step_y = if sin_a >= 0.0 { 1 } else { -1 };

    let mut side_dist_x = if cos_a >= 0.0 {
        (map_x as f32 + 1.0 - x) * delta_dist_x
    } else {
        (x - map_x as f32) * delta_dist_x
    };

    let mut side_dist_y = if sin_a >= 0.0 {
        (map_y as f32 + 1.0 - y) * delta_dist_y
    } else {
        (y - map_y as f32) * delta_dist_y
    };

    let mut hit = false;
    let mut wall_type = ' ';
    let mut is_vertical = false;
    let mut distance = 0.0;
    let mut hit_x = 0.0;
    let mut hit_y = 0.0;

    while !hit {
        if side_dist_x < side_dist_y {
            side_dist_x += delta_dist_x;
            map_x += step_x;
            is_vertical = true;
        } else {
            side_dist_y += delta_dist_y;
            map_y += step_y;
            is_vertical = false;
        }

        if map_y < 0 || map_y >= maze.len() as isize || map_x < 0 || map_x >= maze[0].len() as isize {
            break;
        }

        wall_type = maze[map_y as usize][map_x as usize];
        if wall_type != ' ' {
            hit = true;
            if is_vertical {
                distance = (map_x as f32 - x + (1.0 - step_x as f32) / 2.0) / cos_a;
                hit_x = map_x as f32;
                hit_y = y + distance * sin_a;
            } else {
                distance = (map_y as f32 - y + (1.0 - step_y as f32) / 2.0) / sin_a;
                hit_x = x + distance * cos_a;
                hit_y = map_y as f32;
            }
        }
    }

    RayHit {
        distance,
        hit_x,
        hit_y,
        wall_type,
        is_vertical,
    }
}