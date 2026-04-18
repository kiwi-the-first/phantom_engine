use std::fmt;

#[derive(Debug, Clone, PartialEq)]
pub enum Panels {
    Console,
    Viewport,
    Hierarchy,
    Inspector,
    AssetBrowser,
}
impl fmt::Display for Panels {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.title())
    }
}

impl Panels {
    pub fn title(&self) -> &'static str {
        match self {
            Panels::Console => "Console",
            Panels::Viewport => "Viewport",
            Panels::Hierarchy => "Hierarchy",
            Panels::Inspector => "Inspector",
            Panels::AssetBrowser => "Assets",
        }
    }
}
