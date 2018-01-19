use bitter::BitGet;
use network::{Rotation, Vector};
use parsing::{Header, decode_utf16, decode_windows1252};
use errors::AttributeError;
use std::borrow::Cow;

pub type AttributeDecodeFn = fn(&AttributeDecoder, &mut BitGet) -> Result<Attribute, AttributeError>;

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
    swiftness: f32,
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

    pub fn decode_byte(&self, bits: &mut BitGet) -> Result<Attribute, AttributeError> {
        bits.read_u8()
            .map(Attribute::Byte)
            .ok_or_else(|| AttributeError::NotEnoughDataFor("Byte"))
    }

    pub fn decode_boolean(&self, bits: &mut BitGet) -> Result<Attribute, AttributeError> {
        bits.read_bit()
            .map(Attribute::Boolean)
            .ok_or_else(|| AttributeError::NotEnoughDataFor("Boolean"))
    }

    pub fn decode_applied_damage(&self, bits: &mut BitGet) -> Result<Attribute, AttributeError> {
        if_chain! {
            if let Some(a) = bits.read_u8();
            if let Some(vector) = Vector::decode(bits);
            if let Some(b) = bits.read_u32();
            if let Some(c) = bits.read_u32();
            then {
                Ok(Attribute::AppliedDamage(a, vector, b, c))
            } else {
                Err(AttributeError::NotEnoughDataFor("Applied Damage"))
            }
        }
    }

    pub fn decode_damage_state(&self, bits: &mut BitGet) -> Result<Attribute, AttributeError> {
        if_chain! {
            if let Some(da) = bits.read_u8();
            if let Some(db) = bits.read_bit();
            if let Some(dc) = bits.read_u32();
            if let Some(dd) = Vector::decode(bits);
            if let Some(de) = bits.read_bit();
            if let Some(df) = bits.read_bit();
            then {
                Ok(Attribute::DamageState(da, db, dc, dd, de, df))
            } else {
                Err(AttributeError::NotEnoughDataFor("Damage State"))
            }
        }
    }

    pub fn decode_cam_settings(&self, bits: &mut BitGet) -> Result<Attribute, AttributeError> {
        if_chain! {
            if let Some(fov) = bits.read_f32();
            if let Some(height) = bits.read_f32();
            if let Some(angle) = bits.read_f32();
            if let Some(distance) = bits.read_f32();
            if let Some(swiftness) = bits.read_f32();
            if let Some(swivel) = bits.read_f32();
            if let Some(transition) = if self.major_version >= 868 && self.minor_version >= 20 {
                bits.read_f32().map(Some)
            } else {
                Some(None)
            };

            then {
                Ok(Attribute::CamSettings(CamSettings {
                    fov: fov,
                    height: height,
                    angle: angle,
                    distance: distance,
                    swiftness: swiftness,
                    swivel: swivel,
                    transition: transition,
                }))
            } else {
                Err(AttributeError::NotEnoughDataFor("Cam Settings"))
            }
        }
    }

    pub fn decode_club_colors(&self, bits: &mut BitGet) -> Result<Attribute, AttributeError> {
        if_chain! {
            if let Some(blue_flag) = bits.read_bit();
            if let Some(blue_color) = bits.read_u8();
            if let Some(orange_flag) = bits.read_bit();
            if let Some(orange_color) = bits.read_u8();
            then {
                Ok(Attribute::ClubColors(ClubColors {
                    blue_flag: blue_flag,
                    blue_color: blue_color,
                    orange_flag: orange_flag,
                    orange_color: orange_color,
                }))
            } else {
                Err(AttributeError::NotEnoughDataFor("Club Colors"))
            }
        }
    }

    pub fn decode_demolish(&self, bits: &mut BitGet) -> Result<Attribute, AttributeError> {
        if_chain! {
            if let Some(attacker_flag) = bits.read_bit();
            if let Some(attacker_actor_id) = bits.read_u32();
            if let Some(victim_flag) = bits.read_bit();
            if let Some(victim_actor_id) = bits.read_u32();
            if let Some(attack_velocity) = Vector::decode(bits);
            if let Some(victim_velocity) = Vector::decode(bits);
            then {
                Ok(Attribute::Demolish(Demolish {
                    attacker_flag: attacker_flag,
                    attacker_actor_id: attacker_actor_id,
                    victim_flag: victim_flag,
                    victim_actor_id: victim_actor_id,
                    attack_velocity: attack_velocity,
                    victim_velocity: victim_velocity,
                }))
            } else {
                Err(AttributeError::NotEnoughDataFor("Demolish"))
            }
        }
    }

    pub fn decode_enum(&self, bits: &mut BitGet) -> Result<Attribute, AttributeError> {
        bits.read_u32_bits(11)
            .map(|x| Attribute::Enum(x as u16))
            .ok_or_else(|| AttributeError::NotEnoughDataFor("Enum"))
    }

    pub fn decode_explosion(&self, bits: &mut BitGet) -> Result<Attribute, AttributeError> {
        decode_explosion(bits)
            .map(Attribute::Explosion)
            .ok_or_else(|| AttributeError::NotEnoughDataFor("Explosion"))
    }

    pub fn decode_extended_explosion(
        &self,
        bits: &mut BitGet,
    ) -> Result<Attribute, AttributeError> {
        if_chain! {
            if let Some(explosion) = decode_explosion(bits);
            if let Some(ea) = bits.read_bit();
            if let Some(eb) = bits.read_u32();
            then {
                Ok(Attribute::ExtendedExplosion(explosion, ea, eb))
            } else {
                Err(AttributeError::NotEnoughDataFor("Extended Explosion"))
            }
        }
    }

    pub fn decode_flagged(&self, bits: &mut BitGet) -> Result<Attribute, AttributeError> {
        if_chain! {
            if let Some(on) = bits.read_bit();
            if let Some(val) = bits.read_u32();
            then {
                Ok(Attribute::Flagged(on, val))
            } else {
                Err(AttributeError::NotEnoughDataFor("Flagged"))
            }
        }
    }

    pub fn decode_float(&self, bits: &mut BitGet) -> Result<Attribute, AttributeError> {
        bits.read_f32()
            .map(Attribute::Float)
            .ok_or_else(|| AttributeError::NotEnoughDataFor("Float"))
    }

    pub fn decode_game_mode(&self, bits: &mut BitGet) -> Result<Attribute, AttributeError> {
        let init =
            if self.major_version < 868 || (self.major_version == 868 && self.minor_version < 12) {
                2
            } else {
                8
            };

        bits.read_u32()
            .map(|x| Attribute::GameMode(init, x))
            .ok_or_else(|| AttributeError::NotEnoughDataFor("Game Mode"))
    }

    pub fn decode_int(&self, bits: &mut BitGet) -> Result<Attribute, AttributeError> {
        bits.read_i32()
            .map(Attribute::Int)
            .ok_or_else(|| AttributeError::NotEnoughDataFor("Int"))
    }

    pub fn decode_loadout(&self, bits: &mut BitGet) -> Result<Attribute, AttributeError> {
        decode_loadout(bits)
            .map(Attribute::Loadout)
            .ok_or_else(|| AttributeError::NotEnoughDataFor("Loadout"))
    }

    pub fn decode_team_loadout(&self, bits: &mut BitGet) -> Result<Attribute, AttributeError> {
        if_chain! {
            if let Some(blue) = decode_loadout(bits);
            if let Some(orange) = decode_loadout(bits);
            then {
                Ok(Attribute::TeamLoadout(TeamLoadout {
                    blue: blue,
                    orange: orange,
                }))
            } else {
                Err(AttributeError::NotEnoughDataFor("Team Loadout"))
            }
        }
    }

    pub fn decode_location(&self, bits: &mut BitGet) -> Result<Attribute, AttributeError> {
        Vector::decode(bits)
            .map(Attribute::Location)
            .ok_or_else(|| AttributeError::NotEnoughDataFor("Location"))
    }

    pub fn decode_music_stinger(&self, bits: &mut BitGet) -> Result<Attribute, AttributeError> {
        if_chain! {
            if let Some(flag) = bits.read_bit();
            if let Some(cue) = bits.read_u32();
            if let Some(trigger) = bits.read_u8();
            then {
                Ok(Attribute::MusicStinger(MusicStinger {
                    flag: flag,
                    cue: cue,
                    trigger: trigger,
                }))
            } else {
                Err(AttributeError::NotEnoughDataFor("Music Stinger"))
            }
        }
    }

    pub fn decode_pickup(&self, bits: &mut BitGet) -> Result<Attribute, AttributeError> {
        if_chain! {
            if let Some(instigator_id) = bits.if_get(|s| s.read_u32());
            if let Some(picked_up) = bits.read_bit();
            then {
                Ok(Attribute::Pickup(Pickup {
                    instigator_id: instigator_id,
                    picked_up: picked_up,
                }))
            } else {
                Err(AttributeError::NotEnoughDataFor("Pickup"))
            }
        }
    }

    pub fn decode_qword(&self, bits: &mut BitGet) -> Result<Attribute, AttributeError> {
        bits.read_u64()
            .map(Attribute::QWord)
            .ok_or_else(|| AttributeError::NotEnoughDataFor("QWord"))
    }

    pub fn decode_welded(&self, bits: &mut BitGet) -> Result<Attribute, AttributeError> {
        if_chain! {
            if let Some(active) = bits.read_bit();
            if let Some(actor_id) = bits.read_u32();
            if let Some(offset) = Vector::decode(bits);
            if let Some(mass) = bits.read_f32();
            if let Some(rotation) = Rotation::decode(bits);
            then {
                Ok(Attribute::Welded(Welded {
                    active: active,
                    actor_id: actor_id,
                    offset: offset,
                    mass: mass,
                    rotation: rotation,
                }))
            } else {
                Err(AttributeError::NotEnoughDataFor("Welded"))
            }
        }
    }

    pub fn decode_team_paint(&self, bits: &mut BitGet) -> Result<Attribute, AttributeError> {
        if_chain! {
            if let Some(team) = bits.read_u8();
            if let Some(primary_color) = bits.read_u8();
            if let Some(accent_color) = bits.read_u8();
            if let Some(primary_finish) = bits.read_u32();
            if let Some(accent_finish) = bits.read_u32();
            then {
                Ok(Attribute::TeamPaint(TeamPaint {
                    team: team,
                    primary_color: primary_color,
                    accent_color: accent_color,
                    primary_finish: primary_finish,
                    accent_finish: accent_finish,
                }))
            } else {
                Err(AttributeError::NotEnoughDataFor("Team Paint"))
            }
        }
    }

    pub fn decode_rigid_body(&self, bits: &mut BitGet) -> Result<Attribute, AttributeError> {
        if_chain! {
            if let Some(sleeping) = bits.read_bit();
            if let Some(location) = Vector::decode(bits);
            if let Some(x) = bits.read_u16();
            if let Some(y) = bits.read_u16();
            if let Some(z) = bits.read_u16();

            if let Some((linear_velocity, angular_velocity)) = if !sleeping {
                let lv = Vector::decode(bits);
                let av = Vector::decode(bits);
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
                    sleeping: sleeping,
                    location: location,
                    x: x,
                    y: y,
                    z: z,
                    linear_velocity: linear_velocity,
                    angular_velocity: angular_velocity,
                }))
            } else {
                Err(AttributeError::NotEnoughDataFor("Rigid Body"))
            }
        }
    }

    pub fn decode_not_implemented(&self, _bits: &mut BitGet) -> Result<Attribute, AttributeError> {
        Err(AttributeError::Unimplemented)
    }

    pub fn decode_string(&self, bits: &mut BitGet) -> Result<Attribute, AttributeError> {
        decode_text(bits)
            .map(Attribute::String)
            .ok_or_else(|| AttributeError::NotEnoughDataFor("String"))
    }

    pub fn decode_unique_id(&self, bits: &mut BitGet) -> Result<Attribute, AttributeError> {
        decode_unique_id(bits, self.net_version).map(Attribute::UniqueId)
    }

    pub fn decode_reservation(&self, bits: &mut BitGet) -> Result<Attribute, AttributeError> {
        if_chain! {
            if let Some(number) = bits.read_u32_bits(3);
            let unique = decode_unique_id(bits, self.net_version)?;
            if let Some(name) = if unique.system_id != 0 {
                decode_text(bits).map(Some)
            } else {
                Some(None)
            };

            if let Some(unknown1) = bits.read_bit();
            if let Some(unknown2) = bits.read_bit();
            if let Some(unknown3) = if self.major_version > 868 || (self.major_version == 868 && self.minor_version >= 12) {
                bits.read_u32_bits(6).map(|x| Some(x as u8))
            } else {
                Some(None)
            };

            then {
                Ok(Attribute::Reservation(Reservation {
                    number: number,
                    unique_id: unique,
                    name: name,
                    unknown1: unknown1,
                    unknown2: unknown2,
                    unknown3: unknown3
                }))
            } else {
                Err(AttributeError::NotEnoughDataFor("Reservation"))
            }
        }
    }

    pub fn decode_party_leader(&self, bits: &mut BitGet) -> Result<Attribute, AttributeError> {
        if let Some(system_id) = bits.read_u8() {
            if system_id != 0 {
                let id = decode_unique_id_with_system_id(bits, self.net_version, system_id)?;
                Ok(Attribute::PartyLeader(Some(id)))
            } else {
                Ok(Attribute::PartyLeader(None))
            }
        } else {
            Err(AttributeError::NotEnoughDataFor("Party Leader"))
        }
    }

    pub fn decode_private_match_settings(
        &self,
        bits: &mut BitGet,
    ) -> Result<Attribute, AttributeError> {
        if_chain! {
            if let Some(mutators) = decode_text(bits);
            if let Some(joinable_by) = bits.read_u32();
            if let Some(max_players) = bits.read_u32();
            if let Some(game_name) = decode_text(bits);
            if let Some(password) = decode_text(bits);
            if let Some(flag) = bits.read_bit();
            then {
                Ok(Attribute::PrivateMatch(PrivateMatchSettings {
                    mutators: mutators,
                    joinable_by: joinable_by,
                    max_players: max_players,
                    game_name: game_name,
                    password: password,
                    flag: flag,
                }))
            } else {
                Err(AttributeError::NotEnoughDataFor("Private Match"))
            }
        }
    }

    pub fn decode_loadout_online(&self, bits: &mut BitGet) -> Result<Attribute, AttributeError> {
        self.inner_decode_online_loadout(bits)
            .map(Attribute::LoadoutOnline)
            .ok_or_else(|| AttributeError::NotEnoughDataFor("Loadout Online"))
    }

    pub fn decode_loadouts_online(&self, bits: &mut BitGet) -> Result<Attribute, AttributeError> {
        if_chain! {
            if let Some(blue) = self.inner_decode_online_loadout(bits);
            if let Some(orange) = self.inner_decode_online_loadout(bits);
            if let Some(unknown1) = bits.read_bit();
            if let Some(unknown2) = bits.read_bit();
            then {
                Ok(Attribute::LoadoutsOnline(LoadoutsOnline {
                    blue: blue,
                    orange: orange,
                    unknown1: unknown1,
                    unknown2: unknown2,
                }))
            } else {
                Err(AttributeError::NotEnoughDataFor("Loadouts online"))
            }
        }
    }

    fn decode_product(&self, bits: &mut BitGet) -> Option<Product> {
        if_chain! {
            if let Some(unknown) = bits.read_bit();
            if let Some(obj_ind) = bits.read_u32();
            if let Some(val) = if obj_ind == self.color_ind {
                if let Some(on) = bits.read_bit() {
                    if on {
                        bits.read_u32_bits(31).map(Some)
                    } else {
                        Some(None)
                    }
                } else {
                    None
                }
            } else if obj_ind == self.painted_ind {
                if self.major_version >= 868 && self.minor_version >= 18 {
                    bits.read_u32_bits(31).map(Some)
                } else {
                    bits.read_bits_max(4, 14).map(Some)
                }
            } else {
                Some(None)
            };

            then {
                Some(Product {
                    unknown: unknown,
                    object_ind: obj_ind,
                    value: val,
                })
            } else {
                None
            }
        }
    }

    fn inner_decode_online_loadout(&self, bits: &mut BitGet) -> Option<Vec<Vec<Product>>> {
        if let Some(size) = bits.read_u8() {
            let mut res = Vec::with_capacity(size as usize);
            for _ in 0..size {
                if let Some(attribute_size) = bits.read_u8() {
                    let mut products = Vec::with_capacity(attribute_size as usize);
                    for _ in 0..attribute_size {
                        if let Some(product) = self.decode_product(bits) {
                            products.push(product);
                        } else {
                            return None;
                        }
                    }
                    res.push(products);
                } else {
                    return None;
                }
            }
            Some(res)
        } else {
            None
        }
    }
}

fn decode_explosion(bits: &mut BitGet) -> Option<Explosion> {
    if_chain! {
        if let Some(flag) = bits.read_bit();
        if let Some(actor_id) = bits.read_u32();
        if let Some(location) = Vector::decode(bits);
        then {
            Some(Explosion {
                flag: flag,
                actor_id: actor_id,
                location: location,
            })
        } else {
            None
        }
    }
}

fn decode_text(bits: &mut BitGet) -> Option<String> {
    if let Some(size) = bits.read_i32() {
        if size < 0 {
            bits.read_bytes(size * -2)
                .and_then(|data| decode_utf16(&data[..]).map(Cow::into_owned).ok())
        } else {
            bits.read_bytes(size)
                .and_then(|data| decode_windows1252(&data[..]).map(Cow::into_owned).ok())
        }
    } else {
        None
    }
}

fn decode_loadout_specials(
    bits: &mut BitGet,
) -> Option<(Option<u32>, Option<u32>, Option<u32>, Option<u32>)> {
    if_chain! {
        if let Some(engine_audio) = bits.read_u32();
        if let Some(trail) = bits.read_u32();
        if let Some(goal_explosion) = bits.read_u32();
        if let Some(banner) = bits.read_u32();
        then {
            Some((Some(engine_audio), Some(trail), Some(goal_explosion), Some(banner)))
        } else {
            None
        }
    }
}

fn decode_loadout(bits: &mut BitGet) -> Option<Loadout> {
    if_chain! {
        if let Some(version) = bits.read_u8();
        if let Some(body) = bits.read_u32();
        if let Some(decal) = bits.read_u32();
        if let Some(wheels) = bits.read_u32();
        if let Some(rocket_trail) = bits.read_u32();
        if let Some(antenna) = bits.read_u32();
        if let Some(topper) = bits.read_u32();
        if let Some(unknown1) = bits.read_u32();
        if let Some(unknown2) = if version > 10 {
            bits.read_u32().map(Some)
        } else {
            Some(None)
        };

        if let Some((engine_audio, trail, goal_explosion, banner)) = if version >= 16 {
            decode_loadout_specials(bits)
        } else {
            Some((None, None, None, None))
        };

        then {
            Some(Loadout {
                version: version,
                body: body,
                decal: decal,
                wheels: wheels,
                rocket_trail: rocket_trail,
                antenna: antenna,
                topper: topper,
                unknown1: unknown1,
                unknown2: unknown2,
                engine_audio: engine_audio,
                trail: trail,
                goal_explosion: goal_explosion,
                banner: banner,
            })
        } else {
            None
        }
    }
}

fn decode_unique_id(bits: &mut BitGet, net_version: i32) -> Result<UniqueId, AttributeError> {
    let system_id = bits.read_u8()
        .ok_or_else(|| AttributeError::NotEnoughDataFor("System id"))?;
    decode_unique_id_with_system_id(bits, net_version, system_id)
}

fn decode_unique_id_with_system_id(
    bits: &mut BitGet,
    net_version: i32,
    system_id: u8,
) -> Result<UniqueId, AttributeError> {
    let remote_id = match system_id {
        0 => bits.read_u32_bits(24)
            .ok_or_else(|| AttributeError::NotEnoughDataFor("SplitScreen"))
            .map(RemoteId::SplitScreen),
        1 => bits.read_u64()
            .ok_or_else(|| AttributeError::NotEnoughDataFor("Steam"))
            .map(RemoteId::Steam),
        2 => {
            // TODO: Extract ps4 id
            let to_read = if net_version >= 1 { 40 } else { 32 };
            bits.read_bytes(to_read)
                .ok_or_else(|| AttributeError::NotEnoughDataFor("Playstation"))
                .map(RemoteId::PlayStation)
        }
        4 => bits.read_u64()
            .ok_or_else(|| AttributeError::NotEnoughDataFor("Xbox"))
            .map(RemoteId::Xbox),
        6 => bits.read_bytes(32)
            .ok_or_else(|| AttributeError::NotEnoughDataFor("Switch"))
            .map(RemoteId::Switch),
        x => Err(AttributeError::UnrecognizedRemoteId(x)),
    }?;

    let local_id = bits.read_u8()
        .ok_or_else(|| AttributeError::NotEnoughDataFor("UniqueId local_id"))?;
    Ok(UniqueId {
        system_id: system_id,
        remote_id: remote_id,
        local_id: local_id,
    })
}
