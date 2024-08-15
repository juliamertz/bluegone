use std::fmt::Display;

#[derive(Clone)]
pub struct Pid(pub u32);

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
