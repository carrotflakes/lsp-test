#[derive(Debug)]
pub struct Buffer {
    lines: Vec<String>,
}

impl Buffer {
    pub fn new_empty() -> Self {
        Self {
            lines: vec![String::new()],
        }
    }

    pub fn delete_range(&mut self, start: (usize, usize), end: (usize, usize)) {
        if start.0 == end.0 {
            self.lines[start.0].drain(start.1..end.1).count();
        } else {
            self.lines[start.0].drain(start.1..).count();
            let end_line = self.lines.remove(end.0);
            self.lines[start.0] += &end_line[end.1..];
            if start.0 + 1 < end.0 {
                self.lines.drain(start.0 + 1..end.0).count();
            }
        }
    }

    pub fn insert(&mut self, start: (usize, usize), text: &str) {
        let lines: Vec<_> = Lines(Some(text)).collect();
        if lines.len() == 0 {
            return;
        } else if lines.len() == 1 {
            self.lines[start.0].replace_range(start.1..start.1, text);
        } else {
            let mut last_line = lines.last().unwrap().to_string();
            last_line.push_str(&self.lines[start.0][start.1..]);
            self.lines[start.0].truncate(start.1);
            self.lines[start.0].push_str(&lines[0]);
            self.lines.splice(
                start.0 + 1..start.0 + 1,
                lines[1..lines.len() - 1]
                    .iter()
                    .map(|x| x.to_string())
                    .chain([last_line]),
            );
        }
    }

    pub fn update(&mut self, start: (usize, usize), end: (usize, usize), text: &str) {
        self.delete_range(start, end);
        self.insert(start, text);
    }

    pub fn chars<'a>(&'a self) -> Chars<'a> {
        Chars {
            buffer: &self,
            line: 0,
            chars: self.lines[0].chars(),
        }
    }
}

impl From<&str> for Buffer {
    fn from(str: &str) -> Self {
        Buffer {
            lines: Lines(Some(str)).map(String::from).collect(),
        }
    }
}

pub struct Chars<'a> {
    buffer: &'a Buffer,
    line: usize,
    chars: std::str::Chars<'a>,
}

impl<'a> Iterator for Chars<'a> {
    type Item = char;

    fn next(&mut self) -> Option<Self::Item> {
        if let Some(char) = self.chars.next() {
            Some(char)
        } else {
            self.line += 1;
            if self.buffer.lines.len() == self.line {
                return None;
            }
            self.chars = self.buffer.lines[self.line].chars();
            self.next()
        }
    }
}

struct Lines<'a>(Option<&'a str>);

impl<'a> Iterator for Lines<'a> {
    type Item = &'a str;

    fn next(&mut self) -> Option<Self::Item> {
        if let Some(str) = self.0 {
            match (str.find('\r'), str.find('\n')) {
                (None, None) => {
                    let r = str;
                    self.0 = None;
                    Some(r)
                }
                (Some(i), Some(j)) if i + 1 == j => {
                    let r = &str[..i + 2];
                    self.0 = Some(&str[i + 2..]);
                    Some(r)
                }
                (Some(i), Some(j)) if i < j => {
                    let r = &str[..i + 1];
                    self.0 = Some(&str[i + 1..]);
                    Some(r)
                }
                (Some(_), Some(j)) => {
                    let r = &str[..j + 1];
                    self.0 = Some(&str[j + 1..]);
                    Some(r)
                }
                (None, Some(i)) | (Some(i), None) => {
                    let r = &str[..i + 1];
                    self.0 = Some(&str[i + 1..]);
                    Some(r)
                }
            }
        } else {
            None
        }
    }
}

#[test]
fn test() {
    let mut buf = Buffer::new_empty();
    buf.update((0, 0), (0, 0), "hello");
    assert_eq!(&buf.lines, &vec!["hello"]);
    buf.update((0, 0), (0, 0), ":)");
    assert_eq!(&buf.lines, &vec![":)hello"]);
    buf.update((0, 7), (0, 7), " world!");
    assert_eq!(&buf.lines, &vec![":)hello world!"]);
    buf.update((0, 7), (0, 8), "");
    assert_eq!(&buf.lines, &vec![":)helloworld!"]);
    buf.update((0, 7), (0, 7), "\n");
    assert_eq!(&buf.lines, &vec![":)hello\n", "world!"]);
    buf.update((0, 0), (0, 2), "a\nb\r\nc\r\r");
    assert_eq!(
        &buf.lines,
        &vec!["a\n", "b\r\n", "c\r", "\r", "hello\n", "world!"]
    );
    buf.update((0, 0), (4, 0), "");
    assert_eq!(&buf.lines, &vec!["hello\n", "world!"]);

    dbg!(buf.chars().collect::<String>());
}
