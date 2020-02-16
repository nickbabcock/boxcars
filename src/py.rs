use pyo3::{Python, PyObject, IntoPy};
use pyo3::types::{PyDict, PyList};
use crate::network::attributes::{RemoteId, ProductValue};
use crate::{Attribute, StreamId, ObjectId, ActorId, HeaderProp};

impl pyo3::IntoPy<PyObject> for RemoteId {
    fn into_py(self, py: Python) -> PyObject {
        let dict = PyDict::new(py);
        let res = match self {
            RemoteId::PlayStation(x) => {
                let item = IntoPy::<PyObject>::into_py(x, py);
                dict.set_item("PlayStation", item)
            }
            RemoteId::PsyNet(x) => {
                let item = IntoPy::<PyObject>::into_py(x, py);
                dict.set_item("PsyNet", item)
            }
            RemoteId::SplitScreen(x) => {
                let item = IntoPy::<PyObject>::into_py(x, py);
                dict.set_item("SplitScreen", item)
            }
            RemoteId::Steam(x) => {
                let item = IntoPy::<PyObject>::into_py(x, py);
                dict.set_item("SplitScreen", item)
            }
            RemoteId::Switch(x) => {
                let item = IntoPy::<PyObject>::into_py(x, py);
                dict.set_item("SplitScreen", item)
            }
            RemoteId::Xbox(x) => {
                let item = IntoPy::<PyObject>::into_py(x, py);
                dict.set_item("SplitScreen", item)
            }
            RemoteId::QQ(x) => {
                let item = IntoPy::<PyObject>::into_py(x, py);
                dict.set_item("SplitScreen", item)
            }
        };

        res.unwrap();
        dict.into()
    }
}

impl pyo3::IntoPy<PyObject> for ProductValue {
    fn into_py(self, py: Python) -> PyObject {
        let dict = PyDict::new(py);
        let res = match self {
            ProductValue::NoColor => {
                let item = IntoPy::<PyObject>::into_py((), py);
                dict.set_item("NoColor", item)
            }
            ProductValue::Absent => {
                let item = IntoPy::<PyObject>::into_py((), py);
                dict.set_item("Absent", item)
            }
            ProductValue::OldColor(x) => {
                let item = IntoPy::<PyObject>::into_py(x, py);
                dict.set_item("OldColor", item)
            }
            ProductValue::NewColor(x) => {
                let item = IntoPy::<PyObject>::into_py(x, py);
                dict.set_item("NewColor", item)
            }
            ProductValue::OldPaint(x) => {
                let item = IntoPy::<PyObject>::into_py(x, py);
                dict.set_item("OldPaint", item)
            }
            ProductValue::NewPaint(x) => {
                let item = IntoPy::<PyObject>::into_py(x, py);
                dict.set_item("NewPaint", item)
            }
            ProductValue::Title(x) => {
                let item = IntoPy::<PyObject>::into_py(x, py);
                dict.set_item("Title", item)
            }
            ProductValue::SpecialEdition(x) => {
                let item = IntoPy::<PyObject>::into_py(x, py);
                dict.set_item("SpecialEdition", item)
            }
            ProductValue::OldTeamEdition(x) => {
                let item = IntoPy::<PyObject>::into_py(x, py);
                dict.set_item("OldTeamEdition", item)
            }
            ProductValue::NewTeamEdition(x) => {
                let item = IntoPy::<PyObject>::into_py(x, py);
                dict.set_item("NewTeamEdition", item)
            }
        };

        res.unwrap();
        dict.into()
    }
}

impl pyo3::IntoPy<PyObject> for Attribute {
    fn into_py(self, py: Python) -> PyObject {
        let dict = PyDict::new(py);
        let res = match self {
            Attribute::Boolean(x) => {
                let item = IntoPy::<PyObject>::into_py(x, py);
                dict.set_item("Boolean", item)
            }
            Attribute::Byte(x) => {
                let item = IntoPy::<PyObject>::into_py(x, py);
                dict.set_item("Byte", item)
            }
            Attribute::AppliedDamage(x1, x2, x3, x4) => {
                let list = PyList::empty(py);
                let x2 = IntoPy::<PyObject>::into_py(x2, py);
                list.append(x1).expect("list append");
                list.append(x2).expect("list append");
                list.append(x3).expect("list append");
                list.append(x4).expect("list append");
                dict.set_item("AppliedDamage", list)
            }
            Attribute::DamageState(x1, x2, x3, x4, x5, x6) => {
                let list = PyList::empty(py);
                let x4 = IntoPy::<PyObject>::into_py(x4, py);
                list.append(x1).expect("list append");
                list.append(x2).expect("list append");
                list.append(x3).expect("list append");
                list.append(x4).expect("list append");
                list.append(x5).expect("list append");
                list.append(x6).expect("list append");
                dict.set_item("DamageState", list)
            }
            Attribute::CamSettings(x) => {
                let item = IntoPy::<PyObject>::into_py(x, py);
                dict.set_item("CamSettings", item)
            }
            Attribute::ClubColors(x) => {
                let item = IntoPy::<PyObject>::into_py(x, py);
                dict.set_item("ClubColors", item)
            }
            Attribute::Demolish(x) => {
                let item = IntoPy::<PyObject>::into_py(x, py);
                dict.set_item("Demolish", item)
            }
            Attribute::Enum(x) => {
                let item = IntoPy::<PyObject>::into_py(x, py);
                dict.set_item("Enum", item)
            }
            Attribute::Explosion(x) => {
                let item = IntoPy::<PyObject>::into_py(x, py);
                dict.set_item("Explosion", item)
            }
            Attribute::ExtendedExplosion(x1, x2, x3) => {
                let list = PyList::empty(py);
                let x1 = IntoPy::<PyObject>::into_py(x1, py);
                list.append(x1).expect("list append");
                list.append(x2).expect("list append");
                list.append(x3).expect("list append");
                dict.set_item("ExtendedExplosion", list)
            }
            Attribute::FlaggedByte(x1, x2) => {
                let list = PyList::empty(py);
                list.append(x1).expect("list append");
                list.append(x2).expect("list append");
                dict.set_item("FlaggedByte", list)
            }
            Attribute::Flagged(x1, x2) => {
                let list = PyList::empty(py);
                list.append(x1).expect("list append");
                list.append(x2).expect("list append");
                dict.set_item("Flagged", list)
            }
            Attribute::Float(x) => {
                let item = IntoPy::<PyObject>::into_py(x, py);
                dict.set_item("Float", item)
            }
            Attribute::GameMode(x1, x2) => {
                let list = PyList::empty(py);
                list.append(x1).expect("list append");
                list.append(x2).expect("list append");
                dict.set_item("GameMode", list)
            }
            Attribute::Int(x) => {
                let item = IntoPy::<PyObject>::into_py(x, py);
                dict.set_item("Int", item)
            }
            Attribute::Int64(x) => {
                let item = IntoPy::<PyObject>::into_py(x, py);
                dict.set_item("Int64", item)
            }
            Attribute::Loadout(x) => {
                let item = IntoPy::<PyObject>::into_py(x, py);
                dict.set_item("Loadout", item)
            }
            Attribute::TeamLoadout(x) => {
                let item = IntoPy::<PyObject>::into_py(x, py);
                dict.set_item("TeamLoadout", item)
            }
            Attribute::Location(x) => {
                let item = IntoPy::<PyObject>::into_py(x, py);
                dict.set_item("Location", item)
            }
            Attribute::MusicStinger(x) => {
                let item = IntoPy::<PyObject>::into_py(x, py);
                dict.set_item("MusicStinger", item)
            }
            Attribute::PlayerHistoryKey(x) => {
                let item = IntoPy::<PyObject>::into_py(x, py);
                dict.set_item("PlayerHistoryKey", item)
            }
            Attribute::Pickup(x) => {
                let item = IntoPy::<PyObject>::into_py(x, py);
                dict.set_item("Pickup", item)
            }
            Attribute::PickupNew(x) => {
                let item = IntoPy::<PyObject>::into_py(x, py);
                dict.set_item("PickupNew", item)
            }
            Attribute::QWord(x) => {
                let item = IntoPy::<PyObject>::into_py(x, py);
                dict.set_item("QWord", item)
            }
            Attribute::Welded(x) => {
                let item = IntoPy::<PyObject>::into_py(x, py);
                dict.set_item("Welded", item)
            }
            Attribute::Title(x1, x2, x3, x4, x5, x6, x7, x8) => {
                let list = PyList::empty(py);
                list.append(x1).expect("list append");
                list.append(x2).expect("list append");
                list.append(x3).expect("list append");
                list.append(x4).expect("list append");
                list.append(x5).expect("list append");
                list.append(x6).expect("list append");
                list.append(x7).expect("list append");
                list.append(x8).expect("list append");
                dict.set_item("Title", list)
            }
            Attribute::RigidBody(x) => {
                let item = IntoPy::<PyObject>::into_py(x, py);
                dict.set_item("RigidBody", item)
            }
            Attribute::TeamPaint(x) => {
                let item = IntoPy::<PyObject>::into_py(x, py);
                dict.set_item("TeamPaint", item)
            }
            Attribute::String(x) => {
                let item = IntoPy::<PyObject>::into_py(x, py);
                dict.set_item("String", item)
            }
            Attribute::UniqueId(x) => {
                let item = IntoPy::<PyObject>::into_py(x, py);
                dict.set_item("UniqueId", item)
            }
            Attribute::Reservation(x) => {
                let item = IntoPy::<PyObject>::into_py(x, py);
                dict.set_item("Reservation", item)
            }
            Attribute::PartyLeader(x) => {
                let item = IntoPy::<PyObject>::into_py(x, py);
                dict.set_item("PartyLeader", item)
            }
            Attribute::PrivateMatch(x) => {
                let item = IntoPy::<PyObject>::into_py(x, py);
                dict.set_item("PrivateMatch", item)
            }
            Attribute::LoadoutOnline(x) => {
                let item = IntoPy::<PyObject>::into_py(x, py);
                dict.set_item("LoadoutOnline", item)
            }
            Attribute::LoadoutsOnline(x) => {
                let item = IntoPy::<PyObject>::into_py(x, py);
                dict.set_item("LoadoutsOnline", item)
            }
            Attribute::Rotation(x) => {
                let item = IntoPy::<PyObject>::into_py(x, py);
                dict.set_item("Rotation", item)
            }
            Attribute::RepStatTitle(x) => {
                let item = IntoPy::<PyObject>::into_py(x, py);
                dict.set_item("RepStatTitle", item)
            }
            Attribute::StatEvent(x1, x2) => {
                let list = PyList::empty(py);
                list.append(x1).expect("list append");
                list.append(x2).expect("list append");
                dict.set_item("StatEvent", list)
            }
        };

        res.unwrap();
        dict.into()
    }
}

impl pyo3::IntoPy<PyObject> for HeaderProp {
    fn into_py(self, py: Python) -> PyObject {
        let dict = PyDict::new(py);
        let res = match self {
            HeaderProp::Array(x) => {
                let item = IntoPy::<PyObject>::into_py(x, py);
                dict.set_item("Array", item)
            }
            HeaderProp::Bool(x) => {
                let item = IntoPy::<PyObject>::into_py(x, py);
                dict.set_item("Bool", item)
            }
            HeaderProp::Byte => {
                let item = IntoPy::<PyObject>::into_py((), py);
                dict.set_item("Byte", item)
            }
            HeaderProp::Float(x) => {
                let item = IntoPy::<PyObject>::into_py(x, py);
                dict.set_item("Float", item)
            }
            HeaderProp::Int(x) => {
                let item = IntoPy::<PyObject>::into_py(x, py);
                dict.set_item("Int", item)
            }
            HeaderProp::Name(x) => {
                let item = IntoPy::<PyObject>::into_py(x, py);
                dict.set_item("Name", item)
            }
            HeaderProp::QWord(x) => {
                let item = IntoPy::<PyObject>::into_py(x, py);
                dict.set_item("QWord", item)
            }
            HeaderProp::Str(x) => {
                let item = IntoPy::<PyObject>::into_py(x, py);
                dict.set_item("Str", item)
            }
        };

        res.unwrap();
        dict.into()
    }
}

impl pyo3::IntoPy<PyObject> for ActorId {
    fn into_py(self, py: Python) -> PyObject {
        self.0.into_py(py)
    }
}

impl pyo3::IntoPy<PyObject> for StreamId {
    fn into_py(self, py: Python) -> PyObject {
        self.0.into_py(py)
    }
}

impl pyo3::IntoPy<PyObject> for ObjectId {
    fn into_py(self, py: Python) -> PyObject {
        self.0.into_py(py)
    }
}
