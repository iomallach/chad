use std::{io::Cursor, usize};

use anyhow::Result;
use bytes::{Buf, Bytes};

use thiserror::Error;

#[derive(Error, Debug)]
pub enum ParseError {
    #[error("Incomplete frame")]
    IncompleteFrame,

    #[error("Unexpected value: {0}")]
    UnexpectedValue(String),
}

#[derive(Debug, PartialEq)]
pub enum Frame {
    Array(Vec<Frame>),
    Bulk(Bytes),
}

impl Frame {
    pub fn parse(cur: &mut Cursor<&[u8]>) -> Result<Frame> {
        match parse_u8(cur)? {
            b'*' => Ok(parse_array(cur)?),
            b'$' => Ok(parse_bulk_str(cur)?),
            b'^' => Ok(parse_bulk_str(cur)?),
            unexpected_byte => Err(ParseError::UnexpectedValue(unexpected_byte.to_string()).into()),
        }
    }

    pub fn array() -> Self {
        Self::Array(Vec::new())
    }

    pub fn push_bulk(&mut self, bulk: Frame) {
        if let Self::Array(a) = self {
            a.push(bulk);
        }
    }
}

fn parse_line<'a>(cur: &mut Cursor<&'a [u8]>) -> Result<&'a [u8]> {
    let start = cur.position() as usize;
    let end = cur.get_ref().len() - 1;

    for i in start..end {
        if cur.get_ref()[i] == b'\r' && cur.get_ref()[i + 1] == b'\n' {
            cur.set_position((i + 2) as u64);
            return Ok(&cur.get_ref()[start..i]);
        }
    }

    Err(ParseError::IncompleteFrame.into())
}

fn parse_u8(cur: &mut Cursor<&[u8]>) -> Result<u8> {
    if !cur.has_remaining() {
        return Err(ParseError::IncompleteFrame.into());
    }
    Ok(cur.get_u8())
}

fn parse_u64(cur: &mut Cursor<&[u8]>) -> Result<u64> {
    let line = parse_line(cur)?;
    let number_as_str = std::str::from_utf8(line)?;
    let number = number_as_str.to_string().parse::<u64>()?;

    Ok(number)
}

fn parse_bulk_str(cur: &mut Cursor<&[u8]>) -> Result<Frame> {
    let len = parse_u64(cur)? as usize;
    if cur.remaining() < len {
        return Err(ParseError::IncompleteFrame.into());
    }

    let bulk = Frame::Bulk(Bytes::copy_from_slice(&cur.chunk()[..len]));
    skip_bytes(cur, len + 2)?;

    Ok(bulk)
}

fn skip_bytes(cur: &mut Cursor<&[u8]>, n: usize) -> Result<()> {
    if cur.remaining() < n {
        return Err(ParseError::IncompleteFrame.into());
    }

    cur.advance(n);
    Ok(())
}

fn parse_array(cur: &mut Cursor<&[u8]>) -> Result<Frame> {
    let len = parse_u64(cur)?;
    let mut array = Frame::array();

    for _ in 0..len {
        let frame = Frame::parse(cur)?;
        array.push_bulk(frame);
    }

    Ok(array)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Cursor;

    #[test]
    fn test_parse_empty_array() {
        let mut cur = Cursor::new("*0\r\n".as_bytes());

        let res = Frame::parse(&mut cur).expect("Failed on empty array");
        assert_eq!(res, Frame::array())
    }

    #[test]
    fn test_skip_bytes_success() {
        let mut cur = Cursor::new("12345".as_bytes());
        skip_bytes(&mut cur, 2).expect("failed skipping bytes");

        assert_eq!(cur.remaining(), 3)
    }

    #[test]
    fn test_skip_bytes_fail() {
        let mut cur = Cursor::new("12345".as_bytes());
        let res = skip_bytes(&mut cur, 6);

        match res {
            Ok(_) => panic!("This should fail"),
            Err(e) => assert!(matches!(
                e.downcast_ref::<ParseError>(),
                Some(ParseError::IncompleteFrame)
            )),
        }
    }

    #[test]
    fn test_bulk_string() {
        let mut cur = Cursor::new("5\r\nhello\r\n".as_bytes());
        let res = parse_bulk_str(&mut cur).expect("Failed parsing bulk string");

        assert_eq!(res, Frame::Bulk(Bytes::from_static(b"hello")))
    }

    #[test]
    fn test_parse_u64() {
        let mut cur = Cursor::new("501\r\n".as_bytes());
        let res = parse_u64(&mut cur).expect("Failed parsing u64");

        assert_eq!(res, 501)
    }

    #[test]
    fn test_parse_u8() {
        let mut cur = Cursor::new("*\r\n".as_bytes());
        let res = parse_u8(&mut cur).expect("Failed parsing u8");

        assert_eq!(res, b'*')
    }

    #[test]
    fn test_parse_line() {
        let mut cur = Cursor::new("a sort of line\r\n".as_bytes());
        let res = parse_line(&mut cur).expect("Failed parsing a line");

        assert_eq!(res, b"a sort of line")
    }
}
