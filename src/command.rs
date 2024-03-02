#[derive(Debug)]
pub enum Command<'a> {
    Array(&'a [Command<'a>]),
    String(&'a str),
}
