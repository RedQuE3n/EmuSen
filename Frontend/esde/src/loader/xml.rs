//! A theme file read strictly, else as ES-DE reads it: a bare `&` kept as text, text outside the root ignored
//! (EmuSen_BigPicture.md §31.2). The tree keeps what the loader asks of an element: its name, attributes, line, children
//! and its value, the text of all its descendants with whitespace-only text dropped.

use quick_xml::events::{BytesStart, Event};
use quick_xml::Reader;

/// One element: local name, attributes in file order, the line of its `<`, child elements and its descendant text.
#[derive(Clone, Debug, Default)]
pub struct Node {
    pub name: String,
    pub attributes: Vec<(String, String)>,
    pub line: u32,
    pub children: Vec<Node>,
    pub value: String,
}

impl Node {
    pub fn attribute(&self, name: &str) -> Option<&str> {
        self.attributes.iter().find(|(n, _)| n == name).map(|(_, v)| v.as_str())
    }

    pub fn child(&self, name: &str) -> Option<&Node> {
        self.children.iter().find(|c| c.name == name)
    }

    pub fn children_named<'a>(&'a self, name: &'a str) -> impl Iterator<Item = &'a Node> + 'a {
        self.children.iter().filter(move |c| c.name == name)
    }
}

/// XML that ES-DE refuses too, such as mismatched tags: the reason and the line.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct XmlError {
    pub message: String,
    pub line: u32,
}

/// Reads a file's text; the root and, when the strict reading failed, what was read leniently.
pub fn parse(text: &str) -> Result<(Node, Option<String>), XmlError> {
    let text = normalise(text);
    let strict = match read(&text, true) {
        Ok(Read { root: Some(root), .. }) => return Ok((root, None)),
        Ok(_) => XmlError { message: "the file holds no root element".into(), line: 1 },
        Err(e) => e,
    };
    let (escaped, bare) = protect(&text);
    let read = read(&escaped, false).map_err(|_| strict.clone())?;
    let root = read.root.ok_or_else(|| strict.clone())?;
    let outside = read.outside;
    let mut leniency = String::new();
    if bare > 0 {
        leniency.push_str(&format!("{bare} bare & kept as text"));
    }
    if bare > 0 && outside {
        leniency.push_str("; ");
    }
    if outside {
        leniency.push_str("text outside the root element ignored");
    }
    if leniency.is_empty() {
        leniency = strict.message;
    }
    Ok((root, Some(leniency)))
}

// A byte-order mark dropped and line ends made \n, as an XML parser does before it reads.
fn normalise(text: &str) -> String {
    text.strip_prefix('\u{feff}').unwrap_or(text).replace("\r\n", "\n").replace('\r', "\n")
}

// A bare & escaped, leaving CDATA, comments and the five entities and character references alone; XML declarations blanked.
fn protect(text: &str) -> (String, usize) {
    let mut out = String::with_capacity(text.len() + 16);
    let mut bare = 0;
    let mut i = 0;
    while i < text.len() {
        let rest = &text[i..];
        let skip = [("<![CDATA[", "]]>"), ("<!--", "-->")]
            .iter()
            .find_map(|(open, close)| rest.strip_prefix(open).and_then(|body| body.find(close)).map(|e| open.len() + e + close.len()));
        if let Some(n) = skip {
            out.push_str(&rest[..n]);
            i += n;
            continue;
        }
        if rest.starts_with("<?xml") {
            if let Some(end) = rest[2..].find('?').map(|q| q + 2).filter(|&q| rest[q..].starts_with("?>")) {
                out.extend(std::iter::repeat_n(' ', rest[..end + 2].chars().count()));
                i += end + 2;
                continue;
            }
        }
        let c = rest.chars().next().unwrap_or(' ');
        if c == '&' {
            if let Some(n) = entity_length(rest) {
                out.push_str(&rest[..n]);
                i += n;
                continue;
            }
            bare += 1;
            out.push_str("&amp;");
            i += 1;
            continue;
        }
        out.push(c);
        i += c.len_utf8();
    }
    (out, bare)
}

// The length of a predefined entity or a character reference at the start of the text.
fn entity_length(text: &str) -> Option<usize> {
    for name in ["&amp;", "&lt;", "&gt;", "&quot;", "&apos;"] {
        if text.starts_with(name) {
            return Some(name.len());
        }
    }
    let body = text.strip_prefix("&#")?;
    let (digits, hex) = match body.strip_prefix('x') {
        Some(h) => (h, true),
        None => (body, false),
    };
    let n = digits.chars().take_while(|c| if hex { c.is_ascii_hexdigit() } else { c.is_ascii_digit() }).count();
    (n > 0 && digits[n..].starts_with(';')).then(|| text.len() - digits.len() + n + 1)
}

struct Read {
    root: Option<Node>,
    outside: bool,
}

// One reading: strict refuses what an XML parser refuses; lenient reads the first element and notes what lies outside it.
fn read(text: &str, strict: bool) -> Result<Read, XmlError> {
    let mut reader = Reader::from_str(text);
    let config = reader.config_mut();
    config.check_end_names = true;
    config.trim_text(false);
    let lines = Lines::new(text);
    let mut stack: Vec<Node> = Vec::new();
    let mut root: Option<Node> = None;
    let mut outside = false;
    let mut pending = String::new();
    let mut first = true;
    loop {
        let at = reader.buffer_position() as usize;
        let event = match reader.read_event() {
            Ok(e) => e,
            Err(e) => {
                let error = XmlError { message: e.to_string(), line: lines.at(reader.error_position() as usize) };
                if root.is_some() && stack.is_empty() && !strict {
                    return Ok(Read { root, outside: true });
                }
                return Err(error);
            }
        };
        let fail = |message: &str| XmlError { message: message.to_owned(), line: lines.at(at) };
        let is_text = matches!(event, Event::Text(_) | Event::GeneralRef(_));
        if !is_text && !pending.is_empty() {
            let piece = std::mem::take(&mut pending);
            if !piece.chars().all(|c| matches!(c, ' ' | '\t' | '\n' | '\r')) {
                if stack.is_empty() {
                    if strict {
                        return Err(fail("text outside the root element"));
                    }
                    outside = true;
                } else {
                    stack.iter_mut().for_each(|n| n.value.push_str(&piece));
                }
            }
        }
        match event {
            Event::Text(t) => pending.push_str(&t.decode().map_err(|e| fail(&e.to_string()))?),
            Event::GeneralRef(r) => {
                let resolved = match r.resolve_char_ref() {
                    Ok(Some(c)) => c.to_string(),
                    Ok(None) => match &*r.decode().map_err(|e| fail(&e.to_string()))? {
                        "amp" => "&".into(),
                        "lt" => "<".into(),
                        "gt" => ">".into(),
                        "quot" => "\"".into(),
                        "apos" => "'".into(),
                        other => return Err(fail(&format!("reference to undeclared entity '{other}'"))),
                    },
                    Err(e) => return Err(fail(&e.to_string())),
                };
                pending.push_str(&resolved);
            }
            Event::CData(c) => {
                let piece = c.decode().map_err(|e| fail(&e.to_string()))?.into_owned();
                stack.iter_mut().for_each(|n| n.value.push_str(&piece));
            }
            Event::Start(s) | Event::Empty(s) if root.is_some() && stack.is_empty() => {
                let _ = s;
                if strict {
                    return Err(fail("a second root element"));
                }
                return Ok(Read { root, outside: true });
            }
            Event::Start(s) => stack.push(node(&s, lines.at(at)).map_err(|m| fail(&m))?),
            Event::Empty(s) => {
                let n = node(&s, lines.at(at)).map_err(|m| fail(&m))?;
                match stack.last_mut() {
                    Some(parent) => parent.children.push(n),
                    None => root = Some(n),
                }
            }
            Event::End(_) => {
                let n = stack.pop().ok_or_else(|| fail("an end tag with no start"))?;
                match stack.last_mut() {
                    Some(parent) => parent.children.push(n),
                    None => root = Some(n),
                }
            }
            Event::Decl(_) if strict && !first => return Err(fail("an XML declaration not at the start of the file")),
            Event::Eof => {
                if !stack.is_empty() {
                    return Err(XmlError {
                        message: format!("unexpected end of file: <{}> is not closed", stack[0].name),
                        line: lines.at(text.len()),
                    });
                }
                return Ok(Read { root, outside });
            }
            _ => {}
        }
        first = false;
    }
}

fn node(start: &BytesStart, line: u32) -> Result<Node, String> {
    let name = String::from_utf8_lossy(start.local_name().as_ref()).into_owned();
    let mut attributes = Vec::new();
    for attribute in start.attributes().with_checks(true) {
        let attribute = attribute.map_err(|e| e.to_string())?;
        let key = String::from_utf8_lossy(attribute.key.local_name().as_ref()).into_owned();
        let raw = String::from_utf8_lossy(&attribute.value).replace(['\t', '\n'], " ");
        let value = quick_xml::escape::unescape(&raw).map_err(|e| e.to_string())?.into_owned();
        attributes.push((key, value));
    }
    Ok(Node { name, attributes, line, ..Node::default() })
}

// Byte offsets to 1-based lines.
struct Lines(Vec<usize>);

impl Lines {
    fn new(text: &str) -> Self {
        Lines(text.bytes().enumerate().filter(|(_, b)| *b == b'\n').map(|(i, _)| i).collect())
    }

    fn at(&self, offset: usize) -> u32 {
        (self.0.partition_point(|&n| n < offset) + 1) as u32
    }
}

#[cfg(test)]
mod tests {
    use super::parse;

    #[test]
    fn strict_xml_reads_with_values_and_lines() {
        let (root, leniency) = parse("<theme>\n  <a x=\"1\">one &amp; two</a>\n  <b/>\n</theme>").unwrap();
        assert!(leniency.is_none());
        assert_eq!(root.children[0].value, "one & two");
        assert_eq!(root.children[0].line, 2);
        assert_eq!(root.children[1].line, 3);
        assert_eq!(root.children[0].attribute("x"), Some("1"));
    }

    #[test]
    fn a_bare_ampersand_and_text_outside_are_read_leniently() {
        let (root, leniency) = parse("f\n<theme><a>Q & A</a></theme>zz").unwrap();
        assert_eq!(root.children[0].value, "Q & A");
        assert_eq!(leniency.as_deref(), Some("1 bare & kept as text; text outside the root element ignored"));
    }

    #[test]
    fn mismatched_tags_are_refused() {
        assert!(parse("<theme><a>x</b></theme>").is_err());
        assert!(parse("<theme><a>x</theme>").is_err());
    }
}
