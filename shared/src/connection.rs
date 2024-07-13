use crate::parse_async::Frame;
use crate::parse_async::ParseError;
use anyhow::bail;
use anyhow::Result;
use bytes::Buf;
use bytes::BytesMut;
use std::io::Cursor;
use tokio::io::AsyncWriteExt;
use tokio::io::{AsyncRead, AsyncReadExt, AsyncWrite, BufReader, BufWriter};

pub struct Connection<W, R>
where
    W: AsyncWrite + Unpin,
    R: AsyncRead + Unpin,
{
    reader: BufReader<R>,
    // TODO: this is pub for the time being
    // instead there should be a public write_frame function
    pub writer: BufWriter<W>,
    buffer: BytesMut,
}

impl<W, R> Connection<W, R>
where
    W: AsyncWrite + Unpin,
    R: AsyncRead + Unpin,
{
    pub fn new(reader: R, writer: W) -> Self {
        Self {
            reader: BufReader::new(reader),
            writer: BufWriter::new(writer),
            buffer: BytesMut::with_capacity(1024 * 512),
        }
    }

    pub async fn read_frame(&mut self) -> Result<Frame> {
        loop {
            let mut cursor = Cursor::new(&self.buffer[..]);
            match Frame::parse(&mut cursor) {
                Ok(frame) => {
                    self.buffer.advance(cursor.position() as usize);
                    return Ok(frame);
                }
                Err(e) => match e.downcast_ref::<ParseError>() {
                    Some(ParseError::IncompleteFrame) => {
                        if 0 == self.reader.read_buf(&mut self.buffer).await? {
                            anyhow::bail!("connection reset by peer")
                        }
                    }
                    _ => return Err(e),
                },
            }
        }
    }

    pub async fn write_frame(&mut self, frame: Frame) -> Result<()> {
        write_frame_into(&mut self.writer, frame).await
    }

    // TODO: figure out how to write a recursive structure as async doesn't support recursion
    pub async fn write_who_is_in_chat(&mut self, frame: Frame) -> Result<()> {
        match frame {
            Frame::Array(arr) => {
                self.writer
                    .write_all(format!("*{}\r\n", arr.len()).as_bytes())
                    .await?;
                let mut frame_iter = arr.into_iter();
                if let Some(Frame::Bulk(b)) = frame_iter.next() {
                    self.writer
                        .write_all(format!("${}\r\n", b.len()).as_bytes())
                        .await?;
                    self.writer.write_all(&b).await?;
                    self.writer.write_all(b"\r\n").await?;
                }
                write_frame_into(
                    &mut self.writer,
                    frame_iter.next().expect("Broken who is in chat frame"),
                )
                .await?;
                Ok(())
            }
            Frame::Bulk(_) => bail!("Expected array frame, got bulk"),
        }
    }
}

pub async fn write_frame_into<W: AsyncWrite + Unpin>(dst: &mut W, frame: Frame) -> Result<()> {
    match frame {
        Frame::Array(arr) => {
            dst.write_all(format!("*{}\r\n", arr.len()).as_bytes())
                .await?;
            let mut frame_iter = arr.into_iter();
            while let Some(Frame::Bulk(b)) = frame_iter.next() {
                dst.write_all(format!("${}\r\n", b.len()).as_bytes())
                    .await?;
                dst.write_all(&b).await?;
                dst.write_all(b"\r\n").await?;
            }
            dst.flush().await?;
            Ok(())
        }
        Frame::Bulk(_) => bail!("Expected array frame, got bulk"),
    }
}
