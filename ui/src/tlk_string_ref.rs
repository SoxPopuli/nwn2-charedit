use crate::error::Error;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TlkStringRef {
    pub id: u32,
    pub data: String,
}
impl TlkStringRef {
    pub fn from_id(tlk: &crate::Tlk, id: u32) -> Result<Self, Error> {
        match tlk.get_from_str_ref(id) {
            Ok(Some(s)) => Ok(Self {
                id,
                data: s.to_string(),
            }),
            Ok(None) => Ok(Self {
                id,
                data: "".into(),
            }),
            Err(e) => Err(Error::LibError(e)),
        }
    }
}
