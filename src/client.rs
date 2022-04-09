use serde::{Serialize, Serializer};

fn round_serialize<S>(x: &f64, s: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    s.serialize_f64((x * 10000.).round() / 10000.)
}

#[derive(Debug, Serialize, Clone)]
pub struct Client {
    #[serde(rename = "client")]
    pub id: u16,
    #[serde(serialize_with = "round_serialize")]
    pub available: f64,
    #[serde(serialize_with = "round_serialize")]
    pub held: f64,
    #[serde(serialize_with = "round_serialize")]
    pub total: f64,
    pub locked: bool,
}

impl Client {
    pub fn new(id: u16) -> Self {
        Self {
            id,
            available: 0.,
            held: 0.,
            total: 0.,
            locked: false,
        }
    }
}
