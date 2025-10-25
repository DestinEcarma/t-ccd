use std::fmt;

use glam::Vec2;
use serde::{
    Deserialize, Deserializer,
    de::{self, MapAccess, Visitor},
};

#[derive(Clone)]
pub struct ParticleState {
    pub position: Vec2,
    pub velocity: Vec2,
    pub radius: f32,
    pub mass: f32,
}

impl ParticleState {
    pub fn integrate(&mut self, dt: f32) {
        self.position += self.velocity * dt;
    }
}

impl From<ParticleRow> for ParticleState {
    fn from(value: ParticleRow) -> Self {
        Self {
            position: Vec2::new(value.x, value.y),
            velocity: Vec2::new(value.vx, value.vy),
            radius: value.radius,
            mass: value.radius,
        }
    }
}

#[derive(Debug, Deserialize)]
pub struct ParticleRow {
    pub frame: u64,
    pub time_s: f32,
    pub particle_id: usize,
    pub x: f32,
    pub y: f32,
    pub vx: f32,
    pub vy: f32,
    pub radius: f32,
    pub mass: f32,
}

#[derive(Debug)]
pub enum EventRow {
    Pair {
        frame: u64,
        time_s: f32,
        toi: f32,
        i: usize,
        j: usize,
        ix: f32,
        iy: f32,
        jx: f32,
        jy: f32,
        nx: f32,
        ny: f32,
        vrel_n_before: f32,
        vrel_n_after: f32,
    },
    Wall {
        frame: u64,
        time_s: f32,
        toi: f32,
        i: usize,
        wall: String,
        x: f32,
        y: f32,
        nx: f32,
        ny: f32,
        vn_before: f32,
        vn_after: f32,
    },
}

impl<'de> Deserialize<'de> for EventRow {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        struct EventRowVisitor;

        impl<'de> Visitor<'de> for EventRowVisitor {
            type Value = EventRow;

            fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                formatter.write_str("a CSV row representing an EventRow")
            }

            fn visit_map<M>(self, mut map: M) -> Result<EventRow, M::Error>
            where
                M: MapAccess<'de>,
            {
                let mut type_ = None::<String>;
                let mut frame = None::<u64>;
                let mut time_s = None::<f32>;
                let mut toi = None::<f32>;
                let mut i = None::<usize>;
                let mut j = None::<usize>;
                let mut wall = None::<String>;
                let mut ix = None::<f32>;
                let mut iy = None::<f32>;
                let mut jx = None::<f32>;
                let mut jy = None::<f32>;
                let mut nx = None::<f32>;
                let mut ny = None::<f32>;
                let mut vn_before = None::<f32>;
                let mut vn_after = None::<f32>;
                let mut vn_before = None::<f32>;
                let mut vn_after = None::<f32>;

                while let Some(key) = map.next_key::<String>()? {
                    match key.as_str() {
                        "type" => type_ = Some(map.next_value()?),
                        "frame" => frame = Some(map.next_value()?),
                        "time_s" => time_s = Some(map.next_value()?),
                        "toi" => toi = Some(map.next_value()?),
                        "i" => i = Some(map.next_value()?),
                        "j" => {
                            if let Some(type_) = &type_ {
                                match type_.as_str() {
                                    "Pair" => j = Some(map.next_value()?),
                                    "Wall" => wall = Some(map.next_value()?),
                                    _ => {
                                        map.next_value::<de::IgnoredAny>()?;
                                    }
                                }
                            }
                        }
                        "ix" => {
                            if let Some(type_) = &type_ {
                                match type_.as_str() {
                                    "Pair" => ix = Some(map.next_value()?),
                                    "Wall" => ix = Some(map.next_value()?),
                                    _ => {
                                        map.next_value::<de::IgnoredAny>()?;
                                    }
                                }
                            }
                        }
                        "iy" => {
                            if let Some(type_) = &type_ {
                                match type_.as_str() {
                                    "Pair" => iy = Some(map.next_value()?),
                                    "Wall" => iy = Some(map.next_value()?),
                                    _ => {
                                        map.next_value::<de::IgnoredAny>()?;
                                    }
                                }
                            }
                        }
                        "jx" => {
                            if let Some(type_) = &type_ {
                                match type_.as_str() {
                                    "Pair" => jx = Some(map.next_value()?),
                                    _ => {
                                        map.next_value::<de::IgnoredAny>()?;
                                    }
                                }
                            }
                        }
                        "jy" => {
                            if let Some(type_) = &type_ {
                                match type_.as_str() {
                                    "Pair" => jy = Some(map.next_value()?),
                                    _ => {
                                        map.next_value::<de::IgnoredAny>()?;
                                    }
                                }
                            }
                        }
                        "wall" => wall = Some(map.next_value()?),
                        "nx" => nx = Some(map.next_value()?),
                        "ny" => ny = Some(map.next_value()?),
                        "vrel_n_before" => vn_before = Some(map.next_value()?),
                        "vrel_n_after" => vn_after = Some(map.next_value()?),
                        _ => {
                            map.next_value::<de::IgnoredAny>()?;
                        }
                    }
                }

                let type_ = type_.ok_or_else(|| de::Error::missing_field("type"))?;
                let frame = frame.ok_or_else(|| de::Error::missing_field("frame"))?;
                let time_s = time_s.ok_or_else(|| de::Error::missing_field("time_s"))?;
                let toi = toi.ok_or_else(|| de::Error::missing_field("toi"))?;
                let i = i.ok_or_else(|| de::Error::missing_field("i"))?;
                let ix = ix.ok_or_else(|| de::Error::missing_field("ix"))?;
                let iy = iy.ok_or_else(|| de::Error::missing_field("iy"))?;
                let nx = nx.ok_or_else(|| de::Error::missing_field("nx"))?;
                let ny = ny.ok_or_else(|| de::Error::missing_field("ny"))?;
                let vn_before =
                    vn_before.ok_or_else(|| de::Error::missing_field("vrel_n_before"))?;
                let vn_after = vn_after.ok_or_else(|| de::Error::missing_field("vrel_n_after"))?;

                match type_.as_str() {
                    "Pair" => Ok(EventRow::Pair {
                        frame,
                        time_s,
                        toi,
                        i,
                        j: j.ok_or_else(|| de::Error::missing_field("j"))?,
                        ix,
                        iy,
                        jx: jx.ok_or_else(|| de::Error::missing_field("jx"))?,
                        jy: jy.ok_or_else(|| de::Error::missing_field("jy"))?,
                        nx,
                        ny,
                        vrel_n_before: vn_before,
                        vrel_n_after: vn_after,
                    }),
                    "Wall" => Ok(EventRow::Wall {
                        frame,
                        time_s,
                        toi,
                        i,
                        wall: wall.ok_or_else(|| de::Error::missing_field("wall"))?,
                        x: ix,
                        y: iy,
                        nx,
                        ny,
                        vn_before,
                        vn_after,
                    }),
                    _ => Err(de::Error::unknown_variant(&type_, &["Pair", "Wall"])),
                }
            }
        }

        deserializer.deserialize_map(EventRowVisitor)
    }
}
