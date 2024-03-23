#[derive(Debug, Copy, Clone, PartialEq)]
pub enum Command<'a> {
    DbSize,
    Get(&'a str),
    Set(&'a str, &'a str),
    Quit,
}

impl Command<'_> {
    pub fn all_commands() -> Vec<&'static str> {
        vec!["DBSIZE", "GET", "SET", "QUIT"]
    }
}
