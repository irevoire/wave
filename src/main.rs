use minifb::{Key, Window, WindowOptions};
use pastel::{Color, Fraction, HSLA};
use rand::Rng;
use serde::Deserialize;
use std::{ops::Range, time::Duration};

#[derive(Debug, Clone, Copy, Deserialize)]
struct Coord {
    x: f32,
    y: f32,
}

impl Coord {
    pub fn new(x: impl TryInto<f32>, y: impl TryInto<f32>) -> Self {
        Self {
            x: x.try_into().map_err(|_| ()).unwrap(),
            y: y.try_into().map_err(|_| ()).unwrap(),
        }
    }

    pub fn rand(x: Range<f32>, y: Range<f32>) -> Self {
        let mut rng = rand::thread_rng();
        let x = rng.gen_range(x);
        let y = rng.gen_range(y);
        Self::new(x, y)
    }

    pub fn squared_distance(&self, other: &Self) -> f32 {
        (self.x - other.x).powi(2) + (self.y - other.y).powi(2)
    }
}

#[derive(Debug, Clone, Copy, Deserialize)]
pub struct Particle {
    coord: Coord,
}
impl Particle {
    fn new(x: f32, y: f32) -> Self {
        Self {
            coord: Coord::new(x, y),
        }
    }
    pub fn rand(x: Range<f32>, y: Range<f32>) -> Self {
        Self {
            coord: Coord::rand(x, y),
        }
    }
}

#[derive(Debug, Clone, Copy, Deserialize)]
pub struct Canvas {
    /// in cm
    width: f32,
    /// in cm
    height: f32,

    /// Speaker position
    speaker: Coord,
}

pub struct World {
    pub canvas: Canvas,
    particles: Vec<Particle>,
}

impl World {
    pub fn new(canvas: Canvas) -> Self {
        let particles: Vec<_> = (0..canvas.width as usize)
            .flat_map(|x| {
                (0..canvas.height as usize).flat_map(move |y| {
                    // (0..PARTICLES_PER_CM).map(move |_| Particle::new(x as f32, y as f32))
                    (0..PARTICLES_PER_CM)
                        .map(move |_| Particle::rand(0.0..canvas.width, 0.0..canvas.height))
                })
            })
            .collect();
        println!("World starting with {} particles", particles.len());

        Self { particles, canvas }
    }

    pub fn next_iteration(&mut self) {
        let mut closest: Vec<(f32, Particle)> = Vec::new();

        for current in 0..self.particles.len() {
            closest.clear();

            let particle = self.particles[current];
            // find the closest 7 particles
            for part2 in self.particles.iter() {
                let distance = particle.coord.squared_distance(&part2.coord);
                if closest.len() == 7 {
                    let min = closest
                        .iter_mut()
                        .max_by(|(dist1, _), (dist2, _)| dist1.total_cmp(dist2))
                        .unwrap();
                    *min = (distance, *part2);
                } else {
                    closest.push((distance, *part2));
                }
            }

            let particle = &mut self.particles[current];
            // if something is too close move to the opposite direction by the same distance?
            if closest[0].0 < 0.01 {
                let closest = closest[0].1;
                // we can’t go outside of the canvas
                particle.coord.x = (particle.coord.x + (closest.coord.x - particle.coord.x))
                    .clamp(0.0, self.canvas.width);
                particle.coord.y = (particle.coord.y + (closest.coord.y - particle.coord.y))
                    .clamp(0.0, self.canvas.width);
                // println!("moved someone");
            }
        }
    }

    pub fn add_heatmap(&self, width: usize, height: usize, buffer: &mut [u32]) -> u32 {
        let mut max_heat = 0;

        for particle in self.particles.iter() {
            // scale x/y on the original canvas
            let x = (particle.coord.x / self.canvas.width * width as f32) as usize;
            let y = (particle.coord.y / self.canvas.height * height as f32) as usize;

            // println!("one on ({x}, {y})");
            let cell = &mut buffer[y * width + x];
            *cell += 1;

            max_heat = max_heat.max(*cell);
        }

        max_heat
    }
}

const PARTICLES_PER_CM: usize = 10000;
const PARTICLES_SIZE: usize = 1000;
const WIDTH: usize = 1024;
const HEIGHT: usize = 800;

fn main() {
    let file = std::fs::read_to_string("spec.toml").unwrap();
    let canvas: Canvas = toml::de::from_str(&file).unwrap();
    let mut world = World::new(canvas);

    let mut buffer: Vec<u32> = vec![0; WIDTH * HEIGHT];

    let mut window = Window::new(
        "Test - ESC to exit",
        WIDTH,
        HEIGHT,
        WindowOptions::default(),
    )
    .unwrap_or_else(|e| {
        panic!("{}", e);
    });

    // Limit to max ~30 fps update rate
    window.limit_update_rate(Some(Duration::from_secs(1) / 30));

    while window.is_open() && !window.is_key_down(Key::Escape) {
        let now = std::time::Instant::now();
        buffer.fill(0);

        world.next_iteration();
        let max_heat = world.add_heatmap(WIDTH, HEIGHT, &mut buffer);
        colorize(max_heat, &mut buffer);

        println!("computed frame in {:?}", now.elapsed());

        // We unwrap here as we want this code to exit if it fails. Real applications may want to handle this in a different way
        window.update_with_buffer(&buffer, WIDTH, HEIGHT).unwrap();
    }
}

fn colorize(max_value: u32, buffer: &mut [u32]) {
    for pixel in buffer.iter_mut() {
        *pixel = match *pixel {
            0 => Color::aqua().to_u32(),
            1 => Color::blue().to_u32(),
            _ => Color::red().to_u32(),
        };
        // let normal = *pixel as f64 / max_value as f64;
        // let color: Color = Color::aqua().mix::<HSLA>(&Color::red(), Fraction::from(normal * 360.));
        // *pixel = color.to_u32();
    }
}
