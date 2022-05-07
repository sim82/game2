use bevy::prelude::*;

pub mod auto_collider;
pub mod fx;
pub mod hex;
pub mod shape {
    use bevy::{
        prelude::*,
        render::mesh::{Indices, PrimitiveTopology},
    };

    pub struct HexPlane {
        pub w: f32,
        pub h: f32,
        pub e: f32,
    }

    impl From<HexPlane> for Mesh {
        fn from(plane: HexPlane) -> Self {
            // let extent = plane.size / 2.0;

            let h2 = plane.h / 2.0;
            let h4 = plane.h / 4.0;
            let w2 = plane.w / 2.0;
            let e2 = plane.e / 2.0;
            let o = 7;
            let vertices = [
                ([0.0, e2, 0.0], [0.0, 1.0, 0.0], [1.0, 1.0]),
                ([0.0, e2, h2], [0.0, 1.0, 0.0], [1.0, 1.0]),
                ([w2, e2, h4], [0.0, 1.0, 0.0], [1.0, 0.0]),
                ([w2, e2, -h4], [0.0, 1.0, 0.0], [0.0, 0.0]),
                ([0.0, e2, -h2], [0.0, 1.0, 0.0], [0.0, 1.0]),
                ([-w2, e2, -h4], [0.0, 1.0, 0.0], [0.0, 1.0]),
                ([-w2, e2, h4], [0.0, 1.0, 0.0], [0.0, 1.0]),
                ([0.0, -e2, 0.0], [0.0, -1.0, 0.0], [1.0, 1.0]),
                ([0.0, -e2, h2], [0.0, -1.0, 0.0], [1.0, 1.0]),
                ([w2, -e2, h4], [0.0, -1.0, 0.0], [1.0, 0.0]),
                ([w2, -e2, -h4], [0.0, -1.0, 0.0], [0.0, 0.0]),
                ([0.0, -e2, -h2], [0.0, -1.0, 0.0], [0.0, 1.0]),
                ([-w2, -e2, -h4], [0.0, -1.0, 0.0], [0.0, 1.0]),
                ([-w2, -e2, h4], [0.0, -1.0, 0.0], [0.0, 1.0]),
            ];

            let upper = [0, 1, 2, 0, 2, 3, 0, 3, 4, 0, 4, 5, 0, 5, 6, 0, 6, 1];
            let lower = [0, 2, 1, 0, 3, 2, 0, 4, 3, 0, 5, 4, 0, 6, 5, 0, 1, 6];
            let offs = [1, 2, 3, 4, 5];

            let side = [
                // 1
                1, 0, 8, 8, 0, 7,
            ];
            let side6 = [1, 6, 8, 8, 6, 13];
            let sides = offs.iter().flat_map(|offs| side.iter().map(|i| *i + *offs));

            let indices = Indices::U32(
                upper
                    .iter()
                    .cloned()
                    .chain(lower.iter().map(|p| *p + o))
                    .chain(sides)
                    .chain(side6)
                    .collect(),
            );

            let mut positions = Vec::new();
            let mut normals = Vec::new();
            let mut uvs = Vec::new();
            for (position, normal, uv) in &vertices {
                positions.push(*position);
                normals.push(*normal);
                uvs.push(*uv);
            }

            let mut mesh = Mesh::new(PrimitiveTopology::TriangleList);
            mesh.set_indices(Some(indices));
            mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, positions);
            mesh.insert_attribute(Mesh::ATTRIBUTE_NORMAL, normals);
            mesh.insert_attribute(Mesh::ATTRIBUTE_UV_0, uvs);
            mesh
        }
    }
}

pub const COLORSX: [Color; 18] = [
    Color::PINK,
    Color::CRIMSON,
    Color::AQUAMARINE,
    Color::AZURE,
    Color::BLUE,
    Color::CYAN,
    Color::FUCHSIA,
    Color::GOLD,
    Color::GREEN,
    Color::INDIGO,
    Color::LIME_GREEN,
    Color::ORANGE,
    Color::ORANGE_RED,
    Color::PURPLE,
    Color::TURQUOISE,
    Color::VIOLET,
    Color::YELLOW,
    Color::YELLOW_GREEN,
];

pub const L: f32 = 0.75;

pub const COLORS: [Color; 12] = [
    Color::hsl(0.0, 1.0, L),
    Color::hsl(30.0, 1.0, L),
    Color::hsl(60.0, 1.0, L),
    Color::hsl(90.0, 1.0, L),
    Color::hsl(120.0, 1.0, L),
    Color::hsl(150.0, 1.0, L),
    Color::hsl(180.0, 1.0, L),
    Color::hsl(210.0, 1.0, L),
    Color::hsl(240.0, 1.0, L),
    Color::hsl(270.0, 1.0, L),
    Color::hsl(300.0, 1.0, L),
    Color::hsl(330.0, 1.0, L),
];
