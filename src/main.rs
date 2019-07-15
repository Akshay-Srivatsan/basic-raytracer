use png::HasParameters;
use std::f64::consts::PI;
use std::fs::File;
use std::io::BufWriter;
use std::path::Path;

#[derive(Debug, Copy, Clone)]
struct Vector {
    x: f64,
    y: f64,
    z: f64,
}

use std::ops;

impl Vector {
    fn new(x: f64, y: f64, z: f64) -> Vector {
        Vector { x, y, z }
    }

    fn len(&self) -> f64 {
        let len_squared = self.x.powi(2) + self.y.powi(2) + self.z.powi(2);
        len_squared.sqrt()
    }

    fn normalize(&self) -> Vector {
        let l = self.len();
        (1.0 / l) * self.clone()
    }
}

impl ops::Add<Vector> for Vector {
    type Output = Vector;

    fn add(self, other: Vector) -> Vector {
        Vector::new(self.x + other.x, self.y + other.y, self.z + other.z)
    }
}

impl ops::Sub<Vector> for Vector {
    type Output = Vector;

    fn sub(self, other: Vector) -> Vector {
        Vector::new(self.x - other.x, self.y - other.y, self.z - other.z)
    }
}

impl ops::Mul<Vector> for Vector {
    type Output = f64;

    fn mul(self, other: Vector) -> f64 {
        self.x * other.x + self.y * other.y + self.z * other.z
    }
}

impl ops::Mul<Vector> for f64 {
    type Output = Vector;

    fn mul(self, other: Vector) -> Vector {
        Vector::new(self * other.x, self * other.y, self * other.z)
    }
}

struct Ray {
    origin: Vector,
    direction: Vector,
}

impl Ray {
    fn new(origin: Vector, direction: Vector) -> Ray {
        Ray { origin, direction }
    }

    fn at(&self, length: f64) -> Vector {
        self.origin + length * self.direction
    }
}

trait Shape {
    fn intersect(&self, ray: &Ray) -> Option<f64>;
}

struct Sphere {
    center: Vector,
    radius: f64,
}

impl Sphere {
    fn new(center: Vector, radius: f64) -> Sphere {
        Sphere { center, radius }
    }
}

impl Shape for Sphere {
    fn intersect(&self, ray: &Ray) -> Option<f64> {
        let c = self.center;
        let r = self.radius;
        let l = ray.direction.normalize();
        let o = ray.origin;
        let diff = o - c;
        let dot = l * diff;
        let discriminant = dot.powi(2) - (diff.len().powi(2) - r.powi(2));
        if discriminant < 0.0 {
            None
        } else if discriminant == 0.0 {
            Some(-dot)
        } else {
            let a = -dot + discriminant;
            let b = -dot - discriminant;
            if a > b {
                Some(b)
            } else {
                Some(a)
            }
        }
    }
}

struct PointLight {
    source: Vector,
    color: Vector,
    intensity: f64
}

impl PointLight {
    fn new(source: Vector, color: Vector, intensity: f64) -> PointLight {
        PointLight {source, color, intensity}
    }

    fn illuminate(&self, point: Vector) -> Vector {
        (self.intensity / (point - self.source).len().powi(2)) * self.color
    }
}

fn raytrace() -> () {
    let sphere = Sphere::new(Vector::new(0.0, 0.0, -10.0), 1.0);
    let light_red = PointLight::new(Vector::new(2.0, 0.0, -9.0), Vector::new(1.0, 0.0, 0.0), 2.0);
    let light_green = PointLight::new(Vector::new(-2.0, 0.0, -9.0), Vector::new(0.0, 1.0, 0.0), 2.0);
    let light_blue = PointLight::new(Vector::new(0.0, -2.0, -9.0), Vector::new(0.0, 0.0, 1.0), 2.0);
    let shapes: Vec<&Shape> = vec![&sphere];
    let lights: Vec<&PointLight> = vec![&light_red, &light_green, &light_blue];

    const width: u32 = 640;
    const height: u32 = 480;
    const array_size: usize = (width * height * 4) as usize;
    let fov = 45.0 * PI / 180.0;

    let path = Path::new(r"output.png");
    let file = File::create(path).unwrap();
    let ref mut w = BufWriter::new(file);

    let mut encoder = png::Encoder::new(w, width, height);
    encoder.set(png::ColorType::RGBA).set(png::BitDepth::Eight);
    let mut writer = encoder.write_header().unwrap();

    let fov_y = (height as f64 * fov) / (width as f64);

    let mut data = [0; array_size];
    let origin = Vector::new(0.0, 0.0, 0.0);
    for x in 0..width {
        for y in 0..height {
            let i = ((x + y * width) * 4) as usize;
            let angle_x = ((x as f64) / (width as f64) - 0.5) * fov;
            let angle_y = -((y as f64) / (height as f64) - 0.5) * fov_y;
            let dx = angle_x.tan();
            let dy = angle_y.tan();
            let dz = -(1.0 - dx.powi(2) - dy.powi(2)).sqrt();
            let d = Vector::new(dx, dy, dz);
            let r = Ray::new(origin, d);

            let mut current_closest: Option<&Shape> = None;
            let mut current_closest_distance: Option<f64> = None;
            for &shape in &shapes {
                let distance = shape.intersect(&r);
                if let Some(q) = distance {
                    if current_closest_distance == None || q < current_closest_distance.unwrap() {
                        current_closest_distance = Some(q);
                        current_closest = Some(shape);
                    }
                }
            }

            if let Some(distance) = current_closest_distance {
                let mut color = Vector::new(0.0, 0.0, 0.0);
                for &light in &lights {
                    color = color + light.illuminate(r.at(distance));
                    color.x = if color.x > 1.0 {1.0} else {color.x};
                    color.y = if color.y > 1.0 {1.0} else {color.y};
                    color.z = if color.z > 1.0 {1.0} else {color.z};
                }
                data[i + 0] = (color.x * 255.0) as u8;
                data[i + 1] = (color.y * 255.0) as u8;
                data[i + 2] = (color.z * 255.0) as u8;
                data[i + 3] = 255;
            } else {
                data[i] = 0;
                data[i + 1] = 0;
                data[i + 2] = 0;
                data[i + 3] = 255;
            }
        }
    }
    writer.write_image_data(&data).unwrap();
}

fn main() {
    println!("Hello, world!");
    raytrace();
    println!("Raytraced successfully!");
}
