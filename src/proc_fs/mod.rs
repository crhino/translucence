pub mod stats;
pub mod kernel;

pub trait ToPid {
    fn to_pid(&self) -> String;
}

impl ToPid for usize {
    fn to_pid(&self) -> String {
        format!("{}", self)
    }
}

impl ToPid for u32 {
    fn to_pid(&self) -> String {
        format!("{}", self)
    }
}

impl ToPid for String {
    fn to_pid(&self) -> String {
        self.clone()
    }
}

impl<'a> ToPid for &'a str {
    fn to_pid(&self) -> String {
        self.to_string()
    }
}
