//! A time zone read as .NET's `TimeZoneInfo.FindSystemTimeZoneById` reads it on Linux: the system's TZif file
//! (`$TZDIR`, else `/usr/share/zoneinfo`), its transitions, and past the last of them the POSIX rule in its footer.
//! The scraper keeps ScreenScraper's day as Paris's (EmuSen_BigPicture.md §17.4); with no file, the C# falls back to
//! UTC, and so does this. See EmuSen_Backend.md §30.5.

use emusen_dotnet::date::{TICKS_PER_SECOND, days_from_civil, weekday};

/// One POSIX rule's change: month, week (5 the last), weekday and the local second of the day it happens at.
#[derive(Clone, Copy, Debug, PartialEq)]
struct Change {
    month: i64,
    week: i64,
    day: i64,
    at: i64,
}

#[derive(Clone, Debug, PartialEq)]
struct Rule {
    standard: i64,
    daylight: Option<(i64, Change, Change)>,
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct Zone {
    /// Each transition's UTC second and the offset in seconds that holds from it.
    transitions: Vec<(i64, i64)>,
    /// The offset before the first transition.
    first: i64,
    rule: Option<Rule>,
}

impl Zone {
    pub fn utc() -> Zone {
        Zone::default()
    }

    /// The named zone from the system's files, or UTC where it cannot be read.
    pub fn system(name: &str) -> Zone {
        let dir = std::env::var("TZDIR").unwrap_or_else(|_| "/usr/share/zoneinfo".to_string());
        std::fs::read(std::path::Path::new(&dir).join(name)).ok().and_then(|bytes| Zone::parse(&bytes)).unwrap_or_default()
    }

    fn parse(b: &[u8]) -> Option<Zone> {
        let be32 = |at: usize| -> Option<i64> { Some(i32::from_be_bytes(b.get(at..at + 4)?.try_into().ok()?) as i64) };
        let be64 = |at: usize| -> Option<i64> { Some(i64::from_be_bytes(b.get(at..at + 8)?.try_into().ok()?)) };
        let header = |at: usize| -> Option<[usize; 6]> {
            if b.get(at..at + 4)? != b"TZif" {
                return None;
            }
            let mut counts = [0usize; 6];
            for (i, c) in counts.iter_mut().enumerate() {
                *c = be32(at + 20 + 4 * i)? as usize;
            }
            Some(counts)
        };
        let [isut, isstd, leap, time, types, chars] = header(0)?;
        let version = *b.get(4)?;
        let v1_len = 44 + time * 5 + types * 6 + chars + leap * 8 + isstd + isut;
        let (start, wide) = if version >= b'2' { (v1_len, true) } else { (0, false) };
        let [isut, isstd, leap, time, types, chars] = if wide { header(start)? } else { [isut, isstd, leap, time, types, chars] };
        let size = if wide { 8 } else { 4 };
        let mut at = start + 44;
        let times: Vec<i64> = (0..time).map(|i| if wide { be64(at + i * 8) } else { be32(at + i * 4) }).collect::<Option<_>>()?;
        at += time * size;
        let indexes: Vec<usize> = b.get(at..at + time)?.iter().map(|&i| i as usize).collect();
        at += time;
        let offsets: Vec<i64> = (0..types).map(|i| be32(at + i * 6)).collect::<Option<_>>()?;
        at += types * 6 + chars + leap * (size + 4) + isstd + isut;
        let transitions = times.iter().zip(&indexes).map(|(&t, &i)| offsets.get(i).map(|&o| (t, o))).collect::<Option<Vec<_>>>()?;
        let first = offsets.first().copied().unwrap_or(0);
        let rule = if wide {
            let footer = b.get(at..)?;
            let text = std::str::from_utf8(footer).ok()?.trim_matches('\n');
            rule(text.lines().next().unwrap_or(""))
        } else {
            None
        };
        Some(Zone { transitions, first, rule })
    }

    /// The offset in seconds at a UTC second.
    pub fn offset_at(&self, utc: i64) -> i64 {
        match self.transitions.iter().rposition(|&(t, _)| t <= utc) {
            Some(i) if i + 1 < self.transitions.len() || self.rule.is_none() => self.transitions[i].1,
            None if !self.transitions.is_empty() => self.first,
            _ => match &self.rule {
                Some(rule) => rule.offset_at(utc),
                None => self.first,
            },
        }
    }

    /// `ConvertTime`'s local time, in ticks, of a UTC instant in ticks.
    pub fn local(&self, utc_ticks: i64) -> i64 {
        utc_ticks + self.offset_at(utc_ticks.div_euclid(TICKS_PER_SECOND)) * TICKS_PER_SECOND
    }

    /// `GetUtcOffset` of a local time with no offset, in seconds: the offset that maps some instant to it, the standard
    /// one where two do or none does.
    pub fn offset_of_local(&self, local_ticks: i64) -> i64 {
        let local = local_ticks.div_euclid(TICKS_PER_SECOND);
        let mut candidates: Vec<i64> = Vec::new();
        for guess in [local - 86_400, local, local + 86_400] {
            let o = self.offset_at(guess);
            if self.offset_at(local - o) == o && !candidates.contains(&o) {
                candidates.push(o);
            }
        }
        let standard = self.rule.as_ref().map_or(self.first, |r| r.standard);
        match candidates.as_slice() {
            [one] => *one,
            _ => standard,
        }
    }
}

/// A POSIX TZ rule such as `CET-1CEST,M3.5.0,M10.5.0/3`; only the `M` form of change, which every European zone uses.
fn rule(text: &str) -> Option<Rule> {
    let mut rest = text;
    let name = |s: &mut &str| -> Option<()> {
        if let Some(r) = s.strip_prefix('<') {
            let end = r.find('>')?;
            *s = &r[end + 1..];
        } else {
            let end = s.find(|c: char| !c.is_ascii_alphabetic()).unwrap_or(s.len());
            if end < 3 {
                return None;
            }
            *s = &s[end..];
        }
        Some(())
    };
    let clock = |s: &mut &str| -> Option<i64> {
        let (sign, body) = match s.as_bytes().first()? {
            b'-' => (-1, &s[1..]),
            b'+' => (1, &s[1..]),
            _ => (1, *s),
        };
        let end = body.find(|c: char| !(c.is_ascii_digit() || c == ':')).unwrap_or(body.len());
        let mut parts = body[..end].split(':').map(|p| p.parse::<i64>());
        let h = parts.next()?.ok()?;
        let m = parts.next().transpose().ok()?.unwrap_or(0);
        let sec = parts.next().transpose().ok()?.unwrap_or(0);
        *s = &body[end..];
        Some(sign * (h * 3600 + m * 60 + sec))
    };
    name(&mut rest)?;
    let standard = -clock(&mut rest)?;
    if rest.is_empty() {
        return Some(Rule { standard, daylight: None });
    }
    name(&mut rest)?;
    let daylight = if rest.starts_with(',') { standard + 3600 } else { -clock(&mut rest)? };
    let change = |s: &mut &str| -> Option<Change> {
        let body = s.strip_prefix(",M")?;
        let end = body.find([',', '/']).unwrap_or(body.len());
        let mut f = body[..end].split('.').map(|p| p.parse::<i64>().ok());
        let (month, week, day) = (f.next()??, f.next()??, f.next()??);
        *s = &body[end..];
        let at = if let Some(r) = s.strip_prefix('/') {
            *s = r;
            clock(s)?
        } else {
            7200
        };
        Some(Change { month, week, day, at })
    };
    let start = change(&mut rest)?;
    let end = change(&mut rest)?;
    Some(Rule { standard, daylight: Some((daylight, start, end)) })
}

impl Change {
    /// The UTC second of this change in a year, from local time under the offset that held before it.
    fn utc(&self, year: i64, offset_before: i64) -> i64 {
        let first = days_from_civil(year, self.month, 1);
        let first_day = weekday(first) as i64;
        let mut day = first + (self.day - first_day).rem_euclid(7) + (self.week - 1) * 7;
        let next_month = if self.month == 12 { days_from_civil(year + 1, 1, 1) } else { days_from_civil(year, self.month + 1, 1) };
        while day >= next_month {
            day -= 7;
        }
        day * 86_400 + self.at - offset_before
    }
}

impl Rule {
    fn offset_at(&self, utc: i64) -> i64 {
        let Some((daylight, start, end)) = self.daylight else { return self.standard };
        let year = emusen_dotnet::date::civil((utc + self.standard).div_euclid(86_400)).0;
        let (on, off) = (start.utc(year, self.standard), end.utc(year, daylight));
        let summer = if on < off { utc >= on && utc < off } else { utc >= on || utc < off };
        if summer { daylight } else { self.standard }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn at(text: &str) -> i64 {
        emusen_dotnet::date::instant(text).unwrap() / TICKS_PER_SECOND
    }

    #[test]
    fn the_rule_alone_gives_pariss_two_changes() {
        let z = Zone { transitions: Vec::new(), first: 3600, rule: rule("CET-1CEST,M3.5.0,M10.5.0/3") };
        assert_eq!(z.offset_at(at("2026-03-29T00:59:59Z")), 3600);
        assert_eq!(z.offset_at(at("2026-03-29T01:00:00Z")), 7200);
        assert_eq!(z.offset_at(at("2026-10-25T00:59:59Z")), 7200);
        assert_eq!(z.offset_at(at("2026-10-25T01:00:00Z")), 3600);
        assert_eq!(z.offset_of_local(emusen_dotnet::date::instant("2026-10-19T00:00:00Z").unwrap()), 7200);
        assert_eq!(z.offset_of_local(emusen_dotnet::date::instant("2026-12-19T00:00:00Z").unwrap()), 3600);
    }

    #[test]
    fn the_systems_paris_agrees_where_it_is_installed() {
        let z = Zone::system("Europe/Paris");
        if z == Zone::utc() {
            eprintln!("no Europe/Paris on this machine: nothing was run");
            return;
        }
        assert_eq!(z.offset_at(at("2026-07-14T10:00:00Z")), 7200);
        assert_eq!(z.offset_at(at("2026-01-14T10:00:00Z")), 3600);
        assert_eq!(z.offset_at(at("2040-07-14T10:00:00Z")), 7200);
        assert_eq!(z.offset_at(at("1990-07-14T10:00:00Z")), 7200);
    }
}
