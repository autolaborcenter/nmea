use super::NmeaLine;
use std::str::FromStr;

pub struct NmeaParser<const LEN: usize> {
    buf: [u8; LEN],
    cursor_r: Cursors,
    cursor_w: usize,
}

impl<const LEN: usize> Default for NmeaParser<LEN> {
    fn default() -> Self {
        Self {
            buf: [0u8; LEN],
            cursor_r: Cursors { r: 0, c: 0 },
            cursor_w: 0,
        }
    }
}

macro_rules! move_cursor {
    ($cursor:expr, $ch:expr, $buf:ident) => {
        loop {
            if $cursor < $buf.len() {
                if $buf[$cursor] == $ch {
                    break;
                }
                $cursor += 1;
            } else {
                return false;
            }
        }
    };
}

impl<const LEN: usize> NmeaParser<LEN> {
    #[inline]
    pub fn as_buf<'a>(&'a mut self) -> &'a mut [u8] {
        &mut self.buf[self.cursor_w..]
    }

    #[inline]
    pub fn notify_received<'a>(&'a mut self, n: usize) {
        self.cursor_w += n;
    }
}

impl<const LEN: usize> Iterator for NmeaParser<LEN> {
    type Item = (NmeaLine, u8);

    /// 从缓冲区解析一个 NEMA 消息，当且仅当缓冲区中没有完整的消息时返回 [`None`]，此时需要读取新的数据填充到缓冲区
    fn next(&mut self) -> Option<Self::Item> {
        let result = loop {
            // 无法继续解析或全部解析完成
            if !self.cursor_r.move_on(&self.buf[..self.cursor_w])
                || self.cursor_w < self.cursor_r.c + 3
            {
                // 缓冲区全满，从头丢弃 1 字节
                if self.cursor_r.r == 0 && self.cursor_w == LEN {
                    self.cursor_r.r = 1;
                    continue;
                }
                // 放弃继续解析，准备接收
                else {
                    break None;
                }
            }
            if self.cursor_r.len() >= 3 && self.buf[self.cursor_r.r + 1..].starts_with(b"cmd") {
                // $cmd...*ff
                if self.buf[self.cursor_r.c + 1..].starts_with(b"ff") {
                    break Some(self.cursor_r.complete(&self.buf[..self.cursor_w]));
                }
            } else if self.cursor_r.xor_check(&self.buf[..self.cursor_w]) {
                break Some(self.cursor_r.complete(&self.buf[..self.cursor_w]));
            }
            self.cursor_r.move_next();
        };
        // 尽量挪动内存以尽量多从外设读取
        if (1..self.cursor_w).contains(&self.cursor_r.r) {
            self.buf.copy_within(self.cursor_r.r..self.cursor_w, 0);
        }
        self.cursor_w -= self.cursor_r.r;
        self.cursor_r.reset();
        return result;
    }
}

struct Cursors {
    r: usize, // 光标：读取，移动缓冲字节时要保留的第一个字节
    c: usize, // 光标：检查，扫描的起点
}

impl Cursors {
    /// r 移动到 '$'
    /// c 移动到 '*'
    fn move_on(&mut self, buf: &[u8]) -> bool {
        move_cursor!(self.r, b'$', buf);
        if self.r >= self.c {
            self.c = self.r + 1;
        }
        move_cursor!(self.c, b'*', buf);
        true
    }

    /// 从 buf 切出字符串
    fn complete(&mut self, buf: &[u8]) -> (NmeaLine, u8) {
        let cs = parse_cs(&buf[self.c..]);
        let result = unsafe { std::str::from_utf8_unchecked(&buf[self.r + 1..self.c]) };
        self.move_next();
        self.move_on(buf);
        (NmeaLine::from_str(result).unwrap(), cs)
    }

    /// 一次解析完成
    #[inline]
    fn move_next(&mut self) {
        self.r = self.c + 3;
        self.c += 4;
    }

    /// 回车
    #[inline]
    fn reset(&mut self) {
        if self.c > self.r {
            self.c -= self.r;
        } else {
            self.c = 0;
        }
        self.r = 0;
    }

    /// 异或校验
    #[inline]
    fn xor_check<'a>(&self, buf: &'a [u8]) -> bool {
        buf[self.r + 1..self.c].iter().fold(0, |sum, it| sum ^ *it) == parse_cs(&buf[self.c..])
    }

    /// 已检查的区段长度
    #[inline]
    fn len(&self) -> usize {
        self.c - self.r - 1
    }
}

#[inline]
fn parse_u8(byte: u8) -> Option<u8> {
    match byte {
        b'0'..=b'9' => Some(byte - b'0'),
        b'a'..=b'f' => Some(byte - b'a' + 10),
        b'A'..=b'F' => Some(byte - b'A' + 10),
        _ => None,
    }
}

#[inline]
fn parse_cs(cs: &[u8]) -> u8 {
    parse_u8(cs[1]).unwrap() << 4 | parse_u8(cs[2]).unwrap()
}
