use std::fmt::Display;

pub trait StateFileName {
    fn name() -> String;
}

#[derive(Clone)]
pub struct Pid(pub u32);

impl Pid {
    pub fn as_u32(&self) -> u32 {
        self.0
    }
}

impl From<u32> for Pid {
    fn from(value: u32) -> Self {
        Self(value)
    }
}

impl Display for Pid {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self.0)
    }
}

impl TryFrom<String> for Pid {
    type Error = anyhow::Error;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        Ok(Pid(value.parse()?))
    }
}

impl StateFileName for Pid {
    fn name() -> String {
        "pid".into()
    }
}
