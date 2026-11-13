//! What a disc image is, read from a few sectors and never changed: a PS2
//! disc's serial (the BOOT2 line of SYSTEM.CNF, as the game checks it) or an
//! Xbox image (its "MICROSOFT*XBOX*MEDIA" volume). Formats: .iso, raw .bin
//! (2352-byte sectors, mode 1 or 2) and .cue (its first FILE).

use serde::Serialize;
use std::fs::File;
use std::io::{Read, Seek, SeekFrom};
use std::path::{Path, PathBuf};

#[derive(Clone, Serialize)]
pub struct DiscCheck {
    /// The image is the disc the kind expects.
    pub good: bool,
    /// What it is, for the page (French).
    pub text: String,
}

/// What a kind of the catalogue ("ps2_us", "xbox_us") expects: the PS2
/// serial, or "xbox"; and its name.
fn expected(kind: &str) -> Option<(&'static str, &'static str)> {
    match kind {
        "ps2_us" => Some(("SLUS_210.50", "Burnout 3: Takedown PS2 US")),
        "xbox_us" => Some(("xbox", "Burnout 3: Takedown Xbox")),
        _ => None,
    }
}

/// Is the image at `path` the disc of that kind?
pub fn check(kind: &str, path: &Path) -> DiscCheck {
    let bad = |text: String| DiscCheck { good: false, text };
    let Some((wanted, name)) = expected(kind) else {
        return bad(format!("Disque inconnu du lanceur : {kind}"));
    };
    if !path.is_file() {
        return bad("Fichier introuvable".into());
    }
    let Some(mut image) = Image::open(path) else {
        return bad("Image illisible".into());
    };
    if let Some(serial) = image.ps2_serial() {
        if serial.eq_ignore_ascii_case(wanted) {
            return DiscCheck { good: true, text: format!("{name} ({serial})") };
        }
        return bad(format!("Un autre disque PS2 ({serial}) : il faut {name} ({wanted})"));
    }
    if image.is_xbox() {
        if wanted == "xbox" {
            // The title inside default.xbe is not read: an Xbox image is taken.
            return DiscCheck { good: true, text: "Image Xbox (titre non vérifié)".into() };
        }
        return bad(format!("Une image Xbox : il faut {name}"));
    }
    bad(format!("Ce n'est pas une image de {name}"))
}

/// The disc a version plays when none is chosen in the launcher: its
/// pc_settings.ini's `disc=`, else an image of that kind beside its exe
/// (what the game itself does).
pub fn own_disc(kind: &str, folder: &Path) -> Option<PathBuf> {
    if let Some(disc) = crate::gameini::get(folder, "disc") {
        let disc = PathBuf::from(disc);
        if disc.is_file() {
            return Some(disc);
        }
    }
    let mut beside: Vec<PathBuf> = std::fs::read_dir(folder)
        .ok()?
        .flatten()
        .map(|e| e.path())
        .filter(|p| {
            let ext = p.extension().map(|e| e.to_string_lossy().to_lowercase()).unwrap_or_default();
            matches!(ext.as_str(), "iso" | "bin" | "cue")
        })
        .collect();
    beside.sort();
    beside.into_iter().find(|p| check(kind, p).good)
}

const SYNC: [u8; 12] = [0x00, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0x00];
const XBOX_MAGIC: &[u8] = b"MICROSOFT*XBOX*MEDIA";

struct Image {
    file: File,
    /// 0: 2048-byte sectors; else the data's offset in a 2352-byte sector.
    raw: u64,
}

impl Image {
    fn open(path: &Path) -> Option<Image> {
        let is_cue = path.extension().is_some_and(|e| e.eq_ignore_ascii_case("cue"));
        let image = if is_cue { cue_image(path)? } else { path.to_path_buf() };
        let mut file = File::open(image).ok()?;
        let mut head = [0u8; 16];
        let raw = if read_at(&mut file, 16 * 2352, &mut head) && head[..12] == SYNC {
            match head[15] {
                1 => 16,
                2 => 24,
                _ => 0,
            }
        } else {
            0
        };
        Some(Image { file, raw })
    }

    fn sector(&mut self, lsn: u32, out: &mut [u8; 2048]) -> bool {
        let at = if self.raw == 0 { lsn as u64 * 2048 } else { lsn as u64 * 2352 + self.raw };
        read_at(&mut self.file, at, out)
    }

    /// SYSTEM.CNF's BOOT2 ("cdrom0:\SLUS_210.50;1") as "SLUS_210.50".
    fn ps2_serial(&mut self) -> Option<String> {
        let mut sector = [0u8; 2048];
        if !self.sector(16, &mut sector) || &sector[1..6] != b"CD001" {
            return None;
        }
        let root = &sector[156..];
        let (mut lsn, size) = (le32(&root[2..]), le32(&root[10..]));
        let end = lsn + size.div_ceil(2048).min(16);
        let mut cnf = None;
        'dirs: while lsn < end {
            if !self.sector(lsn, &mut sector) {
                return None;
            }
            let mut at = 0usize;
            while at + 33 < 2048 {
                let length = sector[at] as usize;
                if length == 0 || at + length > 2048 {
                    break;
                }
                let name_len = sector[at + 32] as usize;
                let name = &sector[at + 33..(at + 33 + name_len).min(2048)];
                if name.len() >= 10 && name[..10].eq_ignore_ascii_case(b"SYSTEM.CNF") {
                    cnf = Some((le32(&sector[at + 2..]), le32(&sector[at + 10..])));
                    break 'dirs;
                }
                at += length;
            }
            lsn += 1;
        }
        let (cnf_lsn, cnf_size) = cnf?;
        if !self.sector(cnf_lsn, &mut sector) {
            return None;
        }
        let text = String::from_utf8_lossy(&sector[..(cnf_size as usize).min(2048)]).to_string();
        let line = text.lines().find(|l| l.split('=').next().is_some_and(|k| k.trim().eq_ignore_ascii_case("BOOT2")))?;
        let value = line.split_once('=')?.1.trim();
        let value = value.split_once(':').map_or(value, |(_, v)| v);
        let value = value.split(';').next()?.trim_start_matches(['\\', '/']);
        (!value.is_empty()).then(|| value.to_string())
    }

    /// An Xbox volume at one of the places images put it (an extracted XISO,
    /// a full Redump image, other dumps' offsets).
    fn is_xbox(&mut self) -> bool {
        let mut magic = [0u8; 20];
        [0u64, 0x1830_0000, 0x0FD9_0000, 0x0208_0000]
            .iter()
            .any(|base| read_at(&mut self.file, base + 0x10000, &mut magic) && magic == XBOX_MAGIC)
    }
}

fn read_at(file: &mut File, at: u64, out: &mut [u8]) -> bool {
    file.seek(SeekFrom::Start(at)).is_ok() && file.read_exact(out).is_ok()
}

fn le32(b: &[u8]) -> u32 {
    u32::from_le_bytes([b[0], b[1], b[2], b[3]])
}

/// The first FILE of a .cue, beside it.
fn cue_image(cue: &Path) -> Option<PathBuf> {
    let text = std::fs::read_to_string(cue).ok()?;
    let line = text.lines().map(str::trim).find(|l| l.len() > 5 && l[..5].eq_ignore_ascii_case("FILE "))?;
    let rest = line[5..].trim();
    let name = if let Some(quoted) = rest.strip_prefix('"') {
        quoted.split('"').next()?
    } else {
        rest.split_whitespace().next()?
    };
    Some(cue.parent()?.join(name))
}
