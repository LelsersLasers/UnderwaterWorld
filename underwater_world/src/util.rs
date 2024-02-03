use cgmath::InnerSpace;

const EPSILON: f32 = 1e-5;


#[derive(Clone, Copy)]
pub struct Tri {
    pub verts: [cgmath::Vector3<f32>; 3],
    pub normal: cgmath::Vector3<f32>,
}
impl Tri {
    pub fn new(verts: [[f32; 3]; 3]) -> Self {
        let a = cgmath::Vector3::from(verts[0]);
        let b = cgmath::Vector3::from(verts[1]);
        let c = cgmath::Vector3::from(verts[2]);
        let normal = safe_normalize((b - a).cross(c - a));
        Self {
            verts: [a, b, c],
            normal,
        }
    }
    pub fn intersects(&self, pos_other: cgmath::Vector3<f32>, dir: cgmath::Vector3<f32>, range: f32) -> Option<f32> {
        // https://chat.openai.com/share/e19d45df-2288-4889-8ece-5d0c98d67701

        // Check if the ray is parallel to the triangle
        let dot_normal_dir = self.normal.dot(dir);
        if dot_normal_dir.abs() < EPSILON {
            return None;
        }

        // Compute the intersection point
        let t = self.normal.dot(self.verts[0] - pos_other) / dot_normal_dir;

        // Check if the intersection point is within the range
        if t < 0.0 || t > range {
            return None;
        }

        // Check if the intersection point is inside the triangle
        let intersection_point = pos_other + dir * t;
        let edge0 = self.verts[1] - self.verts[0];
        let edge1 = self.verts[2] - self.verts[1];
        let edge2 = self.verts[0] - self.verts[2];

        let normal0 = edge0.cross(intersection_point - self.verts[0]);
        let normal1 = edge1.cross(intersection_point - self.verts[1]);
        let normal2 = edge2.cross(intersection_point - self.verts[2]);

        let dot0 = normal0.dot(self.normal);
        let dot1 = normal1.dot(self.normal);
        let dot2 = normal2.dot(self.normal);

        if dot0 >= 0.0 && dot1 >= 0.0 && dot2 >= 0.0 {
            Some(t)
        } else {
            None
        }
    }
}

pub fn safe_normalize(v: cgmath::Vector3<f32>) -> cgmath::Vector3<f32> {
    let mag = v.magnitude();
    if mag == 0.0 { v } else { v / mag }
}

pub fn safe_normalize_to(v: cgmath::Vector3<f32>, target: f32) -> cgmath::Vector3<f32> {
   safe_normalize(v) * target
}

pub fn dist_sq(pos1: (i32, i32, i32), pos2: (i32, i32, i32)) -> i32 {
    let dx = pos1.0 - pos2.0;
    let dy = pos1.1 - pos2.1;
    let dz = pos1.2 - pos2.2;
    dx * dx + dy * dy + dz * dz
}

pub fn in_frustum(pt: cgmath::Vector3<f32>, view_proj: cgmath::Matrix4<f32>) -> bool {
    let corner = view_proj * pt.extend(1.0);
    let corner = corner / corner.w;

    corner.x.abs() <= 1.0
        && corner.y.abs() <= 1.0
        && corner.z >= 0.0
        && corner.z <= 1.0
}

pub fn vec3_eq(a: cgmath::Vector3<f32>, b: cgmath::Vector3<f32>) -> bool {
    (a.x - b.x).abs() < EPSILON
        && (a.y - b.y).abs() < EPSILON
        && (a.z - b.z).abs() < EPSILON
}

pub fn create_mix_ratio(min: f32, max: f32, x: f32) -> f32 {
    (x - min) / (max - min)
}

pub fn mix_color(light: [f32; 3], dark: [f32; 3], ratio: f32) -> [f32; 3] {
    [
        light[0] * ratio + dark[0] * (1.0 - ratio),
        light[1] * ratio + dark[1] * (1.0 - ratio),
        light[2] * ratio + dark[2] * (1.0 - ratio),
    ]
}

// srgb_color = ((rgb_color / 255 + 0.055) / 1.055) ^ 2.4
pub fn to_srgb(color: [f32; 3]) -> [f32; 3] {
    [
        ((color[0] / 255.0 + 0.055) / 1.055).powf(2.4),
        ((color[1] / 255.0 + 0.055) / 1.055).powf(2.4),
        ((color[2] / 255.0 + 0.055) / 1.055).powf(2.4),
    ]
}

// https://github.com/jayber/hsv/blob/main/src/lib.rs
pub fn hsv_to_rgb(hue: f32, saturation: f32, value: f32) -> [f32; 3] {
    let hue = hue.rem_euclid(360.0);

    fn is_between(value: f32, min: f32, max: f32) -> bool {
        min <= value && value < max
    }

    let c = value * saturation;
    let h = hue / 60.0;
    let x = c * (1.0 - ((h % 2.0) - 1.0).abs());
    let m = value - c;

    let (r, g, b): (f32, f32, f32) = if is_between(h, 0.0, 1.0) {
        (c, x, 0.0)
    } else if is_between(h, 1.0, 2.0) {
        (x, c, 0.0)
    } else if is_between(h, 2.0, 3.0) {
        (0.0, c, x)
    } else if is_between(h, 3.0, 4.0) {
        (0.0, x, c)
    } else if is_between(h, 4.0, 5.0) {
        (x, 0.0, c)
    } else {
        (c, 0.0, x)
    };

    [
        ((r + m) * 255.0),
        ((g + m) * 255.0),
        ((b + m) * 255.0),
    ]
}
