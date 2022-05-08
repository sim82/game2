use bevy::{
    prelude::{Component, Vec2},
    reflect::Reflect,
};
use num_traits::Num;

// pub mod editor;
// pub mod io;
// pub mod tilemap;
// pub mod wavefunction;

// mostly based on https://www.redblobgames.com/grids/hexagons/
#[derive(Default, Debug, Clone, Copy, Hash, PartialEq, Eq, Reflect, Component)]
pub struct HexCube {
    pub x: i32,
    pub y: i32,
    pub z: i32,
}

impl HexCube {
    pub fn new(x: i32, y: i32, z: i32) -> HexCube {
        HexCube { x, y, z }
    }
    pub fn zero() -> HexCube {
        HexCube::default()
    }
    pub fn to_odd_r_screen(self) -> Vec2 {
        // convert to odd-r coordinates, but already shifted to on screen rendering:
        //  - row height is consolidated to 0.75
        //  - odd rows are shifted 0.5 to the right
        let oddr = self.to_odd_r();
        let shift = (self.z & 1) as f32 * 0.5;
        Vec2::new(oddr.x + shift, oddr.y * 0.75)
    }

    pub fn to_odd_r(self) -> Vec2 {
        let col = self.x + (self.z - (self.z & 1)) / 2;
        let row = self.z;

        Vec2::new(col as f32, row as f32)
    }

    pub fn from_odd_r(v: Vec2) -> HexCube {
        let vx = v.x as i32;
        let vy = v.y as i32;

        let x = vx - (vy - (vy & 1)) / 2;
        let z = vy;
        let y = -x - z;
        HexCube { x, y, z }
    }

    pub fn from_odd_r_screen(v: Vec2) -> HexCube {
        let major_y = (v.y / 0.75).floor();
        let shift = (major_y as i32 & 1) as f32 * 0.5;

        let major_x = (v.x - shift).floor();

        let vx = major_x as i32;
        let vy = major_y as i32;

        let x = vx - (vy - (vy & 1)) / 2;
        let z = vy;
        let y = -x - z;
        HexCube { x, y, z }
    }

    pub fn lerp_between(a: &HexCube, b: &HexCube, t: f32) -> HexCube {
        HexCube {
            x: lerp(a.x as f32, b.x as f32, t) as i32,
            y: lerp(a.y as f32, b.y as f32, t) as i32,
            z: lerp(a.z as f32, b.z as f32, t) as i32,
        }
    }

    pub fn round(x: f32, y: f32, z: f32) -> HexCube {
        let mut rx = x.round();
        let mut ry = y.round();
        let mut rz = z.round();

        let x_diff = (rx - x).abs();
        let y_diff = (ry - y).abs();
        let z_diff = (rz - z).abs();

        if x_diff > y_diff && x_diff > z_diff {
            rx = -ry - rz
        } else if y_diff > z_diff {
            ry = -rx - rz
        } else {
            rz = -rx - ry
        }

        HexCube {
            x: rx as i32,
            y: ry as i32,
            z: rz as i32,
        }
    }

    pub fn distance_between(a: &HexCube, b: &HexCube) -> i32 {
        (a.x - b.x).abs() + (a.y - b.y).abs() + (a.z - b.z).abs() / 2
    }
    pub fn distance(&self, b: &HexCube) -> i32 {
        Self::distance_between(self, b)
    }

    pub fn linedraw_between(a: &HexCube, b: &HexCube) -> (i32, [HexCube; 20]) {
        let n = Self::distance_between(a, b);
        let mut res = [HexCube::default(); 20];
        for i in 0..n.max(20) {
            let mut c = Self::lerp_between(a, b, 1f32 / n as f32 * i as f32);
            c.y = -c.x - c.z;
            res[i as usize] = c;
        }
        (n.max(20), res)
    }
}

// impl From<&Cube> for Cube {
//     fn from(c: &Cube) -> Self {
//         *c
//     }
// }

impl From<HexAxial> for HexCube {
    fn from(h: HexAxial) -> Self {
        HexCube {
            x: h.q,
            y: -h.q - h.r,
            z: h.r,
        }
    }
}

impl ops::Add for HexCube {
    type Output = HexCube;
    fn add(self, rhs: Self) -> Self::Output {
        HexCube::new(self.x + rhs.x, self.y + rhs.y, self.z + rhs.z)
    }
}

impl ops::Sub for HexCube {
    type Output = HexCube;
    fn sub(self, rhs: Self) -> Self::Output {
        HexCube::new(self.x - rhs.x, self.y - rhs.y, self.z - rhs.z)
    }
}

impl ops::AddAssign for HexCube {
    fn add_assign(&mut self, rhs: Self) {
        self.x += rhs.x;
        self.y += rhs.y;
        self.z += rhs.z;
    }
}

impl ops::Mul<i32> for HexCube {
    type Output = HexCube;

    fn mul(self, rhs: i32) -> Self::Output {
        HexCube::new(self.x * rhs, self.y * rhs, self.z * rhs)
    }
}

impl ops::MulAssign<i32> for HexCube {
    fn mul_assign(&mut self, rhs: i32) {
        self.x *= rhs;
        self.y *= rhs;
        self.z *= rhs;
    }
}

impl From<(i32, i32, i32)> for HexCube {
    fn from(v: (i32, i32, i32)) -> Self {
        HexCube::new(v.0, v.1, v.2)
    }
}

#[derive(Reflect, Default, Debug, Clone, Copy, Hash, PartialEq, Eq)]
pub struct HexAxial {
    pub q: i32,
    pub r: i32,
}

impl HexAxial {
    pub fn to_odd_r(&self) -> Vec2 {
        // let col = self.q as f32 + (self.r - (self.r & 1)) as f32 * 0.5;
        let col = (self.q + self.r) as f32 - (self.r & 1) as f32 * 0.5;
        let row = self.r as f32 * 0.75;
        Vec2::new(col as f32, row as f32)
    }
    pub fn from_odd_r(v: Vec2) -> Self {
        let vx = v.x as i32;
        let vy = v.y as i32;
        let q = vx - (vy - (vy & 1)) / 2;
        let r = vy;
        Self { q, r }
    }
}

// impl From<Vec2> for Hex {
//     fn from(v: Vec2) -> Self {
//         let vx = v.x as i32;
//         let vy = v.y as i32;
//         let q = vx - (vy - (vy & 1)) / 2;
//         let r = vy;
//         Self { q, r }
//     }
// }

impl From<HexCube> for HexAxial {
    fn from(c: HexCube) -> Self {
        HexAxial { q: c.x, r: c.z }
    }
}

use std::ops;

pub const HEX_CUBE_DIRECTIONS: [HexCube; 6] = [
    HexCube { x: 1, y: -1, z: 0 },
    HexCube { x: 1, y: 0, z: -1 },
    HexCube { x: 0, y: 1, z: -1 },
    HexCube { x: -1, y: 1, z: 0 },
    HexCube { x: -1, y: 0, z: 1 },
    HexCube { x: 0, y: -1, z: 1 },
];

fn lerp<T: Num + Copy>(a: T, b: T, t: T) -> T {
    a + (b - a) * t
}

pub struct CubeLinedraw {
    a: HexCube,
    b: HexCube,
    n: i32,
    i: i32,
}

impl CubeLinedraw {
    pub fn new(a: HexCube, b: HexCube) -> Self {
        let n = HexCube::distance_between(&a, &b);
        CubeLinedraw { a, b, n, i: 0 }
    }
}

impl Iterator for CubeLinedraw {
    type Item = HexCube;

    fn next(&mut self) -> Option<Self::Item> {
        if self.i >= self.n {
            None
        } else {
            let t = 1f32 / self.n as f32 * self.i as f32;
            let x = lerp(self.a.x as f32, self.b.x as f32, t);
            let y = lerp(self.a.y as f32, self.b.y as f32, t);
            let z = lerp(self.a.z as f32, self.b.z as f32, t);
            self.i += 1;
            Some(HexCube::round(x, y, z))
        }
    }
}

pub mod prelude {
    pub use super::{HexAxial, HexCube};
}
