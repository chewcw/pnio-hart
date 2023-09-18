use crate::config::Config;
use serde::{self, Deserialize, Serialize};

#[derive(Deserialize, Serialize, Debug)]
pub struct ModuleTwin {
    pub desired: Desired,
    pub reported: Reported,
}

impl ModuleTwin {
    pub fn serialize(&self) -> anyhow::Result<String> {
        let str = serde_json::to_string(self)?;
        Ok(str)
    }

    pub fn deserialize(content: &str) -> anyhow::Result<Self> {
        let module_twin = serde_json::from_str::<Self>(content)?;
        Ok(module_twin)
    }
}

impl Default for ModuleTwin {
    fn default() -> Self {
        Self {
            desired: Desired {
                config: Default::default(),
                version: 0,
            },
            reported: Reported {
                config: Default::default(),
                version: 0,
            },
        }
    }
}

#[derive(Deserialize, Serialize, Debug)]
pub struct Desired {
    pub config: Vec<Config>,
    #[serde(rename(serialize = "$version", deserialize = "$version"))]
    pub version: u16,
}

impl Desired {
    pub fn deserialize(content: &str) -> anyhow::Result<Self> {
        let desired = serde_json::from_str::<Self>(content)?;
        Ok(desired)
    }
}

#[derive(Deserialize, Serialize, Debug)]
pub struct Reported {
    // #[serde(skip_deserializing)]
    pub config: Option<Vec<Config>>, // at first there won't be any config
    #[serde(rename(serialize = "$version", deserialize = "$version"))]
    #[serde(skip_serializing)]
    pub version: u16,
}

impl Reported {
    pub fn serialize(&self) -> anyhow::Result<String> {
        let str = serde_json::to_string(self)?;
        Ok(str)
    }
}
