use cgmath::InnerSpace;

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
        let epsilon = 1e-6;

        // Check if the ray is parallel to the triangle
        let dot_normal_dir = self.normal.dot(dir);
        if dot_normal_dir.abs() < epsilon {
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