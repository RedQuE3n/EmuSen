-- Seed entries for the known-differences dictionary - see §3.49.
--
-- Everything here is inserted as 'provisional'. Nothing in this file may say
-- 'proven'; the schema's trigger refuses it, deliberately. Promotion happens only
-- when `--verify-dictionary` re-runs an assertion against a real pair of dump
-- sets and records that it passed.

INSERT OR IGNORE INTO definition (slug, title, claim, cause, scope, subject, system)
VALUES (
    'nes-palette-power-on-undefined',
    'NES palette RAM has no defined power-on contents',
    'The palette column never agrees between two emulators, on any ROM, at any alignment.',
    'The NES does not define the contents of palette RAM at power-on. Each emulator picks its own fill, and a game that writes only the entries it uses leaves the rest showing that fill forever.',
    'column', 'palette', 'nes'
);

INSERT OR IGNORE INTO definition (slug, title, claim, cause, scope, subject, system, left_backend)
VALUES (
    'moon-ciram-mirror-is-4k',
    'This core keeps a 4 KB nametable mirror where the reference exposes 2 KB',
    'The nametable column never agrees, because the two sides are not describing the same number of bytes.',
    'Moon allocates the full 4 KB address space and lets the mirroring function fold it, while the reference exposes the 2 KB the console physically carries.',
    'column', 'nametable', 'nes', 'emusen'
);

INSERT OR IGNORE INTO definition (slug, title, claim, cause, scope, subject, system, left_backend)
VALUES (
    'moon-reports-ntsc-unconditionally',
    'This core reports NTSC for every NES image',
    'The identity region field reads ntsc from emusen regardless of what the image is, so a PAL image produces a region mismatch against any reference that detects one.',
    'PeerProbe has no region detection to report; whether the core models PAL timing at all is a separate and open question.',
    'identity', 'region', 'nes', 'emusen'
);

INSERT OR IGNORE INTO definition (slug, title, claim, cause, scope, subject, system)
VALUES (
    'frame-boundary-is-not-the-same-instant',
    'End-of-frame memory is sampled at different points by different emulators',
    'The ram column agreement rate varies from near zero to well over half between ROMs whose frames are pixel for pixel identical, so it cannot by itself carry a divergence verdict.',
    'Two emulators stop on a frame boundary at different points relative to the game''s own update, so a space the game is midway through writing is caught at different moments.',
    'column', 'ram', 'nes'
);

-- Kept as a worked example of the thing this table exists to prevent. It was
-- written into two man pages as fact, on no evidence, and was false.
INSERT OR IGNORE INTO definition (slug, title, claim, cause, scope, subject, status, retracted_at, retracted_why)
VALUES (
    'mesen-probe-overrides-mapper-from-game-database',
    'RETRACTED: the reference overrides a damaged mapper number from its game database during probe runs',
    'A probe run of the reference resolves the board from MesenNesDB.txt rather than from the image header, so two emulators can be running different boards on the same file.',
    NULL,
    'identity', 'board', 'retracted', '2026-08-08',
    'GameDatabase::InitDatabase reads MesenNesDB.txt from the emulator home folder, and the probe points that at its own dump directory, which never contains one. No database is loaded in a probe run. The identity gate reports mesen board=65 against emusen board=65 on the image this was claimed about. The mechanism exists in the reference but does not operate here, and the claim was made because it was available rather than because it was measured.'
);

INSERT OR IGNORE INTO definition (slug, title, claim, cause, scope, subject, system, left_backend)
VALUES (
    'moon-h3001-magic-kingdom-blank',
    'Adventures in the Magic Kingdom (U) [a1] renders nothing on this core',
    'With both sides resolving board 65, emusen draws a single flat colour where the reference draws content, differing in 87.53% of pixels at frame 300.',
    NULL,
    'screen', 'board-65', 'nes', 'emusen'
);

-- Assertions: the falsifiable half. Each is a predicate the verifier re-runs.
INSERT OR IGNORE INTO assertion (definition_id, kind, subject, op, value, fixture)
SELECT id, 'column-agreement', 'palette', '<=', 0.05, 'any nes pair'
FROM definition WHERE slug = 'nes-palette-power-on-undefined';

INSERT OR IGNORE INTO assertion (definition_id, kind, subject, op, value, fixture)
SELECT id, 'column-agreement', 'nametable', '<=', 0.05, 'any nes pair'
FROM definition WHERE slug = 'moon-ciram-mirror-is-4k';

INSERT OR IGNORE INTO assertion (definition_id, kind, subject, op, value, fixture)
SELECT id, 'identity-field', 'region', '<>', 0, 'Mr. Gimmick'
FROM definition WHERE slug = 'moon-reports-ntsc-unconditionally';

INSERT OR IGNORE INTO evidence (definition_id, kind, description, detail)
SELECT id, 'measurement', 'palette agreement over ~300 frames on three unrelated ROMs',
       'Klax 0.0%, After Burner 0.0%, Adventures in the Magic Kingdom 0.0%'
FROM definition WHERE slug = 'nes-palette-power-on-undefined';

INSERT OR IGNORE INTO evidence (definition_id, kind, description, detail)
SELECT id, 'measurement', 'ram agreement against pixel-identical frames',
       'Klax 62.0% and After Burner 0.3%, both 0 differing pixels; Magic Kingdom 1.0% with 87.53% differing'
FROM definition WHERE slug = 'frame-boundary-is-not-the-same-instant';

INSERT OR IGNORE INTO evidence (definition_id, kind, description, detail)
SELECT id, 'source', 'GameDatabase::InitDatabase in the reference checkout',
       'Reads FolderUtilities::GetHomeFolder() + "MesenNesDB.txt"; the probe sets the home folder to its dump directory'
FROM definition WHERE slug = 'mesen-probe-overrides-mapper-from-game-database';

INSERT OR IGNORE INTO evidence (definition_id, kind, description, detail)
SELECT id, 'counterexample', 'forcing the same image to board 4 does not make it render',
       'Header nibbles transposed; blank as mapper 65 and blank as mapper 4, so the dump is damaged beyond its mapper number - but that does not clear board 65'
FROM definition WHERE slug = 'moon-h3001-magic-kingdom-blank';
