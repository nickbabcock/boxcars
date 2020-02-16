use crate::errors::AttributeError;
use crate::network::{ObjectId, Quaternion, Rotation, Vector3f, VersionTriplet};
use crate::parsing_utils::{decode_utf16, decode_windows1252};
use bitter::BitGet;
use encoding_rs::WINDOWS_1252;
use std::borrow::Cow;
use std::collections::HashMap;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum AttributeTag {
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
    FlaggedByte,
    Flagged,
    Float,
    GameMode,
    Int,
    Int64,
    Loadout,
    TeamLoadout,
    Location,
    MusicStinger,
    Pickup,
    PickupNew,
    PlayerHistoryKey,
    QWord,
    Welded,
    RigidBody,
    Title,
    TeamPaint,
    NotImplemented,
    String,
    UniqueId,
    Reservation,
    PartyLeader,
    PrivateMatchSettings,
    LoadoutOnline,
    LoadoutsOnline,
    StatEvent,
    RotationTag,
    RepStatTitle,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub enum Attribute {
    Boolean(bool),
    Byte(u8),
    AppliedDamage(u8, Vector3f, u32, u32),
    DamageState(u8, bool, u32, Vector3f, bool, bool),
    CamSettings(CamSettings),
    ClubColors(ClubColors),
    Demolish(Demolish),
    Enum(u16),
    Explosion(Explosion),
    ExtendedExplosion(Explosion, bool, u32),
    FlaggedByte(bool, u8),
    Flagged(bool, u32),
    Float(f32),
    GameMode(u8, u8),
    Int(i32),

    #[serde(serialize_with = "crate::serde_utils::display_it")]
    Int64(i64),
    Loadout(Loadout),
    TeamLoadout(TeamLoadout),
    Location(Vector3f),
    MusicStinger(MusicStinger),
    PlayerHistoryKey(u16),
    Pickup(Pickup),
    PickupNew(PickupNew),

    #[serde(serialize_with = "crate::serde_utils::display_it")]
    QWord(u64),
    Welded(Welded),
    Title(bool, bool, u32, u32, u32, u32, u32, bool),
    TeamPaint(TeamPaint),
    RigidBody(RigidBody),
    String(String),
    UniqueId(UniqueId),
    Reservation(Reservation),
    PartyLeader(Option<UniqueId>),
    PrivateMatch(PrivateMatchSettings),
    LoadoutOnline(Vec<Vec<Product>>),
    LoadoutsOnline(LoadoutsOnline),
    StatEvent(bool, u32),
    Rotation(Rotation),
    RepStatTitle(RepStatTitle),
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize)]
#[cfg_attr(feature = "py", derive(dict_derive::IntoPyObject))]
pub struct CamSettings {
    pub fov: f32,
    pub height: f32,
    pub angle: f32,
    pub distance: f32,
    pub swiftness: f32,
    pub swivel: f32,
    pub transition: Option<f32>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[cfg_attr(feature = "py", derive(dict_derive::IntoPyObject))]
pub struct ClubColors {
    pub blue_flag: bool,
    pub blue_color: u8,
    pub orange_flag: bool,
    pub orange_color: u8,
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize)]
#[cfg_attr(feature = "py", derive(dict_derive::IntoPyObject))]
pub struct Demolish {
    pub attacker_flag: bool,
    pub attacker_actor_id: u32,
    pub victim_flag: bool,
    pub victim_actor_id: u32,
    pub attack_velocity: Vector3f,
    pub victim_velocity: Vector3f,
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize)]
#[cfg_attr(feature = "py", derive(dict_derive::IntoPyObject))]
pub struct Explosion {
    pub flag: bool,
    pub actor_id: u32,
    pub location: Vector3f,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[cfg_attr(feature = "py", derive(dict_derive::IntoPyObject))]
pub struct Loadout {
    pub version: u8,
    pub body: u32,
    pub decal: u32,
    pub wheels: u32,
    pub rocket_trail: u32,
    pub antenna: u32,
    pub topper: u32,
    pub unknown1: u32,
    pub unknown2: Option<u32>,
    pub engine_audio: Option<u32>,
    pub trail: Option<u32>,
    pub goal_explosion: Option<u32>,
    pub banner: Option<u32>,
    pub unknown3: Option<u32>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[cfg_attr(feature = "py", derive(dict_derive::IntoPyObject))]
pub struct TeamLoadout {
    pub blue: Loadout,
    pub orange: Loadout,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[cfg_attr(feature = "py", derive(dict_derive::IntoPyObject))]
pub struct MusicStinger {
    pub flag: bool,
    pub cue: u32,
    pub trigger: u8,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[cfg_attr(feature = "py", derive(dict_derive::IntoPyObject))]
pub struct Pickup {
    pub instigator_id: Option<u32>,
    pub picked_up: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[cfg_attr(feature = "py", derive(dict_derive::IntoPyObject))]
pub struct PickupNew {
    pub instigator_id: Option<u32>,
    pub picked_up: u8,
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize)]
#[cfg_attr(feature = "py", derive(dict_derive::IntoPyObject))]
pub struct Welded {
    pub active: bool,
    pub actor_id: u32,
    pub offset: Vector3f,
    pub mass: f32,
    pub rotation: Rotation,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[cfg_attr(feature = "py", derive(dict_derive::IntoPyObject))]
pub struct TeamPaint {
    pub team: u8,
    pub primary_color: u8,
    pub accent_color: u8,
    pub primary_finish: u32,
    pub accent_finish: u32,
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize)]
#[cfg_attr(feature = "py", derive(dict_derive::IntoPyObject))]
pub struct RigidBody {
    pub sleeping: bool,
    pub location: Vector3f,
    pub rotation: Quaternion,
    pub linear_velocity: Option<Vector3f>,
    pub angular_velocity: Option<Vector3f>,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
#[cfg_attr(feature = "py", derive(dict_derive::IntoPyObject))]
pub struct UniqueId {
    pub system_id: u8,
    pub remote_id: RemoteId,
    pub local_id: u8,
}

#[derive(Debug, Default, Clone, PartialEq, Serialize)]
#[cfg_attr(feature = "py", derive(dict_derive::IntoPyObject))]
pub struct PsyNetId {
    #[serde(serialize_with = "crate::serde_utils::display_it")]
    pub online_id: u64,
    pub unknown1: Vec<u8>,
}

#[derive(Debug, Default, Clone, PartialEq, Serialize)]
#[cfg_attr(feature = "py", derive(dict_derive::IntoPyObject))]
pub struct SwitchId {
    #[serde(serialize_with = "crate::serde_utils::display_it")]
    pub online_id: u64,
    pub unknown1: Vec<u8>,
}

#[derive(Debug, Default, Clone, PartialEq, Serialize)]
#[cfg_attr(feature = "py", derive(dict_derive::IntoPyObject))]
pub struct Ps4Id {
    #[serde(serialize_with = "crate::serde_utils::display_it")]
    pub online_id: u64,
    pub name: String,
    pub unknown1: Vec<u8>,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub enum RemoteId {
    PlayStation(Ps4Id),
    PsyNet(PsyNetId),
    SplitScreen(u32),

    #[serde(serialize_with = "crate::serde_utils::display_it")]
    Steam(u64),
    Switch(SwitchId),

    #[serde(serialize_with = "crate::serde_utils::display_it")]
    Xbox(u64),

    #[serde(serialize_with = "crate::serde_utils::display_it")]
    QQ(u64),
}

#[derive(Debug, Clone, PartialEq, Serialize)]
#[cfg_attr(feature = "py", derive(dict_derive::IntoPyObject))]
pub struct Reservation {
    pub number: u32,
    pub unique_id: UniqueId,
    pub name: Option<String>,
    pub unknown1: bool,
    pub unknown2: bool,
    pub unknown3: Option<u8>,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
#[cfg_attr(feature = "py", derive(dict_derive::IntoPyObject))]
pub struct PrivateMatchSettings {
    pub mutators: String,
    pub joinable_by: u32,
    pub max_players: u32,
    pub game_name: String,
    pub password: String,
    pub flag: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[cfg_attr(feature = "py", derive(dict_derive::IntoPyObject))]
pub struct Product {
    pub unknown: bool,
    pub object_ind: u32,
    pub value: ProductValue,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
#[cfg_attr(feature = "py", derive(dict_derive::IntoPyObject))]
pub struct LoadoutsOnline {
    pub blue: Vec<Vec<Product>>,
    pub orange: Vec<Vec<Product>>,
    pub unknown1: bool,
    pub unknown2: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub enum ProductValue {
    NoColor,
    Absent,
    OldColor(u32),
    NewColor(u32),
    OldPaint(u32),
    NewPaint(u32),
    Title(String),
    SpecialEdition(u32),
    OldTeamEdition(u32),
    NewTeamEdition(u32),
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[cfg_attr(feature = "py", derive(dict_derive::IntoPyObject))]
pub struct RepStatTitle {
    pub unknown: bool,
    pub name: String,
    pub unknown2: bool,
    pub index: u32,
    pub value: u32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct ProductValueDecoder {
    version: VersionTriplet,
    color_ind: u32,
    painted_ind: u32,
    special_edition_ind: u32,
    team_edition_ind: u32,
    title_ind: u32,
}

impl ProductValueDecoder {
    pub fn create(version: VersionTriplet, name_obj_ind: &HashMap<&str, Vec<ObjectId>>) -> Self {
        let color_ind = name_obj_ind
            .get("TAGame.ProductAttribute_UserColor_TA")
            .map(|x| usize::from(x[0]) as u32)
            .unwrap_or(0);
        let painted_ind = name_obj_ind
            .get("TAGame.ProductAttribute_Painted_TA")
            .map(|x| usize::from(x[0]) as u32)
            .unwrap_or(0);
        let title_ind = name_obj_ind
            .get("TAGame.ProductAttribute_TitleID_TA")
            .map(|x| usize::from(x[0]) as u32)
            .unwrap_or(0) as u32;
        let special_edition_ind = name_obj_ind
            .get("TAGame.ProductAttribute_SpecialEdition_TA")
            .map(|x| usize::from(x[0]) as u32)
            .unwrap_or(0);
        let team_edition_ind = name_obj_ind
            .get("TAGame.ProductAttribute_TeamEdition_TA")
            .map(|x| usize::from(x[0]) as u32)
            .unwrap_or(0);

        ProductValueDecoder {
            version,
            color_ind,
            painted_ind,
            title_ind,
            special_edition_ind,
            team_edition_ind,
        }
    }

    pub fn decode(&self, bits: &mut BitGet<'_>, obj_ind: u32) -> Option<ProductValue> {
        if obj_ind == self.color_ind {
            if self.version >= VersionTriplet(868, 23, 8) {
                bits.read_u32().map(ProductValue::NewColor)
            } else {
                bits.if_get(|b| b.read_u32_bits(31).map(ProductValue::OldColor))
                    .map(|x| x.unwrap_or(ProductValue::NoColor))
            }
        } else if obj_ind == self.painted_ind {
            if self.version >= VersionTriplet(868, 18, 0) {
                bits.read_u32_bits(31).map(ProductValue::NewPaint)
            } else {
                bits.read_bits_max(14).map(ProductValue::OldPaint)
            }
        } else if obj_ind == self.title_ind {
            decode_text(bits).ok().map(ProductValue::Title)
        } else if obj_ind == self.special_edition_ind {
            bits.read_u32_bits(31).map(ProductValue::SpecialEdition)
        } else if obj_ind == self.team_edition_ind {
            if self.version >= VersionTriplet(868, 18, 0) {
                bits.read_u32_bits(31).map(ProductValue::NewTeamEdition)
            } else {
                bits.read_bits_max(14).map(ProductValue::OldTeamEdition)
            }
        } else {
            Some(ProductValue::Absent)
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct AttributeDecoder {
    version: VersionTriplet,
    product_decoder: ProductValueDecoder,
}

impl AttributeDecoder {
    pub fn new(version: VersionTriplet, product_decoder: ProductValueDecoder) -> Self {
        AttributeDecoder {
            version,
            product_decoder,
        }
    }

    pub fn decode(
        &self,
        tag: AttributeTag,
        bits: &mut BitGet<'_>,
    ) -> Result<Attribute, AttributeError> {
        match tag {
            AttributeTag::Boolean => self.decode_boolean(bits),
            AttributeTag::Byte => self.decode_byte(bits),
            AttributeTag::AppliedDamage => self.decode_applied_damage(bits),
            AttributeTag::DamageState => self.decode_damage_state(bits),
            AttributeTag::CamSettings => self.decode_cam_settings(bits),
            AttributeTag::ClubColors => self.decode_club_colors(bits),
            AttributeTag::Demolish => self.decode_demolish(bits),
            AttributeTag::Enum => self.decode_enum(bits),
            AttributeTag::Explosion => self.decode_explosion(bits),
            AttributeTag::ExtendedExplosion => self.decode_extended_explosion(bits),
            AttributeTag::Flagged => self.decode_flagged(bits),
            AttributeTag::FlaggedByte => self.decode_flagged_byte(bits),
            AttributeTag::Float => self.decode_float(bits),
            AttributeTag::GameMode => self.decode_game_mode(bits),
            AttributeTag::Int => self.decode_int(bits),
            AttributeTag::Int64 => self.decode_int64(bits),
            AttributeTag::Loadout => self.decode_loadout(bits),
            AttributeTag::TeamLoadout => self.decode_team_loadout(bits),
            AttributeTag::Location => self.decode_location(bits),
            AttributeTag::MusicStinger => self.decode_music_stinger(bits),
            AttributeTag::Pickup => self.decode_pickup(bits),
            AttributeTag::PickupNew => self.decode_pickup_new(bits),
            AttributeTag::PlayerHistoryKey => self.decode_player_history_key(bits),
            AttributeTag::QWord => self.decode_qword(bits),
            AttributeTag::Welded => self.decode_welded(bits),
            AttributeTag::RigidBody => self.decode_rigid_body(bits),
            AttributeTag::Title => self.decode_title(bits),
            AttributeTag::TeamPaint => self.decode_team_paint(bits),
            AttributeTag::NotImplemented => self.decode_not_implemented(bits),
            AttributeTag::String => self.decode_string(bits),
            AttributeTag::UniqueId => self.decode_unique_id(bits),
            AttributeTag::Reservation => self.decode_reservation(bits),
            AttributeTag::PartyLeader => self.decode_party_leader(bits),
            AttributeTag::PrivateMatchSettings => self.decode_private_match_settings(bits),
            AttributeTag::LoadoutOnline => self.decode_loadout_online(bits),
            AttributeTag::LoadoutsOnline => self.decode_loadouts_online(bits),
            AttributeTag::StatEvent => self.decode_stat_event(bits),
            AttributeTag::RotationTag => self.decode_rotation(bits),
            AttributeTag::RepStatTitle => self.decode_rep_stat_title(bits),
        }
    }

    pub fn decode_byte(&self, bits: &mut BitGet<'_>) -> Result<Attribute, AttributeError> {
        bits.read_u8()
            .map(Attribute::Byte)
            .ok_or_else(|| AttributeError::NotEnoughDataFor("Byte"))
    }

    pub fn decode_player_history_key(
        &self,
        bits: &mut BitGet<'_>,
    ) -> Result<Attribute, AttributeError> {
        bits.read_u32_bits(14)
            .map(|x| Attribute::PlayerHistoryKey(x as u16))
            .ok_or_else(|| AttributeError::NotEnoughDataFor("PlayerHistoryKey"))
    }

    pub fn decode_flagged_byte(&self, bits: &mut BitGet<'_>) -> Result<Attribute, AttributeError> {
        if_chain! {
            if let Some(b) = bits.read_bit();
            if let Some(data) = bits.read_u8();
            then {
                Ok(Attribute::FlaggedByte(b, data))
            } else {
                Err(AttributeError::NotEnoughDataFor("FlaggedByte"))
            }
        }
    }

    pub fn decode_boolean(&self, bits: &mut BitGet<'_>) -> Result<Attribute, AttributeError> {
        bits.read_bit()
            .map(Attribute::Boolean)
            .ok_or_else(|| AttributeError::NotEnoughDataFor("Boolean"))
    }

    pub fn decode_applied_damage(
        &self,
        bits: &mut BitGet<'_>,
    ) -> Result<Attribute, AttributeError> {
        if_chain! {
            if let Some(a) = bits.read_u8();
            if let Some(vector) = Vector3f::decode(bits, self.version.net_version());
            if let Some(b) = bits.read_u32();
            if let Some(c) = bits.read_u32();
            then {
                Ok(Attribute::AppliedDamage(a, vector, b, c))
            } else {
                Err(AttributeError::NotEnoughDataFor("Applied Damage"))
            }
        }
    }

    pub fn decode_damage_state(&self, bits: &mut BitGet<'_>) -> Result<Attribute, AttributeError> {
        if_chain! {
            if let Some(da) = bits.read_u8();
            if let Some(db) = bits.read_bit();
            if let Some(dc) = bits.read_u32();
            if let Some(dd) = Vector3f::decode(bits, self.version.net_version());
            if let Some(de) = bits.read_bit();
            if let Some(df) = bits.read_bit();
            then {
                Ok(Attribute::DamageState(da, db, dc, dd, de, df))
            } else {
                Err(AttributeError::NotEnoughDataFor("Damage State"))
            }
        }
    }

    pub fn decode_cam_settings(&self, bits: &mut BitGet<'_>) -> Result<Attribute, AttributeError> {
        if_chain! {
            if let Some(fov) = bits.read_f32();
            if let Some(height) = bits.read_f32();
            if let Some(angle) = bits.read_f32();
            if let Some(distance) = bits.read_f32();
            if let Some(swiftness) = bits.read_f32();
            if let Some(swivel) = bits.read_f32();
            if let Some(transition) = if self.version >= VersionTriplet(868, 20, 0) {
                bits.read_f32().map(Some)
            } else {
                Some(None)
            };

            then {
                Ok(Attribute::CamSettings(CamSettings {
                    fov,
                    height,
                    angle,
                    distance,
                    swiftness,
                    swivel,
                    transition,
                }))
            } else {
                Err(AttributeError::NotEnoughDataFor("Cam Settings"))
            }
        }
    }

    pub fn decode_club_colors(&self, bits: &mut BitGet<'_>) -> Result<Attribute, AttributeError> {
        if_chain! {
            if let Some(blue_flag) = bits.read_bit();
            if let Some(blue_color) = bits.read_u8();
            if let Some(orange_flag) = bits.read_bit();
            if let Some(orange_color) = bits.read_u8();
            then {
                Ok(Attribute::ClubColors(ClubColors {
                    blue_flag,
                    blue_color,
                    orange_flag,
                    orange_color,
                }))
            } else {
                Err(AttributeError::NotEnoughDataFor("Club Colors"))
            }
        }
    }

    pub fn decode_demolish(&self, bits: &mut BitGet<'_>) -> Result<Attribute, AttributeError> {
        if_chain! {
            if let Some(attacker_flag) = bits.read_bit();
            if let Some(attacker_actor_id) = bits.read_u32();
            if let Some(victim_flag) = bits.read_bit();
            if let Some(victim_actor_id) = bits.read_u32();
            if let Some(attack_velocity) = Vector3f::decode(bits, self.version.net_version());
            if let Some(victim_velocity) = Vector3f::decode(bits, self.version.net_version());
            then {
                Ok(Attribute::Demolish(Demolish {
                    attacker_flag,
                    attacker_actor_id,
                    victim_flag,
                    victim_actor_id,
                    attack_velocity,
                    victim_velocity,
                }))
            } else {
                Err(AttributeError::NotEnoughDataFor("Demolish"))
            }
        }
    }

    pub fn decode_enum(&self, bits: &mut BitGet<'_>) -> Result<Attribute, AttributeError> {
        bits.read_u32_bits(11)
            .map(|x| Attribute::Enum(x as u16))
            .ok_or_else(|| AttributeError::NotEnoughDataFor("Enum"))
    }

    pub fn decode_explosion(&self, bits: &mut BitGet<'_>) -> Result<Attribute, AttributeError> {
        decode_explosion(bits, self.version.net_version())
            .map(Attribute::Explosion)
            .ok_or_else(|| AttributeError::NotEnoughDataFor("Explosion"))
    }

    pub fn decode_stat_event(&self, bits: &mut BitGet<'_>) -> Result<Attribute, AttributeError> {
        if_chain! {
            if let Some(u1) = bits.read_bit();
            if let Some(id) = bits.read_u32();
            then {
                Ok(Attribute::StatEvent(u1, id))
            } else {
                Err(AttributeError::NotEnoughDataFor("Stat Event"))
            }
        }
    }

    pub fn decode_rep_stat_title(
        &self,
        bits: &mut BitGet<'_>,
    ) -> Result<Attribute, AttributeError> {
        if_chain! {
            if let Some(unknown) = bits.read_bit();
            let name = decode_text(bits)?;
            if let Some(unknown2) = bits.read_bit();
            if let Some(index) = bits.read_u32();
            if let Some(value) = bits.read_u32();
            then {
                Ok(Attribute::RepStatTitle(RepStatTitle {
                    unknown, name, unknown2, index, value
                }))
            } else {
                Err(AttributeError::NotEnoughDataFor("RepStatTitle"))
            }
        }
    }

    pub fn decode_extended_explosion(
        &self,
        bits: &mut BitGet<'_>,
    ) -> Result<Attribute, AttributeError> {
        if_chain! {
            if let Some(explosion) = decode_explosion(bits, self.version.net_version());
            if let Some(ea) = bits.read_bit();
            if let Some(eb) = bits.read_u32();
            then {
                Ok(Attribute::ExtendedExplosion(explosion, ea, eb))
            } else {
                Err(AttributeError::NotEnoughDataFor("Extended Explosion"))
            }
        }
    }

    pub fn decode_flagged(&self, bits: &mut BitGet<'_>) -> Result<Attribute, AttributeError> {
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

    pub fn decode_float(&self, bits: &mut BitGet<'_>) -> Result<Attribute, AttributeError> {
        bits.read_f32()
            .map(Attribute::Float)
            .ok_or_else(|| AttributeError::NotEnoughDataFor("Float"))
    }

    pub fn decode_game_mode(&self, bits: &mut BitGet<'_>) -> Result<Attribute, AttributeError> {
        let init: u8 = if self.version < VersionTriplet(868, 12, 0) {
            2
        } else {
            8
        };

        bits.read_u32_bits(i32::from(init))
            .map(|x| Attribute::GameMode(init, x as u8))
            .ok_or_else(|| AttributeError::NotEnoughDataFor("Game Mode"))
    }

    pub fn decode_int(&self, bits: &mut BitGet<'_>) -> Result<Attribute, AttributeError> {
        bits.read_i32()
            .map(Attribute::Int)
            .ok_or_else(|| AttributeError::NotEnoughDataFor("Int"))
    }

    pub fn decode_int64(&self, bits: &mut BitGet<'_>) -> Result<Attribute, AttributeError> {
        bits.read_i64()
            .map(Attribute::Int64)
            .ok_or_else(|| AttributeError::NotEnoughDataFor("Int64"))
    }

    pub fn decode_loadout(&self, bits: &mut BitGet<'_>) -> Result<Attribute, AttributeError> {
        decode_loadout(bits)
            .map(Attribute::Loadout)
            .ok_or_else(|| AttributeError::NotEnoughDataFor("Loadout"))
    }

    pub fn decode_team_loadout(&self, bits: &mut BitGet<'_>) -> Result<Attribute, AttributeError> {
        if_chain! {
            if let Some(blue) = decode_loadout(bits);
            if let Some(orange) = decode_loadout(bits);
            then {
                Ok(Attribute::TeamLoadout(TeamLoadout {
                    blue,
                    orange,
                }))
            } else {
                Err(AttributeError::NotEnoughDataFor("Team Loadout"))
            }
        }
    }

    pub fn decode_location(&self, bits: &mut BitGet<'_>) -> Result<Attribute, AttributeError> {
        Vector3f::decode(bits, self.version.net_version())
            .map(Attribute::Location)
            .ok_or_else(|| AttributeError::NotEnoughDataFor("Location"))
    }

    pub fn decode_music_stinger(&self, bits: &mut BitGet<'_>) -> Result<Attribute, AttributeError> {
        if_chain! {
            if let Some(flag) = bits.read_bit();
            if let Some(cue) = bits.read_u32();
            if let Some(trigger) = bits.read_u8();
            then {
                Ok(Attribute::MusicStinger(MusicStinger {
                    flag,
                    cue,
                    trigger,
                }))
            } else {
                Err(AttributeError::NotEnoughDataFor("Music Stinger"))
            }
        }
    }

    pub fn decode_pickup(&self, bits: &mut BitGet<'_>) -> Result<Attribute, AttributeError> {
        if_chain! {
            if let Some(instigator_id) = bits.if_get(BitGet::read_u32);
            if let Some(picked_up) = bits.read_bit();
            then {
                Ok(Attribute::Pickup(Pickup {
                    instigator_id,
                    picked_up,
                }))
            } else {
                Err(AttributeError::NotEnoughDataFor("Pickup"))
            }
        }
    }

    pub fn decode_pickup_new(&self, bits: &mut BitGet<'_>) -> Result<Attribute, AttributeError> {
        if_chain! {
            if let Some(instigator_id) = bits.if_get(BitGet::read_u32);
            if let Some(picked_up) = bits.read_u8();
            then {
                Ok(Attribute::PickupNew(PickupNew {
                    instigator_id,
                    picked_up,
                }))
            } else {
                Err(AttributeError::NotEnoughDataFor("PickupNew"))
            }
        }
    }

    pub fn decode_qword(&self, bits: &mut BitGet<'_>) -> Result<Attribute, AttributeError> {
        bits.read_u64()
            .map(Attribute::QWord)
            .ok_or_else(|| AttributeError::NotEnoughDataFor("QWord"))
    }

    pub fn decode_welded(&self, bits: &mut BitGet<'_>) -> Result<Attribute, AttributeError> {
        if_chain! {
            if let Some(active) = bits.read_bit();
            if let Some(actor_id) = bits.read_u32();
            if let Some(offset) = Vector3f::decode(bits, self.version.net_version());
            if let Some(mass) = bits.read_f32();
            if let Some(rotation) = Rotation::decode(bits);
            then {
                Ok(Attribute::Welded(Welded {
                    active,
                    actor_id,
                    offset,
                    mass,
                    rotation,
                }))
            } else {
                Err(AttributeError::NotEnoughDataFor("Welded"))
            }
        }
    }

    pub fn decode_rotation(&self, bits: &mut BitGet<'_>) -> Result<Attribute, AttributeError> {
        let rot =
            Rotation::decode(bits).ok_or_else(|| AttributeError::NotEnoughDataFor("Rotation"))?;
        Ok(Attribute::Rotation(rot))
    }

    pub fn decode_title(&self, bits: &mut BitGet<'_>) -> Result<Attribute, AttributeError> {
        if_chain! {
            if let Some(unknown1) = bits.read_bit();
            if let Some(unknown2) = bits.read_bit();
            if let Some(unknown3) = bits.read_u32();
            if let Some(unknown4) = bits.read_u32();
            if let Some(unknown5) = bits.read_u32();
            if let Some(unknown6) = bits.read_u32();
            if let Some(unknown7) = bits.read_u32();
            if let Some(unknown8) = bits.read_bit();
            then {
                Ok(Attribute::Title(
                    unknown1,
                    unknown2,
                    unknown3,
                    unknown4,
                    unknown5,
                    unknown6,
                    unknown7,
                    unknown8,
                ))
            } else {
                Err(AttributeError::NotEnoughDataFor("Title"))
            }
        }
    }

    pub fn decode_team_paint(&self, bits: &mut BitGet<'_>) -> Result<Attribute, AttributeError> {
        if_chain! {
            if let Some(team) = bits.read_u8();
            if let Some(primary_color) = bits.read_u8();
            if let Some(accent_color) = bits.read_u8();
            if let Some(primary_finish) = bits.read_u32();
            if let Some(accent_finish) = bits.read_u32();
            then {
                Ok(Attribute::TeamPaint(TeamPaint {
                    team,
                    primary_color,
                    accent_color,
                    primary_finish,
                    accent_finish,
                }))
            } else {
                Err(AttributeError::NotEnoughDataFor("Team Paint"))
            }
        }
    }

    pub fn decode_rigid_body(&self, bits: &mut BitGet<'_>) -> Result<Attribute, AttributeError> {
        if_chain! {
            if let Some(sleeping) = bits.read_bit();
            if let Some(location) = Vector3f::decode(bits, self.version.net_version());

            if let Some(rotation) = if self.version.net_version() >= 7 {
                Quaternion::decode(bits)
            } else {
                Quaternion::decode_compressed(bits)
            };

            if let Some((linear_velocity, angular_velocity)) = if !sleeping {
                let lv = Vector3f::decode(bits, self.version.net_version());
                let av = Vector3f::decode(bits, self.version.net_version());
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
                    rotation,
                    linear_velocity,
                    angular_velocity,
                }))
            } else {
                Err(AttributeError::NotEnoughDataFor("Rigid Body"))
            }
        }
    }

    pub fn decode_not_implemented(
        &self,
        _bits: &mut BitGet<'_>,
    ) -> Result<Attribute, AttributeError> {
        Err(AttributeError::Unimplemented)
    }

    pub fn decode_string(&self, bits: &mut BitGet<'_>) -> Result<Attribute, AttributeError> {
        Ok(Attribute::String(decode_text(bits)?))
    }

    pub fn decode_unique_id(&self, bits: &mut BitGet<'_>) -> Result<Attribute, AttributeError> {
        decode_unique_id(bits, self.version.net_version()).map(Attribute::UniqueId)
    }

    pub fn decode_reservation(&self, bits: &mut BitGet<'_>) -> Result<Attribute, AttributeError> {
        if_chain! {
            if let Some(number) = bits.read_u32_bits(3);
            let unique = decode_unique_id(bits, self.version.net_version())?;
            if let Some(name) = if unique.system_id != 0 {
                Some(Some(decode_text(bits)?))
            } else {
                Some(None)
            };

            if let Some(unknown1) = bits.read_bit();
            if let Some(unknown2) = bits.read_bit();
            if let Some(unknown3) = if self.version >= VersionTriplet(868, 12, 0) {
                bits.read_u32_bits(6).map(|x| Some(x as u8))
            } else {
                Some(None)
            };

            then {
                Ok(Attribute::Reservation(Reservation {
                    number,
                    unique_id: unique,
                    name,
                    unknown1,
                    unknown2,
                    unknown3
                }))
            } else {
                Err(AttributeError::NotEnoughDataFor("Reservation"))
            }
        }
    }

    pub fn decode_party_leader(&self, bits: &mut BitGet<'_>) -> Result<Attribute, AttributeError> {
        if let Some(system_id) = bits.read_u8() {
            if system_id != 0 {
                let id =
                    decode_unique_id_with_system_id(bits, self.version.net_version(), system_id)?;
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
        bits: &mut BitGet<'_>,
    ) -> Result<Attribute, AttributeError> {
        if_chain! {
            let mutators = decode_text(bits)?;
            if let Some(joinable_by) = bits.read_u32();
            if let Some(max_players) = bits.read_u32();
            let game_name = decode_text(bits)?;
            let password = decode_text(bits)?;
            if let Some(flag) = bits.read_bit();

            then {
                Ok(Attribute::PrivateMatch(PrivateMatchSettings {
                    mutators,
                    joinable_by,
                    max_players,
                    game_name,
                    password,
                    flag,
                }))
            } else {
                Err(AttributeError::NotEnoughDataFor("Private Match"))
            }
        }
    }

    pub fn decode_loadout_online(
        &self,
        bits: &mut BitGet<'_>,
    ) -> Result<Attribute, AttributeError> {
        self.inner_decode_online_loadout(bits)
            .map(Attribute::LoadoutOnline)
            .ok_or_else(|| AttributeError::NotEnoughDataFor("Loadout Online"))
    }

    pub fn decode_loadouts_online(
        &self,
        bits: &mut BitGet<'_>,
    ) -> Result<Attribute, AttributeError> {
        if_chain! {
            if let Some(blue) = self.inner_decode_online_loadout(bits);
            if let Some(orange) = self.inner_decode_online_loadout(bits);
            if let Some(unknown1) = bits.read_bit();
            if let Some(unknown2) = bits.read_bit();
            then {
                Ok(Attribute::LoadoutsOnline(LoadoutsOnline {
                    blue,
                    orange,
                    unknown1,
                    unknown2,
                }))
            } else {
                Err(AttributeError::NotEnoughDataFor("Loadouts online"))
            }
        }
    }

    fn decode_product(&self, bits: &mut BitGet<'_>) -> Option<Product> {
        if_chain! {
            if let Some(unknown) = bits.read_bit();
            if let Some(obj_ind) = bits.read_u32();
            if let Some(val) = self.product_decoder.decode(bits, obj_ind);

            then {
                Some(Product {
                    unknown,
                    object_ind: obj_ind,
                    value: val,
                })
            } else {
                None
            }
        }
    }

    fn inner_decode_online_loadout(&self, bits: &mut BitGet<'_>) -> Option<Vec<Vec<Product>>> {
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

fn decode_explosion(bits: &mut BitGet<'_>, net_version: i32) -> Option<Explosion> {
    if_chain! {
        if let Some(flag) = bits.read_bit();
        if let Some(actor_id) = bits.read_u32();
        if let Some(location) = Vector3f::decode(bits, net_version);
        then {
            Some(Explosion {
                flag,
                actor_id,
                location,
            })
        } else {
            None
        }
    }
}

fn decode_text(bits: &mut BitGet<'_>) -> Result<String, AttributeError> {
    use std::cmp::Ordering;

    let size = bits
        .read_i32()
        .ok_or_else(|| AttributeError::NotEnoughDataFor("text string"))?;

    // A zero length string for attributes is fine (this differs from the replay header where we
    // never see zero length strings)
    match size.cmp(&0) {
        Ordering::Equal => Ok(String::from("")),
        Ordering::Less => size
            .checked_mul(-2)
            .ok_or_else(|| AttributeError::TooBigString(size))
            .and_then(|len| {
                bits.read_bytes(len)
                    .and_then(|data| decode_utf16(&data[..]).ok())
                    .ok_or_else(|| AttributeError::TooBigString(len))
            }),
        Ordering::Greater => bits
            .read_bytes(size)
            .and_then(|data| decode_windows1252(&data[..]).ok())
            .ok_or_else(|| AttributeError::TooBigString(size)),
    }
}

fn decode_loadout_specials(
    bits: &mut BitGet<'_>,
) -> Option<(Option<u32>, Option<u32>, Option<u32>)> {
    if_chain! {
        if let Some(engine_audio) = bits.read_u32();
        if let Some(trail) = bits.read_u32();
        if let Some(goal_explosion) = bits.read_u32();
        then {
            Some((Some(engine_audio), Some(trail), Some(goal_explosion)))
        } else {
            None
        }
    }
}

fn decode_loadout(bits: &mut BitGet<'_>) -> Option<Loadout> {
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

        if let Some((engine_audio, trail, goal_explosion)) = if version >= 16 {
            decode_loadout_specials(bits)
        } else {
            Some((None, None, None))
        };

        if let Some(banner) = if version >= 17 {
            bits.read_u32().map(Some)
        } else {
            Some(None)
        };

        if let Some(unknown3) = if version >= 19 {
            bits.read_u32().map(Some)
        } else {
            Some(None)
        };

        if let Some(_unknown4) = if version >= 22 {
            bits.read_u32()
                .and(bits.read_u32())
                .and(bits.read_u32())
        } else {
            Some(0)
        };


        then {
            Some(Loadout {
                version,
                body,
                decal,
                wheels,
                rocket_trail,
                antenna,
                topper,
                unknown1,
                unknown2,
                engine_audio,
                trail,
                goal_explosion,
                banner,
                unknown3,
            })
        } else {
            None
        }
    }
}

fn decode_unique_id(bits: &mut BitGet<'_>, net_version: i32) -> Result<UniqueId, AttributeError> {
    let system_id = bits
        .read_u8()
        .ok_or_else(|| AttributeError::NotEnoughDataFor("System id"))?;
    decode_unique_id_with_system_id(bits, net_version, system_id)
}

fn decode_unique_id_with_system_id(
    bits: &mut BitGet<'_>,
    net_version: i32,
    system_id: u8,
) -> Result<UniqueId, AttributeError> {
    let remote_id = match system_id {
        0 => bits
            .read_u32_bits(24)
            .ok_or_else(|| AttributeError::NotEnoughDataFor("SplitScreen"))
            .map(RemoteId::SplitScreen),
        1 => bits
            .read_u64()
            .ok_or_else(|| AttributeError::NotEnoughDataFor("Steam"))
            .map(RemoteId::Steam),
        2 => {
            let name_bytes = bits
                .read_bytes(16)
                .ok_or_else(|| AttributeError::NotEnoughDataFor("PS4 Name"))?
                .iter()
                .take_while(|&&x| x != 0)
                .cloned()
                .collect::<Vec<u8>>();

            let (name, _) = WINDOWS_1252.decode_without_bom_handling(&name_bytes[..]);
            let to_read = if net_version >= 1 { 16 } else { 8 };

            let unknown1 = bits
                .read_bytes(to_read)
                .ok_or_else(|| AttributeError::NotEnoughDataFor("PS4 Unknown"))
                .map(Cow::into_owned)?;

            let online_id = bits
                .read_u64()
                .ok_or_else(|| AttributeError::NotEnoughDataFor("PS4 ID"))?;

            Ok(RemoteId::PlayStation(Ps4Id {
                name: name.to_string(),
                unknown1,
                online_id,
            }))
        }
        4 => bits
            .read_u64()
            .ok_or_else(|| AttributeError::NotEnoughDataFor("Xbox"))
            .map(RemoteId::Xbox),
        5 => bits
            .read_u64()
            .ok_or_else(|| AttributeError::NotEnoughDataFor("QQ ID"))
            .map(RemoteId::QQ),
        6 => {
            let online_id = bits
                .read_u64()
                .ok_or_else(|| AttributeError::NotEnoughDataFor("Switch ID"))?;

            let unknown1 = bits
                .read_bytes(24)
                .ok_or_else(|| AttributeError::NotEnoughDataFor("Switch ID Unknown"))
                .map(Cow::into_owned)?;

            Ok(RemoteId::Switch(SwitchId {
                online_id,
                unknown1,
            }))
        }
        7 => {
            let online_id = bits
                .read_u64()
                .ok_or_else(|| AttributeError::NotEnoughDataFor("PsyNet ID"))?;

            if net_version < 10 {
                let unknown1 = bits
                    .read_bytes(24)
                    .ok_or_else(|| AttributeError::NotEnoughDataFor("PsyNet ID Unknown"))
                    .map(Cow::into_owned)?;

                Ok(RemoteId::PsyNet(PsyNetId {
                    online_id,
                    unknown1,
                }))
            } else {
                Ok(RemoteId::PsyNet(PsyNetId {
                    online_id,
                    ..Default::default()
                }))
            }
        }
        x => Err(AttributeError::UnrecognizedRemoteId(x)),
    }?;

    let local_id = bits
        .read_u8()
        .ok_or_else(|| AttributeError::NotEnoughDataFor("UniqueId local_id"))?;
    Ok(UniqueId {
        system_id,
        remote_id,
        local_id,
    })
}
