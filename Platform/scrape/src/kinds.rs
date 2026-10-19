//! The kinds of media the scraper fetches: ES-DE's type name and folder, ScreenScraper's names for it in the order
//! tried, and what its file must be. The one table of ES-DE's type folders (EmuSen_Backend.md §30.1); the C#
//! `ScrapeRules` kinds and Big Picture's `EsdeMediaFolder.Folders` read it in state 2. See EmuSen_BigPicture.md §5.4, §38.

/// What a kind's file must be to be kept.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Payload {
    Image,
    Pdf,
    Video,
}

impl Payload {
    pub fn word(self) -> &'static str {
        match self {
            Payload::Image => "image",
            Payload::Pdf => "pdf",
            Payload::Video => "video",
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Kind {
    pub esde_type: &'static str,
    pub folder: &'static str,
    pub screenscraper: &'static [&'static str],
    pub payload: Payload,
}

const fn kind(esde_type: &'static str, folder: &'static str, screenscraper: &'static [&'static str], payload: Payload) -> Kind {
    Kind { esde_type, folder, screenscraper, payload }
}

pub const COVER: Kind = kind("cover", "covers", &["box-2D"], Payload::Image);
pub const SCREENSHOT: Kind = kind("screenshot", "screenshots", &["ss"], Payload::Image);
pub const MARQUEE: Kind = kind("marquee", "marquees", &["wheel-hd", "wheel"], Payload::Image);
pub const TITLE_SCREEN: Kind = kind("titlescreen", "titlescreens", &["sstitle"], Payload::Image);
pub const MIXIMAGE: Kind = kind("miximage", "miximages", &["mixrbv2"], Payload::Image);
pub const BACK_COVER: Kind = kind("backcover", "backcovers", &["box-2D-back"], Payload::Image);
pub const BOX_3D: Kind = kind("3dbox", "3dboxes", &["box-3D"], Payload::Image);
pub const PHYSICAL_MEDIA: Kind = kind("physicalmedia", "physicalmedia", &["support-2D"], Payload::Image);
pub const FAN_ART: Kind = kind("fanart", "fanart", &["fanart"], Payload::Image);
pub const MANUAL: Kind = kind("manual", "manuals", &["manuel"], Payload::Pdf);
pub const VIDEO: Kind = kind("video", "videos", &["video-normalized"], Payload::Video);

/// The kinds before Pass 8: an answer recorded then listed only these as offered, so it says nothing of the others.
pub const FIRST: [Kind; 5] = [COVER, SCREENSHOT, MARQUEE, TITLE_SCREEN, MIXIMAGE];

/// Every kind, in `ScrapeRules.AllKinds`' order.
pub const ALL: [Kind; 11] = [COVER, SCREENSHOT, MARQUEE, TITLE_SCREEN, MIXIMAGE, BACK_COVER, BOX_3D, PHYSICAL_MEDIA, FAN_ART, MANUAL, VIDEO];

pub fn by_type(esde_type: &str) -> Option<Kind> {
    ALL.iter().copied().find(|k| k.esde_type == esde_type)
}

/// The extensions ES-DE's guide lists for a type's files ("Manually copying game media files"), in the order tried.
pub fn extensions(esde_type: &str) -> &'static [&'static str] {
    match esde_type {
        "video" => &[".mp4", ".mkv", ".avi", ".wmv", ".mov", ".webm"],
        "manual" => &[".pdf"],
        _ => &[".png", ".jpg", ".webp"],
    }
}

/// The table as `get {"what": "kinds"}` answers it.
pub fn to_json() -> String {
    let mut out = String::from("[");
    for (i, k) in ALL.iter().enumerate() {
        if i > 0 {
            out.push(',');
        }
        out.push_str(&format!("{{\"type\":\"{}\",\"folder\":\"{}\",\"screenscraper\":[", k.esde_type, k.folder));
        out.push_str(&k.screenscraper.iter().map(|t| format!("\"{t}\"")).collect::<Vec<_>>().join(","));
        out.push_str(&format!("],\"payload\":\"{}\"}}", k.payload.word()));
    }
    out.push(']');
    out
}
