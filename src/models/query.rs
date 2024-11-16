use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Clone)]
pub struct CommaSeparatedVec(pub Vec<u32>);

impl Default for CommaSeparatedVec {
    fn default() -> Self {
        CommaSeparatedVec(Vec::new())
    }
}

impl CommaSeparatedVec {
    pub fn into_inner(self) -> Vec<u32> {
        self.0
    }
}
impl<'de> Deserialize<'de> for CommaSeparatedVec {
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
        where
            D: serde::Deserializer<'de>
    {
        let s = String::deserialize(deserializer)?;
        let vec : Result<Vec<u32>, _> = s.split(',').map(|s| s.parse()).collect();
        Ok(CommaSeparatedVec(vec.map_err(serde::de::Error::custom)?))
    }
}
