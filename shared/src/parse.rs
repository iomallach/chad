use std::{io::{BufReader, Read, BufRead}, str::FromStr};

use super::{MessageHeader, Message};

pub struct RequestParser<'a> {
    buf_reader: BufReader<&'a mut dyn Read>
}

impl<'a> RequestParser<'a> {
    pub fn new(stream: &'a mut dyn Read) -> Self {
        Self {
            buf_reader: BufReader::new(stream),
        }
    }

    pub fn parse_message(&mut self) -> Result<Message, std::io::Error> {
        let mut content_length: usize = 0;
        let mut body: Option<String> = None;
        let mut username: Option<String> = None;
        let mut timestamp: Option<String> = None;
        let mut connections: Option<usize> = None;
        loop {
            let mut line_buf = String::new();
            match self.buf_reader.read_line(&mut line_buf) {
                Ok(0) => return Err(std::io::Error::new(std::io::ErrorKind::BrokenPipe, "Broken pipe")),
                Ok(_) => {
                    if line_buf.trim().is_empty() && content_length > 0 {
                        let mut body_buf = vec![0u8; content_length];
                        self.buf_reader.read_exact(&mut body_buf).expect("Failed reading");
                        body = String::from_utf8(body_buf).ok();
                        break;
                    } else if line_buf.trim().is_empty() && content_length == 0 {
                        break;
                    } else {
                        let header = match MessageHeader::from_str(&line_buf) {
                            Ok(h) => h,
                            Err(_) => continue,
                        };
                        match header {
                            MessageHeader::Username(un) => username = Some(un),
                            MessageHeader::Timestamp(ts) => timestamp = Some(ts),
                            MessageHeader::Connections(c) => connections = Some(c),
                            MessageHeader::ContentLength(cl) => content_length = cl,
                        };
                    }
                },
                Err(e) => {
                    match e.kind() {
                        std::io::ErrorKind::WouldBlock => return Err(e),
                        std::io::ErrorKind::ConnectionReset => return Err(e),
                        std::io::ErrorKind::BrokenPipe => return Err(e),
                        _ => panic!("Something went wrong {}", e),
                    }
                }
            }
        }
        Ok(Message::new(
            &username.unwrap(),
            connections,
            timestamp.unwrap(),
            body.as_deref(),
        ))
    }
}

#[cfg(test)]
mod test {
    use chrono::TimeZone;

    use super::*;

    #[test]
    fn test_single_message() {
        let mut single_message = "Username:test\r\nTimestamp:01-01-2024 01:01:01\r\nConnections:3\r\nContent-Length:7\r\n\r\nmessage".as_bytes();
        let msg = RequestParser::new(&mut single_message).parse_message().unwrap();
        let dt: chrono::DateTime<chrono::Local> = chrono::prelude::Local.with_ymd_and_hms(2024, 1, 1, 1, 1, 1).unwrap();
        let expected_msg = Message::new("test", Some(3), Message::from_chrono(dt), Some("message"));
        assert_eq!(msg, expected_msg);
    }

    #[test]
    fn test_single_message_no_body() {
        let mut single_message = "Username:test\r\nTimestamp:01-01-2024 01:01:01\r\nConnections:3\r\nContent-Length:0\r\n\r\n".as_bytes();
        let msg = RequestParser::new(&mut single_message).parse_message().unwrap();
        let dt: chrono::DateTime<chrono::Local> = chrono::prelude::Local.with_ymd_and_hms(2024, 1, 1, 1, 1, 1).unwrap();
        let expected_msg = Message::new("test", Some(3), Message::from_chrono(dt), None);
        assert_eq!(msg, expected_msg);
    }

    #[test]
    fn test_messages() {
        let messages = format!(
            "{}{}{}",
            "Username:test\r\nTimestamp:01-01-2024 01:01:01\r\nConnections:3\r\nContent-Length:0\r\n\r\n",
            "Username:test2\r\nTimestamp:01-01-2024 01:01:01\r\nConnections:1\r\nContent-Length:2\r\n\r\nHi",
            "Username:test3\r\nTimestamp:01-01-2024 01:01:01\r\nConnections:0\r\nContent-Length:0\r\n\r\n"
        );
        let mut messages = messages.as_bytes();
        let mut parser = RequestParser::new(&mut messages);
        
        let dt: chrono::DateTime<chrono::Local> = chrono::prelude::Local.with_ymd_and_hms(2024, 1, 1, 1, 1, 1).unwrap();
        let msg_1 = parser.parse_message().unwrap();
        let msg_2 = parser.parse_message().unwrap();
        let msg_3 = parser.parse_message().unwrap();
        let expected_msg_1 = Message::new("test", Some(3), Message::from_chrono(dt), None);
        let expected_msg_2 = Message::new("test2", Some(1), Message::from_chrono(dt), Some("Hi"));
        let expected_msg_3 = Message::new("test3", Some(0), Message::from_chrono(dt), None);
        assert_eq!(msg_1, expected_msg_1);
        assert_eq!(msg_2, expected_msg_2);
        assert_eq!(msg_3, expected_msg_3);
    }
}