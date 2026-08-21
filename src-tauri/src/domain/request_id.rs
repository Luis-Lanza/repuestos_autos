use uuid::Uuid;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RequestId(Uuid);

impl RequestId {
    pub fn parse(value: &str) -> Result<Self, uuid::Error> {
        Uuid::parse_str(value).map(Self)
    }

    pub fn as_uuid(&self) -> &Uuid {
        &self.0
    }
}
