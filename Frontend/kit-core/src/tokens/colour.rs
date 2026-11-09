use palette::color_difference::Wcag21RelativeContrast;
use palette::{FromColor, Hsl, Srgb};

/// An sRGB colour with straight alpha, 8 bits a channel, written as Avalonia writes it: `#RRGGBB` or `#AARRGGBB`.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct Colour {
    pub r: u8,
    pub g: u8,
    pub b: u8,
    pub a: u8,
}

impl Colour {
    pub const fn rgb(r: u8, g: u8, b: u8) -> Colour {
        Colour { r, g, b, a: 255 }
    }

    pub const fn rgba(r: u8, g: u8, b: u8, a: u8) -> Colour {
        Colour { r, g, b, a }
    }

    /// `#RRGGBB` or `#AARRGGBB` as a theme writes it, or nothing when it is neither.
    pub fn parse(text: &str) -> Option<Colour> {
        let digits = text.strip_prefix('#')?;
        ((digits.len() == 6 || digits.len() == 8) && digits.chars().all(|c| c.is_ascii_hexdigit())).then(|| Colour::hex(text))
    }

    /// Parses `#RRGGBB` or `#AARRGGBB`; a malformed string is a programming error in a token table, so it panics.
    pub const fn hex(text: &str) -> Colour {
        let bytes = text.as_bytes();
        assert!(bytes[0] == b'#' && (bytes.len() == 7 || bytes.len() == 9), "a colour is #RRGGBB or #AARRGGBB");
        let mut value: u32 = 0;
        let mut i = 1;
        while i < bytes.len() {
            let digit = match bytes[i] {
                b'0'..=b'9' => bytes[i] - b'0',
                b'a'..=b'f' => bytes[i] - b'a' + 10,
                b'A'..=b'F' => bytes[i] - b'A' + 10,
                _ => panic!("a colour's digits are hexadecimal"),
            };
            value = (value << 4) | digit as u32;
            i += 1;
        }
        if bytes.len() == 7 {
            value |= 0xFF00_0000
        }
        Colour { a: (value >> 24) as u8, r: (value >> 16) as u8, g: (value >> 8) as u8, b: value as u8 }
    }

    /// A colour from a hue in degrees (any value, wrapped), saturation and lightness in 0..=1, and an alpha in 0..=1.
    pub fn hsl(hue: f64, saturation: f64, lightness: f64, alpha: f64) -> Colour {
        let hue = ((hue % 360.0) + 360.0) % 360.0;
        let rgb: Srgb<f32> = Srgb::from_color(Hsl::new_srgb(hue as f32, saturation as f32, lightness as f32));
        let rgb = rgb.into_format::<u8>();
        Colour { r: rgb.red, g: rgb.green, b: rgb.blue, a: channel(alpha) }
    }

    /// The colour as `#RRGGBB`, or `#AARRGGBB` when it is not opaque.
    pub fn to_hex(self) -> String {
        if self.a == 255 {
            format!("#{:02X}{:02X}{:02X}", self.r, self.g, self.b)
        } else {
            format!("#{:02X}{:02X}{:02X}{:02X}", self.a, self.r, self.g, self.b)
        }
    }

    /// The alpha as a share, 0 to 1.
    pub fn alpha(self) -> f64 {
        self.a as f64 / 255.0
    }

    /// This colour composited over an opaque background, channel by channel in sRGB as a compositor paints 8-bit layers.
    pub fn over(self, background: Colour) -> Colour {
        let a = self.alpha();
        let mix = |fore: u8, back: u8| (fore as f64 * a + back as f64 * (1.0 - a)).round() as u8;
        Colour::rgb(mix(self.r, background.r), mix(self.g, background.g), mix(self.b, background.b))
    }

    /// The WCAG 2.1 contrast ratio between two opaque colours, 1 to 21 (alpha ignored).
    pub fn contrast(self, other: Colour) -> f64 {
        let linear = |c: Colour| Srgb::new(c.r, c.g, c.b).into_format::<f32>().into_linear();
        linear(self).relative_contrast(linear(other)) as f64
    }
}

fn channel(share: f64) -> u8 {
    (share.clamp(0.0, 1.0) * 255.0).round() as u8
}
