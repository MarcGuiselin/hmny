use super::*;
use bevy_math::{Quat, Vec2, Vec3};

#[derive(Clone, Decode, Encode, PartialEq, Debug)]
pub struct Quaternion {
    pub x: f32,
    pub y: f32,
    pub z: f32,
    pub w: f32,
}

impl Default for Quaternion {
    fn default() -> Self {
        Self {
            x: 0.0,
            y: 0.0,
            z: 0.0,
            w: 1.0,
        }
    }
}

impl Into<Quat> for Quaternion {
    fn into(self) -> Quat {
        Quat::from_xyzw(self.x, self.y, self.z, self.w)
    }
}

impl Into<Quaternion> for Quat {
    fn into(self) -> Quaternion {
        Quaternion {
            x: self.x,
            y: self.y,
            z: self.z,
            w: self.w,
        }
    }
}

#[derive(Clone, Decode, Encode, PartialEq, Debug)]
pub struct Position3D {
    pub x: f32,
    pub y: f32,
    pub z: f32,
}

impl Default for Position3D {
    fn default() -> Self {
        Self {
            x: 0.0,
            y: 0.0,
            z: 0.0,
        }
    }
}

impl Into<Vec3> for Position3D {
    fn into(self) -> Vec3 {
        Vec3 {
            x: self.x,
            y: self.y,
            z: self.z,
        }
    }
}

impl Into<Position3D> for Vec3 {
    fn into(self) -> Position3D {
        Position3D {
            x: self.x,
            y: self.y,
            z: self.z,
        }
    }
}

#[derive(Clone, Decode, Encode, PartialEq, Debug)]
pub struct Position2D {
    pub x: f32,
    pub y: f32,
}

impl Default for Position2D {
    fn default() -> Self {
        Self { x: 0.0, y: 0.0 }
    }
}

impl Into<Vec2> for Position2D {
    fn into(self) -> Vec2 {
        Vec2 {
            x: self.x,
            y: self.y,
        }
    }
}

impl Into<Position2D> for Vec2 {
    fn into(self) -> Position2D {
        Position2D {
            x: self.x,
            y: self.y,
        }
    }
}

#[derive(Clone, Decode, Encode, PartialEq, Debug)]
pub struct Location3D {
    pub rotation: Quaternion,
    pub position: Position3D,
}

impl Default for Location3D {
    fn default() -> Self {
        Self {
            rotation: Quaternion::default(),
            position: Position3D::default(),
        }
    }
}

#[derive(Clone, Decode, Encode, PartialEq, Debug)]
pub struct Location2D {
    pub rotation: Quaternion,
    pub position: Position2D,
}

impl Default for Location2D {
    fn default() -> Self {
        Self {
            rotation: Quaternion::default(),
            position: Position2D::default(),
        }
    }
}
