use attributes::{Attribute, RigidBody};
use bitter::BitGet;
use errors::AttributeError;
use network::Vector;

pub trait NetworkParser {
    fn decode_vector(bits: &mut BitGet) -> Option<Vector>;
    fn decode_rigid_body(bits: &mut BitGet) -> Result<Attribute, AttributeError>;
}

pub struct BeachParser;

impl NetworkParser for BeachParser {
    fn decode_vector(bits: &mut BitGet) -> Option<Vector> {
        if_chain! {
            if let Some(size_bits) = bits.read_bits_max(5, 22);
            let bias = 1 << (size_bits + 1);
            let bit_limit = (size_bits + 2) as i32;
            if let Some(dx) = bits.read_u32_bits(bit_limit);
            if let Some(dy) = bits.read_u32_bits(bit_limit);
            if let Some(dz) = bits.read_u32_bits(bit_limit);
            then {
                Some(Vector {
                    bias: bias as i32,
                    dx: dx as i32,
                    dy: dy as i32,
                    dz: dz as i32,
                })
            } else {
                None
            }
        }
    }

    fn decode_rigid_body(bits: &mut BitGet) -> Result<Attribute, AttributeError> {
        if_chain! {
            if let Some(sleeping) = bits.read_bit();
            if let Some(location) =  Self::decode_vector(bits);
            if let Some(_u1) = bits.read_bit();
            if let Some(x) = bits.read_u32_bits(18);
            if let Some(y) = bits.read_u32_bits(18);
            if let Some(z) = bits.read_u32_bits(18);
            if let Some(_u2) = bits.read_bit();

            if let Some((linear_velocity, angular_velocity)) = if !sleeping {
                let lv = Self::decode_vector(bits);
                let av = Self::decode_vector(bits);
                if lv.is_some() && av.is_some() {
                    Some((lv, av))
                } else {
                    None
                }
            } else {
                Some((None, None))
            };

            then {
                Ok(Attribute::RigidBody(RigidBody {
                    sleeping,
                    location,
                    x: x as u16,
                    y: y as u16,
                    z: z as u16,
                    linear_velocity,
                    angular_velocity,
                }))
            } else {
                Err(AttributeError::NotEnoughDataFor("Rigid Body"))
            }
        }
    }
}

pub struct OldParser;

impl NetworkParser for OldParser {
    fn decode_vector(bits: &mut BitGet) -> Option<Vector> {
        if_chain! {
            if let Some(size_bits) = bits.read_bits_max(5, 20);
            let bias = 1 << (size_bits + 1);
            let bit_limit = (size_bits + 2) as i32;
            if let Some(dx) = bits.read_u32_bits(bit_limit);
            if let Some(dy) = bits.read_u32_bits(bit_limit);
            if let Some(dz) = bits.read_u32_bits(bit_limit);
            then {
                Some(Vector {
                    bias: bias as i32,
                    dx: dx as i32,
                    dy: dy as i32,
                    dz: dz as i32,
                })
            } else {
                None
            }
        }
    }

    fn decode_rigid_body(bits: &mut BitGet) -> Result<Attribute, AttributeError> {
        if_chain! {
            if let Some(sleeping) = bits.read_bit();
            if let Some(location) =  Self::decode_vector(bits);
            if let Some(x) = bits.read_u16();
            if let Some(y) = bits.read_u16();
            if let Some(z) = bits.read_u16();

            if let Some((linear_velocity, angular_velocity)) = if !sleeping {
                let lv = Self::decode_vector(bits);
                let av = Self::decode_vector(bits);
                if lv.is_some() && av.is_some() {
                    Some((lv, av))
                } else {
                    None
                }
            } else {
                Some((None, None))
            };

            then {
                Ok(Attribute::RigidBody(RigidBody {
                    sleeping,
                    location,
                    x,
                    y,
                    z,
                    linear_velocity,
                    angular_velocity,
                }))
            } else {
                Err(AttributeError::NotEnoughDataFor("Rigid Body"))
            }
        }
    }
}
