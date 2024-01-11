use std::{str::Chars, iter::Peekable};

#[derive(Debug)]
pub enum ParseResult {
    NotACommand,
    InvalidCommand,
    UnexpectedToken,
    NoArgument,
    Command(Command),
    EmptyInput,
}

#[derive(Debug)]
pub enum Command {
    Login(String),
    Connect,
    Disconnect,
}

pub struct CommandParser<'a> {
    source: Peekable<Chars<'a>>,
}

impl<'a> CommandParser<'a> {
    pub fn new(source: &'a str) -> Self {
        Self {
            source: source.chars().peekable(),
        }
    }

    pub fn next_command(&mut self) -> ParseResult {
        if self.source.clone().collect::<Vec<char>>().is_empty() {
            return ParseResult::EmptyInput
        }
        if self.is_command() {
            let command = match self.parse_command() {
                Ok(c) => c,
                Err(e) => return e,
            };
            let arg = self.parse_argument();
            self.map_command(command, arg)
        } else {
            ParseResult::NotACommand
        }
    }

    fn is_command(&mut self) -> bool {
        if let Some('/') = self.source.peek() {
            let _ = self.source.next();
            true
        } else {
            false
        }
    }

    fn parse_command(&mut self) -> Result<String, ParseResult> {
        let mut command_buffer = String::new();

        while let Some(next) = self.source.next() {
            match next {
                c if c.is_ascii_alphabetic() => command_buffer.push(c),
                c if c.is_ascii_whitespace() => break,
                _ => return Err(ParseResult::UnexpectedToken),
            }
        }
        Ok(command_buffer)
    }

    fn parse_argument(&mut self) -> Option<String> {
        let mut command_arg_buffer = String::new();
        if self.source.peek().is_none() {
            return None
        }
        while let Some(next) = self.source.next() {
            match next {
                c if c.is_alphanumeric() => command_arg_buffer.push(c),
                _ => break,
            }
        }
        Some(command_arg_buffer)
    }

    fn map_command(&self, command: String, arg: Option<String>) -> ParseResult {
        match command.as_str() {
            "login" => {
                match arg {
                    Some(s) => ParseResult::Command(Command::Login(s)),
                    None => ParseResult::NoArgument
                }
            }
            "connect" => ParseResult::Command(Command::Connect),
            "disconnect" => ParseResult::Command(Command::Disconnect),
            _ => ParseResult::InvalidCommand,
        }
    }
}

#[cfg(test)]
mod tests {
    // use super::*;

    // #[test]
    // fn test_parses_connect() {
    //     let source = "/connect";
    //     let mut parser = CommandParser::new(source);
    //     let command = parser.next_command();
    //     match command {
    //         Ok(cmd) => {
    //             match cmd {
    //                 Command::Connect => {},
    //                 _ => panic!("Wrong command")
    //             }
    //         },
    //         Err(e) => {
    //             panic!("Unexpected error {:?}", e)
    //         }
    //     }
    // }

    // #[test]
    // fn test_parses_correct_login() {
    //     let source = "/login vasya";
    //     let mut parser = CommandParser::new(source);
    //     match parser.next_command() {
    //         Ok(cmd) => {
    //             match cmd {
    //                 Command::Login(l) => {
    //                     if l == "vasya" {} else {
    //                         panic!("Expected vasya, but got {}", l)
    //                     }
    //                 },
    //                 _ => panic!("Wrong command")
    //             }
    //         },
    //         Err(e) => {
    //             panic!("Unexpected error {:?}", e)
    //         }
    //     }
    // }

    // #[test]
    // fn test_fails_on_no_login() {
    //     let source = "/login";
    //     let mut parser = CommandParser::new(source);
    //     match parser.next_command() {
    //         Ok(_) => panic!("This is not OK"),
    //         Err(e) => {
    //             match e {
    //                 ParseError::NoArgument => {},
    //                 _ => panic!("Wrong error"),
    //             }
    //         }
    //     }
    // }

    // #[test]
    // fn test_fails_on_wrong_command() {
    //     let source = "/hey";
    //     let mut parser = CommandParser::new(source);
    //     match parser.next_command() {
    //         Ok(_) => panic!("This is not OK"),
    //         Err(e) => {
    //             match e {
    //                 ParseError::InvalidCommand => {},
    //                 _ => panic!("Wrong error, expected an InvalidCommand")
    //             }
    //         }
    //     }
    // }

    // #[test]
    // fn test_fails_not_a_command() {
    //     let source = "not a command";
    //     let mut parser = CommandParser::new(source);
    //     match parser.next_command() {
    //         Ok(_) => panic!("This is not OK"),
    //         Err(e) => {
    //             match e {
    //                 ParseError::NotACommand => {},
    //                 _ => panic!("Wrong error, expected a NotACommand error")
    //             }
    //         }
    //     }
    // }

    // #[test]
    // fn test_fails_on_unexpected_token() {
    //     let source = "/comm4and";
    //     let mut parser = CommandParser::new(source);
    //     match parser.next_command() {
    //         Ok(_) => panic!("This is not OK"),
    //         Err(e) => {
    //             match e {
    //                 ParseError::UnexpectedToken => {},
    //                 _ => panic!("Wrong error, expected an UnexpectedToken error")
    //             }
    //         }
    //     }
    // }
}