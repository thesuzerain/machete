use serde::{Deserialize, Serialize};

use super::ids::InternalId;

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub struct Character {
    pub id: InternalId,
    pub name: String,
    pub player: Option<String>,
    pub class: InternalId,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub enum Stat {
    Strength,
    Dexterity,
    Constitution,
    Intelligence,
    Wisdom,
    Charisma,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Skill {
    Acrobatics,
    Arcana,
    Athletics,
    Crafting,
    Deception,
    Diplomacy,
    Intimidation,
    Lore(Option<String>),
    Medicine,
    Nature,
    Occultism,
    Performance,
    Religion,
    Society,
    Stealth,
    Survival,
    Thievery,
    Unknown,
}

impl Stat {
    pub fn from_str(s: &str) -> Option<Self> {
        match s {
            "Strength" => Some(Self::Strength),
            "Dexterity" => Some(Self::Dexterity),
            "Constitution" => Some(Self::Constitution),
            "Intelligence" => Some(Self::Intelligence),
            "Wisdom" => Some(Self::Wisdom),
            "Charisma" => Some(Self::Charisma),
            _ => None,
        }
    }

    pub fn to_string(&self) -> String {
        match self {
            Self::Strength => "Strength",
            Self::Dexterity => "Dexterity",
            Self::Constitution => "Constitution",
            Self::Intelligence => "Intelligence",
            Self::Wisdom => "Wisdom",
            Self::Charisma => "Charisma",
        }
        .to_string()
    }

    pub fn iter() -> impl Iterator<Item = Self> {
        vec![
            Self::Strength,
            Self::Dexterity,
            Self::Constitution,
            Self::Intelligence,
            Self::Wisdom,
            Self::Charisma,
        ]
        .into_iter()
    }
}

impl Skill {
    pub fn to_string(&self) -> String {
        match self {
            Self::Acrobatics => "Acrobatics".to_string(),
            Self::Arcana => "Arcana".to_string(),
            Self::Athletics => "Athletics".to_string(),
            Self::Crafting => "Crafting".to_string(),
            Self::Deception => "Deception".to_string(),
            Self::Diplomacy => "Diplomacy".to_string(),
            Self::Intimidation => "Intimidation".to_string(),
            Self::Lore(None) => "Lore".to_string(),
            Self::Lore(Some(s)) => format!("Lore ({})", s),
            Self::Medicine => "Medicine".to_string(),
            Self::Nature => "Nature".to_string(),
            Self::Occultism => "Occultism".to_string(),
            Self::Performance => "Performance".to_string(),
            Self::Religion => "Religion".to_string(),
            Self::Society => "Society".to_string(),
            Self::Stealth => "Stealth".to_string(),
            Self::Survival => "Survival".to_string(),
            Self::Thievery => "Thievery".to_string(),
            Self::Unknown => "Unknown".to_string(),
        }
    }

    pub fn from_str(s: &str) -> Option<Self> {
        match s {
            "Acrobatics" => Some(Self::Acrobatics),
            "Arcana" => Some(Self::Arcana),
            "Athletics" => Some(Self::Athletics),
            "Crafting" => Some(Self::Crafting),
            "Deception" => Some(Self::Deception),
            "Diplomacy" => Some(Self::Diplomacy),
            "Intimidation" => Some(Self::Intimidation),
            s if s.starts_with("Lore (") && s.ends_with(")") => {
                Some(Self::Lore(Some(s[6..s.len() - 1].to_string())))
            }
            "Lore" => Some(Self::Lore(None)),
            "Medicine" => Some(Self::Medicine),
            "Nature" => Some(Self::Nature),
            "Occultism" => Some(Self::Occultism),
            "Performance" => Some(Self::Performance),
            "Religion" => Some(Self::Religion),
            "Society" => Some(Self::Society),
            "Stealth" => Some(Self::Stealth),
            "Survival" => Some(Self::Survival),
            "Thievery" => Some(Self::Thievery),
            "Unknown" => Some(Self::Unknown),
            _ => None,
        }
    }

    pub fn iter() -> impl Iterator<Item = Self> {
        vec![
            Self::Acrobatics,
            Self::Arcana,
            Self::Athletics,
            Self::Crafting,
            Self::Deception,
            Self::Diplomacy,
            Self::Intimidation,
            Self::Lore(None),
            Self::Medicine,
            Self::Nature,
            Self::Occultism,
            Self::Performance,
            Self::Religion,
            Self::Society,
            Self::Stealth,
            Self::Survival,
            Self::Thievery,
        ]
        .into_iter()
    }
}

/*
    Lore should parse three different ways.
    1. A string with just "Lore" => Lore(None)
    2. A string with "Lore (Some string)" => Lore(Some(Some string))
    3. A 'directly' serialized structure - {"Lore":"Wizards"} or {"Lore":null} (Though we won't use this)
*/
pub mod skill_serialize {
    use serde::Deserialize;

    use crate::models::characters::Skill;

    pub fn deserialize<'de, D>(deserializer: D) -> Result<Skill, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        #[derive(Deserialize)]
        #[serde(untagged)]
        pub enum SkillWrapper {
            Skill(String),
            Lore {
                #[serde(rename = "Lore")]
                lore: Option<String>,
            },
        }
        if let Ok(lore_wrapper) = SkillWrapper::deserialize(deserializer) {
            match lore_wrapper {
                SkillWrapper::Skill(s) => {
                    if let Some(skill) = Skill::from_str(&s) {
                        Ok(skill)
                    } else {
                        Err(serde::de::Error::custom(format!("Invalid skill: {}", s)))
                    }
                }
                SkillWrapper::Lore { lore } => Ok(Skill::Lore(lore)),
            }
        } else {
            Err(serde::de::Error::custom("Invalid skill"))
        }
    }

    pub fn serialize<S>(skill: &Skill, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        match skill {
            Skill::Lore(Some(lore)) => serializer.serialize_str(&format!("Lore ({})", lore)),
            Skill::Lore(None) => serializer.serialize_str("Lore"),
            _ => serializer.serialize_str(&skill.to_string()),
        }
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn test_lore_parsing() {
        use super::Skill;
        use crate::models::characters::skill_serialize;

        #[derive(serde::Deserialize)]
        pub struct TestStruct {
            #[serde(with = "skill_serialize")]
            pub skill: Skill,
        }

        let skill = r#"{"skill": "Athletics"}"#;
        let skill = serde_json::from_str::<TestStruct>(skill).unwrap();
        assert_eq!(skill.skill, Skill::Athletics);

        let lore_str = r#"{ "skill": "Lore (Wizards)" }"#;
        let lore = serde_json::from_str::<TestStruct>(lore_str).unwrap();
        assert_eq!(lore.skill, Skill::Lore(Some("Wizards".to_string())));

        let lore_str = r#"{ "skill": "Lore" }"#;
        let lore = serde_json::from_str::<TestStruct>(lore_str).unwrap();
        assert_eq!(lore.skill, Skill::Lore(None));

        let lore_str = r#"{ "skill": { "Lore": null } }"#;
        let lore = serde_json::from_str::<TestStruct>(lore_str).unwrap();
        assert_eq!(lore.skill, Skill::Lore(None));

        let lore_str = r#"{ "skill": { "Lore": "Wizards" } }"#;
        let lore = serde_json::from_str::<TestStruct>(lore_str).unwrap();
        assert_eq!(lore.skill, Skill::Lore(Some("Wizards".to_string())));
    }
}
