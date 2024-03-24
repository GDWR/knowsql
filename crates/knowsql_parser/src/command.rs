#[derive(Copy, Clone, Debug, PartialEq)]
pub enum Command<'a> {
    DbSize,
    Command(SubCommand),
    Echo(&'a str),
    Get(&'a str),
    Keys(&'a str),
    Set(&'a str, &'a str),
    Ping,
    Quit,
}

#[derive(Copy, Clone, Debug, PartialEq)]
pub enum SubCommand {
    Docs,
}

impl Command<'_> {
    pub fn all_commands() -> &'static [(&'static str, &'static [&'static str])] {
        &[
            ("DBSIZE", &["Return the number of keys in the database."]),
            (
                "COMMAND DOCS",
                &["Return documentary information about commands."],
            ),
            ("ECHO", &["Returns message."]),
            ("GET", &["Get the value of key."]),
            ("KEYS", &["Get all keys matching pattern."]),
            ("SET", &["Set the value of key."]),
            ("PING", &["Pong."]),
            ("QUIT", &["Ask the server to close the connection."]),
        ]
    }
}
