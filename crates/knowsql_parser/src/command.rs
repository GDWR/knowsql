#[derive(Copy, Clone, Debug, PartialEq)]
pub enum Command<'a> {
    DbSize,
    Command(SubCommand),
    Get(&'a str),
    Set(&'a str, &'a str),
    Ping,
    Quit,
}

#[derive(Copy, Clone, Debug, PartialEq)]
pub enum SubCommand {
    Docs,
}

impl Command<'_> {
    pub fn all_commands() -> Vec<(&'static str, &'static [&'static str])> {
        vec![
            ("DBSIZE", &["Return the number of keys in the database."]),
            (
                "COMMAND DOCS",
                &["Return documentary information about commands."],
            ),
            ("GET", &["Get the value of key."]),
            ("SET", &["Set the value of key."]),
            ("PING", &["Pong."]),
            ("QUIT", &["Ask the server to close the connection."]),
        ]
    }
}
