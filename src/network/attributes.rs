use crate::errors::AttributeError;
use crate::network::{ActorId, ObjectId, Quaternion, Rotation, Vector3f, VersionTriplet};
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
    DemolishFx,
    Enum,
    Explosion,
    ExtendedExplosion,
    FlaggedByte,
    ActiveActor,
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

/// The attributes for updated actors in the network data.
///
/// The vast majority of attributes in the network data are rigid bodies. As a performance
/// improvent, any attribute variant larger than the size of a rigid body is moved to the heap (ie:
/// `Box::new`). This change increased throughput by 40%.
#[derive(Debug, Clone, PartialEq, Serialize)]
pub enum Attribute {
    Boolean(bool),
    Byte(u8),
    AppliedDamage(AppliedDamage),
    DamageState(DamageState),
    CamSettings(Box<CamSettings>),
    ClubColors(ClubColors),
    Demolish(Box<Demolish>),
    DemolishFx(Box<DemolishFx>),
    Enum(u16),
    Explosion(Explosion),
    ExtendedExplosion(ExtendedExplosion),
    FlaggedByte(bool, u8),
    ActiveActor(ActiveActor),
    Float(f32),
    GameMode(u8, u8),
    Int(i32),

    #[serde(serialize_with = "crate::serde_utils::display_it")]
    Int64(i64),
    Loadout(Box<Loadout>),
    TeamLoadout(Box<TeamLoadout>),
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
    UniqueId(Box<UniqueId>),
    Reservation(Box<Reservation>),
    PartyLeader(Option<Box<UniqueId>>),
    PrivateMatch(Box<PrivateMatchSettings>),
    LoadoutOnline(Vec<Vec<Product>>),
    LoadoutsOnline(LoadoutsOnline),
    StatEvent(StatEvent),
    Rotation(Rotation),
    RepStatTitle(RepStatTitle),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub struct ActiveActor {
    pub active: bool,
    pub actor: ActorId,
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize)]
pub struct CamSettings {
    pub fov: f32,
    pub height: f32,
    pub angle: f32,
    pub distance: f32,
    pub stiffness: f32,
    pub swivel: f32,
    pub transition: Option<f32>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub struct ClubColors {
    pub blue_flag: bool,
    pub blue_color: u8,
    pub orange_flag: bool,
    pub orange_color: u8,
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize)]
pub struct AppliedDamage {
    pub id: u8,
    pub position: Vector3f,
    pub damage_index: i32,
    pub total_damage: i32,
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize)]
pub struct DamageState {
    /// State of the dropshot tile (0 - undamaged, 1 - damaged, 2 - destroyed)
    pub tile_state: u8,

    /// True if damaged
    pub damaged: bool,

    /// Player actor that inflicted the damage
    pub offender: ActorId,

    /// Position of the ball at the time of the damage
    pub ball_position: Vector3f,

    /// True for the dropshot tile that was hit by the ball (center tile of the damage area)
    pub direct_hit: bool,
    pub unknown1: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize)]
pub struct Demolish {
    pub attacker_flag: bool,
    pub attacker: ActorId,
    pub victim_flag: bool,
    pub victim: ActorId,
    pub attack_velocity: Vector3f,
    pub victim_velocity: Vector3f,
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize)]
pub struct DemolishFx {
    pub custom_demo_flag: bool,
    pub custom_demo_id: i32,
    pub attacker_flag: bool,
    pub attacker: ActorId,
    pub victim_flag: bool,
    pub victim: ActorId,
    pub attack_velocity: Vector3f,
    pub victim_velocity: Vector3f,
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize)]
pub struct Explosion {
    pub flag: bool,
    pub actor: ActorId,
    pub location: Vector3f,
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize)]
pub struct ExtendedExplosion {
    pub explosion: Explosion,
    pub unknown1: bool,
    pub secondary_actor: ActorId,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
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
    pub product_id: Option<u32>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub struct TeamLoadout {
    pub blue: Loadout,
    pub orange: Loadout,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub struct StatEvent {
    pub unknown1: bool,
    pub object_id: i32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub struct MusicStinger {
    pub flag: bool,
    pub cue: u32,
    pub trigger: u8,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub struct Pickup {
    pub instigator: Option<ActorId>,
    pub picked_up: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub struct PickupNew {
    pub instigator: Option<ActorId>,
    pub picked_up: u8,
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize)]
pub struct Welded {
    pub active: bool,
    pub actor: ActorId,
    pub offset: Vector3f,
    pub mass: f32,
    pub rotation: Rotation,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub struct TeamPaint {
    pub team: u8,
    pub primary_color: u8,
    pub accent_color: u8,
    pub primary_finish: u32,
    pub accent_finish: u32,
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize)]
pub struct RigidBody {
    pub sleeping: bool,
    pub location: Vector3f,
    pub rotation: Quaternion,
    pub linear_velocity: Option<Vector3f>,
    pub angular_velocity: Option<Vector3f>,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct UniqueId {
    pub system_id: u8,
    pub remote_id: RemoteId,
    pub local_id: u8,
}

#[derive(Debug, Default, Clone, PartialEq, Serialize)]
pub struct PsyNetId {
    #[serde(serialize_with = "crate::serde_utils::display_it")]
    pub online_id: u64,
    pub unknown1: Vec<u8>,
}

#[derive(Debug, Default, Clone, PartialEq, Serialize)]
pub struct SwitchId {
    #[serde(serialize_with = "crate::serde_utils::display_it")]
    pub online_id: u64,
    pub unknown1: Vec<u8>,
}

#[derive(Debug, Default, Clone, PartialEq, Serialize)]
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
    Epic(String),
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct Reservation {
    pub number: u32,
    pub unique_id: UniqueId,
    pub name: Option<String>,
    pub unknown1: bool,
    pub unknown2: bool,
    pub unknown3: Option<u8>,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct PrivateMatchSettings {
    pub mutators: String,
    pub joinable_by: u32,
    pub max_players: u32,
    pub game_name: String,
    pub password: String,
    pub flag: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct Product {
    pub unknown: bool,
    pub object_ind: u32,
    pub value: ProductValue,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
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
    NewColor(i32),
    OldPaint(u32),
    NewPaint(u32),
    Title(String),
    SpecialEdition(u32),
    OldTeamEdition(u32),
    NewTeamEdition(u32),
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
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
                bits.read_i32().map(ProductValue::NewColor)
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
            AttributeTag::DemolishFx => self.decode_demolish_fx(bits),
            AttributeTag::Enum => self.decode_enum(bits),
            AttributeTag::Explosion => self.decode_explosion(bits),
            AttributeTag::ExtendedExplosion => self.decode_extended_explosion(bits),
            AttributeTag::ActiveActor => self.decode_active_actor(bits),
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

    fn _decode_flagged_byte(&self, bits: &mut BitGet<'_>) -> Option<Attribute> {
        let b = get!(bits.read_bit());
        let data = get!(bits.read_u8());
        Some(Attribute::FlaggedByte(b, data))
    }

    pub fn decode_flagged_byte(&self, bits: &mut BitGet<'_>) -> Result<Attribute, AttributeError> {
        self._decode_flagged_byte(bits)
            .ok_or_else(|| AttributeError::NotEnoughDataFor("FlaggedByte"))
    }

    pub fn decode_boolean(&self, bits: &mut BitGet<'_>) -> Result<Attribute, AttributeError> {
        bits.read_bit()
            .map(Attribute::Boolean)
            .ok_or_else(|| AttributeError::NotEnoughDataFor("Boolean"))
    }

    pub fn _decode_applied_damage(&self, bits: &mut BitGet<'_>) -> Option<AppliedDamage> {
        let id = get!(bits.read_u8());
        let position = get!(Vector3f::decode(bits, self.version.net_version()));
        let damage_index = get!(bits.read_i32());
        let total_damage = get!(bits.read_i32());
        Some(AppliedDamage {
            id,
            position,
            damage_index,
            total_damage,
        })
    }

    pub fn decode_applied_damage(
        &self,
        bits: &mut BitGet<'_>,
    ) -> Result<Attribute, AttributeError> {
        self._decode_applied_damage(bits)
            .map(Attribute::AppliedDamage)
            .ok_or_else(|| AttributeError::NotEnoughDataFor("Applied Damage"))
    }

    fn _decode_damage_state(&self, bits: &mut BitGet<'_>) -> Option<DamageState> {
        let tile_state = get!(bits.read_u8());
        let damaged = get!(bits.read_bit());
        let offender = get!(bits.read_i32().map(ActorId));
        let ball_position = get!(Vector3f::decode(bits, self.version.net_version()));
        let direct_hit = get!(bits.read_bit());
        let unknown1 = get!(bits.read_bit());
        Some(DamageState {
            tile_state,
            damaged,
            offender,
            ball_position,
            direct_hit,
            unknown1,
        })
    }

    pub fn decode_damage_state(&self, bits: &mut BitGet<'_>) -> Result<Attribute, AttributeError> {
        self._decode_damage_state(bits)
            .map(Attribute::DamageState)
            .ok_or_else(|| AttributeError::NotEnoughDataFor("Damage State"))
    }

    fn _decode_cam_settings(&self, bits: &mut BitGet<'_>) -> Option<CamSettings> {
        let fov = get!(bits.read_f32());
        let height = get!(bits.read_f32());
        let angle = get!(bits.read_f32());
        let distance = get!(bits.read_f32());
        let stiffness = get!(bits.read_f32());
        let swivel = get!(bits.read_f32());
        let transition = if self.version >= VersionTriplet(868, 20, 0) {
            Some(get!(bits.read_f32()))
        } else {
            None
        };

        Some(CamSettings {
            fov,
            height,
            angle,
            distance,
            stiffness,
            swivel,
            transition,
        })
    }

    pub fn decode_cam_settings(&self, bits: &mut BitGet<'_>) -> Result<Attribute, AttributeError> {
        self._decode_cam_settings(bits)
            .map(Box::new)
            .map(Attribute::CamSettings)
            .ok_or_else(|| AttributeError::NotEnoughDataFor("Cam Settings"))
    }

    fn _decode_club_colors(&self, bits: &mut BitGet<'_>) -> Option<ClubColors> {
        let blue_flag = get!(bits.read_bit());
        let blue_color = get!(bits.read_u8());
        let orange_flag = get!(bits.read_bit());
        let orange_color = get!(bits.read_u8());
        Some(ClubColors {
            blue_flag,
            blue_color,
            orange_flag,
            orange_color,
        })
    }

    pub fn decode_club_colors(&self, bits: &mut BitGet<'_>) -> Result<Attribute, AttributeError> {
        self._decode_club_colors(bits)
            .map(Attribute::ClubColors)
            .ok_or_else(|| AttributeError::NotEnoughDataFor("Club Colors"))
    }

    fn _decode_demolish(&self, bits: &mut BitGet<'_>) -> Option<Demolish> {
        let attacker_flag = get!(bits.read_bit());
        let attacker = get!(bits.read_i32().map(ActorId));
        let victim_flag = get!(bits.read_bit());
        let victim = get!(bits.read_i32().map(ActorId));
        let attack_velocity = get!(Vector3f::decode(bits, self.version.net_version()));
        let victim_velocity = get!(Vector3f::decode(bits, self.version.net_version()));
        Some(Demolish {
            attacker_flag,
            attacker,
            victim_flag,
            victim,
            attack_velocity,
            victim_velocity,
        })
    }

    pub fn decode_demolish(&self, bits: &mut BitGet<'_>) -> Result<Attribute, AttributeError> {
        self._decode_demolish(bits)
            .map(Box::new)
            .map(Attribute::Demolish)
            .ok_or_else(|| AttributeError::NotEnoughDataFor("Demolish"))
    }

    pub fn _decode_demolish_fx(&self, bits: &mut BitGet<'_>) -> Option<DemolishFx> {
        let custom_demo_flag = get!(bits.read_bit());
        let custom_demo_id = get!(bits.read_i32());
        let attacker_flag = get!(bits.read_bit());
        let attacker = get!(bits.read_i32().map(ActorId));
        let victim_flag = get!(bits.read_bit());
        let victim = get!(bits.read_i32().map(ActorId));
        let attack_velocity = get!(Vector3f::decode(bits, self.version.net_version()));
        let victim_velocity = get!(Vector3f::decode(bits, self.version.net_version()));

        Some(DemolishFx {
            custom_demo_flag,
            custom_demo_id,
            attacker_flag,
            attacker,
            victim_flag,
            victim,
            attack_velocity,
            victim_velocity,
        })
    }

    pub fn decode_demolish_fx(&self, bits: &mut BitGet<'_>) -> Result<Attribute, AttributeError> {
        self._decode_demolish_fx(bits)
            .map(Box::new)
            .map(Attribute::DemolishFx)
            .ok_or_else(|| AttributeError::NotEnoughDataFor("DemolishFx"))
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

    fn _decode_stat_event(&self, bits: &mut BitGet<'_>) -> Option<StatEvent> {
        let unknown1 = get!(bits.read_bit());
        let object_id = get!(bits.read_i32());
        Some(StatEvent {
            unknown1,
            object_id,
        })
    }

    pub fn decode_stat_event(&self, bits: &mut BitGet<'_>) -> Result<Attribute, AttributeError> {
        self._decode_stat_event(bits)
            .map(Attribute::StatEvent)
            .ok_or_else(|| AttributeError::NotEnoughDataFor("Stat Event"))
    }

    pub fn decode_rep_stat_title(
        &self,
        bits: &mut BitGet<'_>,
    ) -> Result<Attribute, AttributeError> {
        let unknown = bits
            .read_bit()
            .ok_or_else(|| AttributeError::NotEnoughDataFor("RepStatTitle"))?;
        let name = decode_text(bits)?;
        let unknown2 = bits
            .read_bit()
            .ok_or_else(|| AttributeError::NotEnoughDataFor("RepStatTitle"))?;
        let index = bits
            .read_u32()
            .ok_or_else(|| AttributeError::NotEnoughDataFor("RepStatTitle"))?;
        let value = bits
            .read_u32()
            .ok_or_else(|| AttributeError::NotEnoughDataFor("RepStatTitle"))?;
        Ok(Attribute::RepStatTitle(RepStatTitle {
            unknown,
            name,
            unknown2,
            index,
            value,
        }))
    }

    fn _decode_extended_explosion(&self, bits: &mut BitGet<'_>) -> Option<ExtendedExplosion> {
        let explosion = get!(decode_explosion(bits, self.version.net_version()));
        let unknown1 = get!(bits.read_bit());
        let secondary_actor = get!(bits.read_i32().map(ActorId));
        Some(ExtendedExplosion {
            explosion,
            unknown1,
            secondary_actor,
        })
    }

    pub fn decode_extended_explosion(
        &self,
        bits: &mut BitGet<'_>,
    ) -> Result<Attribute, AttributeError> {
        self._decode_extended_explosion(bits)
            .map(Attribute::ExtendedExplosion)
            .ok_or_else(|| AttributeError::NotEnoughDataFor("Extended Explosion"))
    }

    pub fn decode_active_actor(&self, bits: &mut BitGet<'_>) -> Result<Attribute, AttributeError> {
        let active = bits
            .read_bit()
            .ok_or_else(|| AttributeError::NotEnoughDataFor("Flagged"))?;
        let actor = bits
            .read_i32()
            .map(ActorId)
            .ok_or_else(|| AttributeError::NotEnoughDataFor("Flagged"))?;
        Ok(Attribute::ActiveActor(ActiveActor { active, actor }))
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
            .map(Box::new)
            .map(Attribute::Loadout)
            .ok_or_else(|| AttributeError::NotEnoughDataFor("Loadout"))
    }

    pub fn decode_team_loadout(&self, bits: &mut BitGet<'_>) -> Result<Attribute, AttributeError> {
        let blue =
            decode_loadout(bits).ok_or_else(|| AttributeError::NotEnoughDataFor("Team Loadout"))?;
        let orange =
            decode_loadout(bits).ok_or_else(|| AttributeError::NotEnoughDataFor("Team Loadout"))?;
        Ok(Attribute::TeamLoadout(Box::new(TeamLoadout {
            blue,
            orange,
        })))
    }

    pub fn decode_location(&self, bits: &mut BitGet<'_>) -> Result<Attribute, AttributeError> {
        Vector3f::decode(bits, self.version.net_version())
            .map(Attribute::Location)
            .ok_or_else(|| AttributeError::NotEnoughDataFor("Location"))
    }

    fn _decode_music_stinger(&self, bits: &mut BitGet<'_>) -> Option<MusicStinger> {
        let flag = get!(bits.read_bit());
        let cue = get!(bits.read_u32());
        let trigger = get!(bits.read_u8());
        Some(MusicStinger { flag, cue, trigger })
    }

    pub fn decode_music_stinger(&self, bits: &mut BitGet<'_>) -> Result<Attribute, AttributeError> {
        self._decode_music_stinger(bits)
            .map(Attribute::MusicStinger)
            .ok_or_else(|| AttributeError::NotEnoughDataFor("Music Stinger"))
    }

    pub fn decode_pickup(&self, bits: &mut BitGet<'_>) -> Result<Attribute, AttributeError> {
        let instigator = bits
            .if_get(BitGet::read_i32)
            .map(|x| x.map(ActorId))
            .ok_or_else(|| AttributeError::NotEnoughDataFor("Pickup"))?;
        let picked_up = bits
            .read_bit()
            .ok_or_else(|| AttributeError::NotEnoughDataFor("Pickup"))?;
        Ok(Attribute::Pickup(Pickup {
            instigator,
            picked_up,
        }))
    }

    pub fn decode_pickup_new(&self, bits: &mut BitGet<'_>) -> Result<Attribute, AttributeError> {
        let instigator = bits
            .if_get(BitGet::read_i32)
            .map(|x| x.map(ActorId))
            .ok_or_else(|| AttributeError::NotEnoughDataFor("PickupNew"))?;
        let picked_up = bits
            .read_u8()
            .ok_or_else(|| AttributeError::NotEnoughDataFor("PickupNew"))?;
        Ok(Attribute::PickupNew(PickupNew {
            instigator,
            picked_up,
        }))
    }

    pub fn decode_qword(&self, bits: &mut BitGet<'_>) -> Result<Attribute, AttributeError> {
        bits.read_u64()
            .map(Attribute::QWord)
            .ok_or_else(|| AttributeError::NotEnoughDataFor("QWord"))
    }

    fn _decode_welded(&self, bits: &mut BitGet<'_>) -> Option<Welded> {
        let active = get!(bits.read_bit());
        let actor = get!(bits.read_i32().map(ActorId));
        let offset = get!(Vector3f::decode(bits, self.version.net_version()));
        let mass = get!(bits.read_f32());
        let rotation = get!(Rotation::decode(bits));
        Some(Welded {
            active,
            actor,
            offset,
            mass,
            rotation,
        })
    }

    pub fn decode_welded(&self, bits: &mut BitGet<'_>) -> Result<Attribute, AttributeError> {
        self._decode_welded(bits)
            .map(Attribute::Welded)
            .ok_or_else(|| AttributeError::NotEnoughDataFor("Welded"))
    }

    pub fn decode_rotation(&self, bits: &mut BitGet<'_>) -> Result<Attribute, AttributeError> {
        let rot =
            Rotation::decode(bits).ok_or_else(|| AttributeError::NotEnoughDataFor("Rotation"))?;
        Ok(Attribute::Rotation(rot))
    }

    fn _decode_title(&self, bits: &mut BitGet<'_>) -> Option<Attribute> {
        let unknown1 = get!(bits.read_bit());
        let unknown2 = get!(bits.read_bit());
        let unknown3 = get!(bits.read_u32());
        let unknown4 = get!(bits.read_u32());
        let unknown5 = get!(bits.read_u32());
        let unknown6 = get!(bits.read_u32());
        let unknown7 = get!(bits.read_u32());
        let unknown8 = get!(bits.read_bit());
        Some(Attribute::Title(
            unknown1, unknown2, unknown3, unknown4, unknown5, unknown6, unknown7, unknown8,
        ))
    }
    pub fn decode_title(&self, bits: &mut BitGet<'_>) -> Result<Attribute, AttributeError> {
        self._decode_title(bits)
            .ok_or_else(|| AttributeError::NotEnoughDataFor("Title"))
    }

    fn _decode_team_paint(&self, bits: &mut BitGet<'_>) -> Option<TeamPaint> {
        let team = get!(bits.read_u8());
        let primary_color = get!(bits.read_u8());
        let accent_color = get!(bits.read_u8());
        let primary_finish = get!(bits.read_u32());
        let accent_finish = get!(bits.read_u32());

        Some(TeamPaint {
            team,
            primary_color,
            accent_color,
            primary_finish,
            accent_finish,
        })
    }

    pub fn decode_team_paint(&self, bits: &mut BitGet<'_>) -> Result<Attribute, AttributeError> {
        self._decode_team_paint(bits)
            .map(Attribute::TeamPaint)
            .ok_or_else(|| AttributeError::NotEnoughDataFor("Team Paint"))
    }

    fn _decode_rigid_body(&self, bits: &mut BitGet<'_>) -> Option<RigidBody> {
        let sleeping = get!(bits.read_bit());
        let location = get!(Vector3f::decode(bits, self.version.net_version()));

        let rotation = if self.version.net_version() >= 7 {
            get!(Quaternion::decode(bits))
        } else {
            get!(Quaternion::decode_compressed(bits))
        };

        let mut linear_velocity = None;
        let mut angular_velocity = None;

        if !sleeping {
            linear_velocity = Some(get!(Vector3f::decode(bits, self.version.net_version())));
            angular_velocity = Some(get!(Vector3f::decode(bits, self.version.net_version())));
        }

        Some(RigidBody {
            sleeping,
            location,
            rotation,
            linear_velocity,
            angular_velocity,
        })
    }

    pub fn decode_rigid_body(&self, bits: &mut BitGet<'_>) -> Result<Attribute, AttributeError> {
        self._decode_rigid_body(bits)
            .map(Attribute::RigidBody)
            .ok_or_else(|| AttributeError::NotEnoughDataFor("Rigid Body"))
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
        decode_unique_id(bits, self.version.net_version())
            .map(Box::new)
            .map(Attribute::UniqueId)
    }

    pub fn decode_reservation(&self, bits: &mut BitGet<'_>) -> Result<Attribute, AttributeError> {
        let component = "Reservation";
        let number = get_or!(bits.read_u32_bits(3), component)?;
        let unique = decode_unique_id(bits, self.version.net_version())?;
        let mut name = None;
        if unique.system_id != 0 {
            name = Some(decode_text(bits)?);
        }

        let unknown1 = get_or!(bits.read_bit(), component)?;
        let unknown2 = get_or!(bits.read_bit(), component)?;
        let mut unknown3 = None;
        if self.version >= VersionTriplet(868, 12, 0) {
            unknown3 = get_or!(bits.read_u32_bits(6).map(|x| Some(x as u8)), component)?;
        };

        Ok(Attribute::Reservation(Box::new(Reservation {
            number,
            unique_id: unique,
            name,
            unknown1,
            unknown2,
            unknown3,
        })))
    }

    pub fn decode_party_leader(&self, bits: &mut BitGet<'_>) -> Result<Attribute, AttributeError> {
        if let Some(system_id) = bits.read_u8() {
            if system_id != 0 {
                let id =
                    decode_unique_id_with_system_id(bits, self.version.net_version(), system_id)?;
                Ok(Attribute::PartyLeader(Some(Box::new(id))))
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
        let component = "Private Match";
        let mutators = decode_text(bits)?;
        let joinable_by = get_or!(bits.read_u32(), component)?;
        let max_players = get_or!(bits.read_u32(), component)?;
        let game_name = decode_text(bits)?;
        let password = decode_text(bits)?;
        let flag = get_or!(bits.read_bit(), component)?;

        Ok(Attribute::PrivateMatch(Box::new(PrivateMatchSettings {
            mutators,
            joinable_by,
            max_players,
            game_name,
            password,
            flag,
        })))
    }

    pub fn decode_loadout_online(
        &self,
        bits: &mut BitGet<'_>,
    ) -> Result<Attribute, AttributeError> {
        self.inner_decode_online_loadout(bits)
            .map(Attribute::LoadoutOnline)
            .ok_or_else(|| AttributeError::NotEnoughDataFor("Loadout Online"))
    }

    fn _decode_loadouts_online(&self, bits: &mut BitGet<'_>) -> Option<LoadoutsOnline> {
        let blue = get!(self.inner_decode_online_loadout(bits));
        let orange = get!(self.inner_decode_online_loadout(bits));
        let unknown1 = get!(bits.read_bit());
        let unknown2 = get!(bits.read_bit());
        Some(LoadoutsOnline {
            blue,
            orange,
            unknown1,
            unknown2,
        })
    }

    pub fn decode_loadouts_online(
        &self,
        bits: &mut BitGet<'_>,
    ) -> Result<Attribute, AttributeError> {
        self._decode_loadouts_online(bits)
            .map(Attribute::LoadoutsOnline)
            .ok_or_else(|| AttributeError::NotEnoughDataFor("Loadouts online"))
    }

    fn decode_product(&self, bits: &mut BitGet<'_>) -> Option<Product> {
        let unknown = get!(bits.read_bit());
        let obj_ind = get!(bits.read_u32());
        let val = get!(self.product_decoder.decode(bits, obj_ind));

        Some(Product {
            unknown,
            object_ind: obj_ind,
            value: val,
        })
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
    let flag = get!(bits.read_bit());
    let actor = get!(bits.read_i32().map(ActorId));
    let location = get!(Vector3f::decode(bits, net_version));
    Some(Explosion {
        flag,
        actor,
        location,
    })
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
    let engine_audio = get!(bits.read_u32());
    let trail = get!(bits.read_u32());
    let goal_explosion = get!(bits.read_u32());
    Some((Some(engine_audio), Some(trail), Some(goal_explosion)))
}

fn decode_loadout(bits: &mut BitGet<'_>) -> Option<Loadout> {
    let version = get!(bits.read_u8());
    let body = get!(bits.read_u32());
    let decal = get!(bits.read_u32());
    let wheels = get!(bits.read_u32());
    let rocket_trail = get!(bits.read_u32());
    let antenna = get!(bits.read_u32());
    let topper = get!(bits.read_u32());
    let unknown1 = get!(bits.read_u32());
    let unknown2 = if version > 10 {
        Some(get!(bits.read_u32()))
    } else {
        None
    };

    let (engine_audio, trail, goal_explosion) = if version >= 16 {
        get!(decode_loadout_specials(bits))
    } else {
        (None, None, None)
    };

    let banner = if version >= 17 {
        Some(get!(bits.read_u32()))
    } else {
        None
    };

    let product_id = if version >= 19 {
        Some(get!(bits.read_u32()))
    } else {
        None
    };

    if version >= 22 {
        let _ = get!(bits.read_u32());
        let _ = get!(bits.read_u32());
        let _ = get!(bits.read_u32());
    }

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
        product_id,
    })
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
        11 => Ok(RemoteId::Epic(decode_text(bits)?)),
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_size_of_rigid_body() {
        assert_eq!(::std::mem::size_of::<RigidBody>(), 64);
    }

    #[test]
    fn test_size_of_attribute() {
        assert_eq!(
            ::std::mem::size_of::<Attribute>(),
            ::std::mem::size_of::<RigidBody>() + ::std::mem::size_of::<usize>()
        );
    }
}
