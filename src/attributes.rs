use bitter::BitGet;
use network::{Vector, Rotation};
use parsing::{Header, decode_utf16, decode_windows1252};
use std::borrow::Cow;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AttributeTag {
    Boolean,
    Byte,
    AppliedDamage,
    DamageState,
    CamSettings,
    ClubColors,
    Demolish,
    Enum,
    Explosion,
    ExtendedExplosion,
    Flagged,
    Float,
    GameMode,
    Int,
    Loadout,
    TeamLoadout,
    Location,
    MusicStinger,
    Pickup,
    QWord,
    Welded,
    RigidBody,
    TeamPaint,
    NotImplemented,
    String,
    UniqueId,
    Reservation,
    PartyLeader,
    PrivateMatchSettings,
    LoadoutOnline,
    LoadoutsOnline,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub enum Attribute {
    Boolean(bool),
    Byte(u8),
    AppliedDamage(u8, Vector, u32, u32),
    DamageState(u8, bool, u32, Vector, bool, bool),
    CamSettings(CamSettings),
    ClubColors(ClubColors),
    Demolish(Demolish),
    Enum(u16),
    Explosion(Explosion),
    ExtendedExplosion(Explosion, bool, u32),
    Flagged(bool, u32),
    Float(f32),
    GameMode(u8, u32),
    Int(i32),
    Loadout(Loadout),
    TeamLoadout(TeamLoadout),
    Location(Vector),
    MusicStinger(MusicStinger),
    Pickup(Pickup),
    QWord(u64),
    Welded(Welded),
    TeamPaint(TeamPaint),
    RigidBody(RigidBody),
    String(String),
    UniqueId(UniqueId),
    Reservation(Reservation),
    PartyLeader(Option<UniqueId>),
    PrivateMatch(PrivateMatchSettings),
    LoadoutOnline(Vec<Vec<Product>>),
    LoadoutsOnline(LoadoutsOnline),
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize)]
pub struct CamSettings {
    fov: f32,
    height: f32,
    angle: f32,
    distance: f32,
    switftness: f32,
    swivel: f32,
    transition: Option<f32>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub struct ClubColors {
    blue_flag: bool,
    blue_color: u8,
    orange_flag: bool,
    orange_color: u8,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub struct Demolish {
    attacker_flag: bool,
    attacker_actor_id: u32,
    victim_flag: bool,
    victim_actor_id: u32,
    attack_velocity: Vector,
    victim_velocity: Vector,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub struct Explosion {
    flag: bool,
    actor_id: u32,
    location: Vector,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub struct Loadout {
    version: u8,
    body: u32,
    decal: u32,
    wheels: u32,
    rocket_trail: u32,
    antenna: u32,
    topper: u32,
    unknown1: u32,
    unknown2: Option<u32>,
    engine_audio: Option<u32>,
    trail: Option<u32>,
    goal_explosion: Option<u32>,
    banner: Option<u32>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub struct TeamLoadout {
    blue: Loadout,
    orange: Loadout,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub struct MusicStinger {
    flag: bool,
    cue: u32,
    trigger: u8,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub struct Pickup {
    instigator_id: Option<u32>,
    picked_up: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize)]
pub struct Welded {
    active: bool,
    actor_id: u32,
    offset: Vector,
    mass: f32,
    rotation: Rotation,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub struct TeamPaint {
    team: u8,
    primary_color: u8,
    accent_color: u8,
    primary_finish: u32,
    accent_finish: u32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub struct RigidBody {
    sleeping: bool,
    location: Vector,
    x: u16,
    y: u16,
    z: u16,
    linear_velocity: Option<Vector>,
    angular_velocity: Option<Vector>,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct UniqueId {
    system_id: u8,
    remote_id: RemoteId,
    local_id: u8,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub enum RemoteId {
    PlayStation(Vec<u8>),
    SplitScreen(u32),
    Steam(u64),
    Switch(Vec<u8>),
    Xbox(u64),
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct Reservation {
    number: u32,
    unique_id: UniqueId,
    name: Option<String>,
    unknown1: bool,
    unknown2: bool,
    unknown3: Option<u8>,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct PrivateMatchSettings {
    mutators: String,
    joinable_by: u32,
    max_players: u32,
    game_name: String,
    password: String,
    flag: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub struct Product {
    unknown: bool,
    object_ind: u32,
    value: Option<u32>,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct LoadoutsOnline {
    blue: Vec<Vec<Product>>,
    orange: Vec<Vec<Product>>,
    unknown1: bool,
    unknown2: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub struct AttributeDecoder {
    major_version: i32,
    minor_version: i32,
    net_version: i32,
    color_ind: u32,
    painted_ind: u32,
}

impl AttributeDecoder {
    pub fn new(header: &Header, color_ind: u32, painted_ind: u32) -> Self {
        AttributeDecoder {
            major_version: header.major_version,
            minor_version: header.minor_version,
            net_version: header.net_version.unwrap_or(0),
            color_ind: color_ind,
            painted_ind: painted_ind,
        }
    }

    pub fn decode_byte(&self, bits: &mut BitGet) -> Attribute {
        Attribute::Byte(bits.read_u8_unchecked())
    }

    pub fn decode_boolean(&self, bits: &mut BitGet) -> Attribute {
        Attribute::Boolean(bits.read_bit_unchecked())
    }

    pub fn decode_applied_damage(&self, bits: &mut BitGet) -> Attribute {
        Attribute::AppliedDamage(
            bits.read_u8_unchecked(),
            Vector::decode_unchecked(bits),
            bits.read_u32_unchecked(),
            bits.read_u32_unchecked()
        )
    }

    pub fn decode_damage_state(&self, bits: &mut BitGet) -> Attribute {
        Attribute::DamageState(
            bits.read_u8_unchecked(),
            bits.read_bit_unchecked(),
            bits.read_u32_unchecked(),
            Vector::decode_unchecked(bits),
            bits.read_bit_unchecked(),
            bits.read_bit_unchecked()
        )
    }
    
    pub fn decode_cam_settings(&self, bits: &mut BitGet) -> Attribute {
        Attribute::CamSettings(CamSettings {
            fov: bits.read_f32_unchecked(),
            height: bits.read_f32_unchecked(),
            angle: bits.read_f32_unchecked(),
            distance: bits.read_f32_unchecked(),
            switftness: bits.read_f32_unchecked(),
            swivel: bits.read_f32_unchecked(),
            transition: if self.major_version >= 868 && self.minor_version >= 20 {
                Some(bits.read_f32_unchecked())
            } else {
                None
            },
        })
    }

    pub fn decode_club_colors(&self, bits: &mut BitGet) -> Attribute {
        Attribute::ClubColors(ClubColors {
            blue_flag: bits.read_bit_unchecked(),
            blue_color: bits.read_u8_unchecked(),
            orange_flag: bits.read_bit_unchecked(),
            orange_color: bits.read_u8_unchecked(),
        })
    }

    pub fn decode_demolish(&self, bits: &mut BitGet) -> Attribute {
        Attribute::Demolish(Demolish {
            attacker_flag: bits.read_bit_unchecked(),
            attacker_actor_id: bits.read_u32_unchecked(),
            victim_flag: bits.read_bit_unchecked(),
            victim_actor_id: bits.read_u32_unchecked(),
            attack_velocity: Vector::decode_unchecked(bits),
            victim_velocity: Vector::decode_unchecked(bits),
        })
    }

    pub fn decode_enum(&self, bits: &mut BitGet) -> Attribute {
        Attribute::Enum(bits.read_u32_bits_unchecked(11) as u16)
    }

    pub fn decode_explosion(&self, bits: &mut BitGet) -> Attribute {
        Attribute::Explosion(decode_explosion(bits))
    }

    pub fn decode_extended_explosion(&self, bits: &mut BitGet) -> Attribute {
        Attribute::ExtendedExplosion(
            decode_explosion(bits),
            bits.read_bit_unchecked(),
            bits.read_u32_unchecked()
        )
    }

    pub fn decode_flagged(&self, bits: &mut BitGet) -> Attribute {
        Attribute::Flagged(bits.read_bit_unchecked(), bits.read_u32_unchecked())
    }

    pub fn decode_float(&self, bits: &mut BitGet) -> Attribute {
        Attribute::Float(bits.read_f32_unchecked())
    }

    pub fn decode_game_mode(&self, bits: &mut BitGet) -> Attribute {
        let init = if self.major_version < 868 || (self.major_version == 868 && self.minor_version < 12) {
            2
        } else {
            8
        };

        Attribute::GameMode(init, bits.read_u32_unchecked())
    }

    pub fn decode_int(&self, bits: &mut BitGet) -> Attribute {
        Attribute::Int(bits.read_i32_unchecked())
    }

    pub fn decode_loadout(&self, bits: &mut BitGet) -> Attribute {
        Attribute::Loadout(decode_loadout(bits))
    }

    pub fn decode_team_loadout(&self, bits: &mut BitGet) -> Attribute {
        Attribute::TeamLoadout(TeamLoadout {
            blue: decode_loadout(bits),
            orange: decode_loadout(bits),
        })
    }

    pub fn decode_location(&self, bits: &mut BitGet) -> Attribute {
        Attribute::Location(Vector::decode_unchecked(bits))
    }

    pub fn decode_music_stinger(&self, bits: &mut BitGet) -> Attribute {
        Attribute::MusicStinger(MusicStinger {
            flag: bits.read_bit_unchecked(),
            cue: bits.read_u32_unchecked(),
            trigger: bits.read_u8_unchecked(),
        })
    }

    pub fn decode_pickup(&self, bits: &mut BitGet) -> Attribute {
        Attribute::Pickup(Pickup {
            instigator_id: bits.if_get_unchecked(|s| s.read_u32_unchecked()),
            picked_up: bits.read_bit_unchecked(),
        })
    }

    pub fn decode_qword(&self, bits: &mut BitGet) -> Attribute {
        Attribute::QWord(bits.read_u64_unchecked())
    }

    pub fn decode_welded(&self, bits: &mut BitGet) -> Attribute {
        Attribute::Welded(Welded {
            active: bits.read_bit_unchecked(),
            actor_id: bits.read_u32_unchecked(),
            offset: Vector::decode_unchecked(bits),
            mass: bits.read_f32_unchecked(),
            rotation: Rotation::decode_unchecked(bits),
        })
    }

    pub fn decode_team_paint(&self, bits: &mut BitGet) -> Attribute {
        Attribute::TeamPaint(TeamPaint {
            team: bits.read_u8_unchecked(),
            primary_color: bits.read_u8_unchecked(),
            accent_color: bits.read_u8_unchecked(),
            primary_finish: bits.read_u32_unchecked(),
            accent_finish: bits.read_u32_unchecked(),
        })
    }

    pub fn decode_rigid_body(&self, bits: &mut BitGet) -> Attribute {
        let sleeping = bits.read_bit_unchecked(); 
        Attribute::RigidBody(RigidBody {
            sleeping: sleeping,
            location: Vector::decode_unchecked(bits),
            x: bits.read_u16_unchecked(),
            y: bits.read_u16_unchecked(),
            z: bits.read_u16_unchecked(),
            linear_velocity: if sleeping { None } else { Some(Vector::decode_unchecked(bits)) },
            angular_velocity: if sleeping { None } else { Some(Vector::decode_unchecked(bits)) },
        })
    }

    pub fn decode_not_implemented(&self, _bits: &mut BitGet) -> Attribute {
        unimplemented!()
    }

    pub fn decode_string(&self, bits: &mut BitGet) -> Attribute {
        decode_text(bits).map(Attribute::String).unwrap_or_else(|| Attribute::String(String::from("boxcars!byte_error")))
    }

    pub fn decode_unique_id(&self, bits: &mut BitGet) -> Attribute {
        Attribute::UniqueId(decode_unique_id(bits, self.net_version))
    }

    pub fn decode_reservation(&self, bits: &mut BitGet) -> Attribute {
        let number = bits.read_u32_bits_unchecked(3);
        let unique = decode_unique_id(bits, self.net_version);
        let system_id = unique.system_id;
        Attribute::Reservation(Reservation {
            number: number,
            unique_id: unique,
            name: if system_id != 0 { decode_text(bits) } else { None },
            unknown1: bits.read_bit_unchecked(),
            unknown2: bits.read_bit_unchecked(),
            unknown3: if self.major_version > 868 || (self.major_version == 868 && self.minor_version >= 12) {
            Some(bits.read_u32_bits_unchecked(6) as u8)
        } else {
            None
        } })
    }

    pub fn decode_party_leader(&self, bits: &mut BitGet) -> Attribute {
        let system_id = bits.read_u8_unchecked();
        if system_id != 0 {
            Attribute::PartyLeader(Some(decode_unique_id_with_system_id(bits, self.net_version, system_id)))
        } else {
            Attribute::PartyLeader(None)
        }
    }
    
    pub fn decode_private_match_settings(&self, bits: &mut BitGet) -> Attribute {
        Attribute::PrivateMatch(PrivateMatchSettings {
            mutators: decode_text(bits).unwrap(),
            joinable_by: bits.read_u32_unchecked(),
            max_players: bits.read_u32_unchecked(),
            game_name: decode_text(bits).unwrap(),
            password: decode_text(bits).unwrap(),
            flag: bits.read_bit_unchecked(),
        })
    }

    pub fn decode_loadout_online(&self, bits: &mut BitGet) -> Attribute {
        Attribute::LoadoutOnline(self.inner_decode_online_loadout(bits))
    }

    pub fn decode_loadouts_online(&self, bits: &mut BitGet) -> Attribute {
        Attribute::LoadoutsOnline(LoadoutsOnline {
            blue: self.inner_decode_online_loadout(bits),
            orange: self.inner_decode_online_loadout(bits),
            unknown1: bits.read_bit_unchecked(),
            unknown2: bits.read_bit_unchecked(),
        })
    }

    fn inner_decode_online_loadout(&self, bits: &mut BitGet) -> Vec<Vec<Product>> {
        let size = bits.read_u8_unchecked();
        let mut res = Vec::with_capacity(size as usize);
        for _ in 0..size {
            let attribute_size = bits.read_u8_unchecked();
            let mut products = Vec::with_capacity(attribute_size as usize);
            for _ in 0..attribute_size {
                let unknown = bits.read_bit_unchecked();
                let obj_ind = bits.read_u32_unchecked();
                let val = if obj_ind == self.color_ind && bits.read_bit_unchecked() {
                    Some(bits.read_u32_bits_unchecked(31))
                } else if obj_ind == self.painted_ind {
                    if self.major_version >= 868 && self.minor_version >= 18 {
                        Some(bits.read_u32_bits_unchecked(31))
                    } else {
                        Some(bits.read_bits_max_unchecked(4, 14))
                    }
                } else {
                    None
                };

                products.push(Product {
                    unknown: unknown,
                    object_ind: obj_ind,
                    value: val,
                });
            }
            res.push(products);
        }
        res
    }
}

fn decode_explosion(bits: &mut BitGet) -> Explosion {
    Explosion {
        flag: bits.read_bit_unchecked(),
        actor_id: bits.read_u32_unchecked(),
        location: Vector::decode_unchecked(bits),
    }
}

fn decode_text(bits: &mut BitGet) -> Option<String> {
    let size = bits.read_i32_unchecked();
    if size < 0 {
        bits.read_bytes(size * -2).and_then(|data| decode_utf16(&data[..]).map(Cow::into_owned).ok())
    } else {
        bits.read_bytes(size).and_then(|data| decode_windows1252(&data[..]).map(Cow::into_owned).ok())
    }
}

fn decode_loadout(bits: &mut BitGet) -> Loadout {
    let version = bits.read_u8_unchecked();
    Loadout {
        version: version,
        body: bits.read_u32_unchecked(),
        decal: bits.read_u32_unchecked(),
        wheels: bits.read_u32_unchecked(),
        rocket_trail: bits.read_u32_unchecked(),
        antenna: bits.read_u32_unchecked(),
        topper: bits.read_u32_unchecked(),
        unknown1: bits.read_u32_unchecked(),
        unknown2: if version > 10 { Some(bits.read_u32_unchecked()) } else { None },
        engine_audio: if version >= 16 { Some(bits.read_u32_unchecked()) } else { None },
        trail: if version >= 16 { Some(bits.read_u32_unchecked()) } else { None },
        goal_explosion: if version >= 16 { Some(bits.read_u32_unchecked()) } else { None },
        banner: if version >= 16 { Some(bits.read_u32_unchecked()) } else { None },
    }
}

fn decode_unique_id(bits: &mut BitGet, net_version: i32) -> UniqueId {
    let system_id = bits.read_u8_unchecked();
    decode_unique_id_with_system_id(bits, net_version, system_id)
}

fn decode_unique_id_with_system_id(bits: &mut BitGet, net_version: i32, system_id: u8) -> UniqueId {
    let remote_id = match system_id {
        0 => RemoteId::SplitScreen(bits.read_u32_bits_unchecked(24)),
        1 => RemoteId::Steam(bits.read_u64_unchecked()),
        2 => {
            // TODO: Extract ps4 id
            if net_version >= 1 {
                RemoteId::PlayStation(bits.read_bytes(40).unwrap())
            } else {
                RemoteId::PlayStation(bits.read_bytes(32).unwrap())
            }
        },
        4 => RemoteId::Xbox(bits.read_u64_unchecked()),
        6 => RemoteId::Switch(bits.read_bytes(32).unwrap()),
        x => panic!("EEEEEK"),
    };
    let local_id = bits.read_u8_unchecked();
    UniqueId {
        system_id: system_id,
        remote_id: remote_id,
        local_id: local_id,
    }
}


