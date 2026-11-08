/* emusen_crystal.h - the frontend API, version 2.0. Owned by Crystal.
 *
 * How every frontend drives the cores: the runtime (the systems, the engines, a file's route), the session's life (a
 * game or a console started empty, discs, reset, close), one frame call that performs the whole frame-end order and
 * hands over the picture and the sound, input, speed, rewind, states, settings, firmware, the save media's cards,
 * events, and the debugger, which is the session service's table, emusen_session.h, handed out by
 * emusen_crystal_session_service. The normative text is EmuSen_Backend.md in the EmuSen manual, section 3 for the
 * design and section 14 for version 1.0 as built; 2.0 is EmuSen_RustPlatform.md section 47.23. Where the two disagree
 * the page wins and this header is in error.
 *
 * Names: exports emusen_crystal_*, types emusen_crystal_*, constants EMUSEN_CRYSTAL_*. The statuses and the
 * enumerations the session service already names (EMUSEN_SVC_*, EMUSEN_SESSION_*) are this header's too, which is why
 * it includes that header: the table's statuses are this API's too.
 *
 * Rules every function follows (EmuSen_Backend.md section 3.3, the core ABI's one layer up):
 *  - Version. emusen_crystal_abi_version() is (MAJOR << 16) | MINOR. The major is 2: 2.0 removed the session
 *    service's twins, whose one surface is the table session_service hands out, and kept the rest of 1.13 unchanged.
 *    A minor adds functions, appends fields to size-prefixed structs, adds values to open enumerations and keys to
 *    JSON. Nothing is removed within a major. A caller checks the major before calling anything else.
 *  - Pull, never push. The runtime calls nothing of the client's: notifications are drained from a subscription,
 *    and a client that must wait blocks in emusen_crystal_wait or emusen_crystal_subscription_wait with a timeout.
 *  - Handles for what has a life (the runtime, a subscription); 32-bit ids for sessions, numbered from 1. Session 0
 *    names the current session: the one opened last that is still open.
 *  - Statuses. A negative return is a status; zero or more is success: a length, an id, a count or a truth value, as
 *    each function says. The words for a failure are read with emusen_crystal_last_error on the same thread.
 *    EMUSEN_SVC_ABSENT is an answer, not a failure.
 *  - Text is UTF-8, passed as a pointer and a length in bytes and never assumed to end in a NUL. A null pointer with
 *    a zero length is the absent string.
 *  - Copy out. A function that fills a buffer takes (out, len) and returns the whole length; it writes only when the
 *    buffer holds all of it, and a null out asks the length. Nothing passed in is kept after the call, and no pointer
 *    into a session is returned, except the session service's table, which lives as long as its session.
 *  - Shapes. Lists, records and descriptors are JSON, keys in snake_case, read once and never on the per-frame path.
 *    The per-frame calls take size-prefixed structs and plain buffers. A caller skips keys it does not know.
 *  - Structs. Every struct begins with uint32_t size, set by its allocator and never written by the callee, which
 *    reads and writes only the prefix both know. A struct shorter than its 1.0 size is refused with
 *    EMUSEN_CRYSTAL_BAD_STRUCT.
 *  - Threads. A session has one frame thread, the thread that calls emusen_crystal_frame. A read is answered on any
 *    thread from the session as it stands, never waiting for a frame. A change called from another thread waits for
 *    the frame in progress and is applied at its boundary; a change called on the frame thread between frames is
 *    applied at once. A frame called from a second thread while another is inside one is refused with
 *    EMUSEN_CRYSTAL_BUSY. open may be called on any thread; close from another thread waits for the frame in
 *    progress. Each function's comment says whether it reads or changes.
 *  - Nothing unwinds. A failure inside the runtime is returned as EMUSEN_SVC_FAULT with its words.
 */
#ifndef EMUSEN_CRYSTAL_H
#define EMUSEN_CRYSTAL_H

#include <stddef.h>
#include <stdint.h>

#include "emusen_session.h"

#ifdef __cplusplus
extern "C" {
#endif

#define EMUSEN_CRYSTAL_MAJOR 2u
#define EMUSEN_CRYSTAL_MINOR 0u
#define EMUSEN_CRYSTAL_VERSION ((EMUSEN_CRYSTAL_MAJOR << 16) | EMUSEN_CRYSTAL_MINOR)

/* ---- Statuses (EmuSen_Backend.md section 3.2). The session service's keep their numbers and names; these follow. -- */
#define EMUSEN_CRYSTAL_NO_ENGINE  (-1109) /* no engine opens the file or starts the console; the routing decision's words */
#define EMUSEN_CRYSTAL_FIRMWARE   (-1110) /* the game needs firmware the library does not hold; firmware_needed lists it */
#define EMUSEN_CRYSTAL_BUSY       (-1111) /* a frame called while another thread is inside one */
#define EMUSEN_CRYSTAL_BAD_STRUCT (-1112) /* a struct shorter than its 1.0 size */
#define EMUSEN_CRYSTAL_IO         (-1113) /* reading or writing a file failed; the system's words */
#define EMUSEN_CRYSTAL_STATE_REFUSED (-1114) /* since 1.1: the state is not this machine's: another console's, a version
                                              * outside machine info's loads_from, or the core's own refusal, whose
                                              * words follow; the machine is unchanged */
#define EMUSEN_CRYSTAL_NOT_APPROVED (-1115) /* since 1.3: the engine's library is in no cores manifest and the player has
                                             * not approved it; the words name the library, and cores gives its hash
                                             * (section 29.40) */

/* ---- Open enumerations ------------------------------------------------------------------------------------- */

/* open's flags. */
#define EMUSEN_CRYSTAL_OPEN_NO_BATTERY  1u /* the game's battery save is neither read nor written */
#define EMUSEN_CRYSTAL_OPEN_SINGLE_ROWS 2u /* picture_copy gives each row once, and the frame's row_repeat says how often
                                            * it is shown; without it the rows come repeated and row_repeat is 1 */
#define EMUSEN_CRYSTAL_OPEN_RING        4u /* since 1.4: with a device, a machine whose core claims DEVICE_RING is lent a
                                            * ring made on it in place of a lend; frame_info's ring fields and
                                            * picture_ring say where each picture lies (EmuSen_Backend.md section 33) */

/* speed_mode's mode: which of speed_set's three speeds applies. */
#define EMUSEN_CRYSTAL_SPEED_NORMAL 0u
#define EMUSEN_CRYSTAL_SPEED_TURBO  1u
#define EMUSEN_CRYSTAL_SPEED_SLOW   2u

/* speed_set's flags. */
#define EMUSEN_CRYSTAL_SPEED_DRAW_NONE 1u /* frames advance and none is drawn, whatever the speed */

/* settings_get's and settings_set's scope. A value is read game over console over the schema's default. */
#define EMUSEN_CRYSTAL_SCOPE_CONSOLE 0u
#define EMUSEN_CRYSTAL_SCOPE_GAME    2u /* 1 is kept for an engine's own layer */

/* session_service's flags. */
#define EMUSEN_CRYSTAL_SERVICE_READ_ONLY 1u /* every change refused with EMUSEN_SVC_READ_ONLY (EmuSen_DianaSh.md 6.5) */
#define EMUSEN_CRYSTAL_SERVICE_NO_GAME 2u   /* since 1.12: the runtime's table with no game, whatever is open (EmuSen_Backend.md 40.7) */

/* Event kinds, as bits; subscribe's kinds of 0 asks for every kind. Each event is a JSON object with "kind",
 * "session" and the fields its kind names. */
#define EMUSEN_CRYSTAL_EVENT_SESSION  1u   /* {"op":"opened"|"closed"} */
#define EMUSEN_CRYSTAL_EVENT_RUN      2u   /* {"op":"paused"|"resumed"|"halted"|"stepped","processor","address"} */
#define EMUSEN_CRYSTAL_EVENT_STATE    4u   /* {"op":"saved"|"loaded","path"}: after loaded, a client resets its audio rate control */
#define EMUSEN_CRYSTAL_EVENT_REWIND   8u   /* {"op":"stepped"|"moved"|"cleared","frame"} */
#define EMUSEN_CRYSTAL_EVENT_SETTINGS 16u  /* {"scope","key","value"} */
#define EMUSEN_CRYSTAL_EVENT_DISC     32u  /* {"op":"inserted"|"ejected"|"changed"|"lid","index","open"} */
#define EMUSEN_CRYSTAL_EVENT_LOG      64u  /* {"lines":[text],"source"}: lines oldest first; since 1.2 "source" is "core"
                                            * for what the core logged and "runtime" for the runtime's own lines (the
                                            * firmware a load used, a battery file copied in, a card's notice, the
                                            * recorder's words), which a program files where its own log goes */

/* ---- Types ------------------------------------------------------------------------------------------------- */

typedef struct emusen_crystal_runtime emusen_crystal_runtime;
typedef struct emusen_crystal_subscription emusen_crystal_subscription;

/* config's flags, since 1.2. */
#define EMUSEN_CRYSTAL_CONFIG_DEVELOPMENT 1u /* cores still in development are listed; without it, as the variable
                                              * EMUSEN_DEVELOPMENT_CORES says */

typedef struct emusen_crystal_config { /* 48 bytes since 1.3; 32 in 1.2; 8 in 1.0 */
    uint32_t size;
    uint32_t flags;                    /* EMUSEN_CRYSTAL_CONFIG_*; none in 1.0 and 1.1 */
    void *sqlite;                      /* since 1.2: the host's loaded SQLite (the e_sqlite3 the program ships), lent
                                        * for the runtime's life so the process holds one SQLite; states.db is reached
                                        * through it, and with none the records' calls answer NOT_SUPPORTED */
    const uint8_t *cores;              /* since 1.2: the cores directories as a JSON array of paths, scanned in its
                                        * order; absent for the directories the implementation uses by default, the
                                        * program's own and its cores folder */
    size_t cores_len;
    const uint8_t *player_cores;       /* since 1.3: the player's own cores directories as a JSON array of paths, scanned
                                        * after cores, their libraries loaded only once approved; absent for the
                                        * player's /Cores where cores is absent too, and for none where it is not */
    size_t player_cores_len;
} emusen_crystal_config;

typedef struct emusen_crystal_open_params { /* 72 bytes since 1.1; 64 in 1.0 */
    uint32_t size;
    uint32_t flags;                    /* EMUSEN_CRYSTAL_OPEN_* */
    const uint8_t *path;               /* the game; absent to start a console empty */
    size_t path_len;
    const uint8_t *console;            /* the console started empty, by its name or a system id; absent with a game */
    size_t console_len;
    const uint8_t *engine;             /* an engine by name; absent for the settings' choice, game over console */
    size_t engine_len;
    uint32_t audio_buffer_max_samples; /* the most samples the core holds undrained; 0 is the runtime's default */
    uint32_t reserved;                 /* 0 */
    const void *device;                /* since 1.1: the presenter's Vulkan device, the platform library's
                                        * emusen_serenity_device, or null; each machine of the session is lent it, and
                                        * its pictures are taken there inside frame (section 14.12) */
} emusen_crystal_open_params;

typedef struct emusen_crystal_frame_info {  /* 136 bytes since 1.4, 112 since 1.1, 104 in 1.0; filled by frame and rewind_step */
    uint32_t size;
    uint32_t ran;                      /* 1 when a frame completed; 0 while paused or halted, or when one halted */
    uint32_t halted;                   /* a breakpoint or a step stopped the session inside a frame */
    uint32_t rendered;                 /* the picture is new: 0 on a frame speed left undrawn, or when nothing ran */
    uint64_t picture_serial;           /* the core's frame serial; unchanged means the same picture (core ABI 6.6) */
    int32_t width;                     /* the picture as picture_copy gives it */
    int32_t height;
    int32_t row_repeat;                /* how often each row is shown; 1 unless opened with SINGLE_ROWS */
    int32_t display_lines;             /* the console's own lines in the picture; 0 when its rows are its lines */
    double display_aspect;             /* width over height of the screen the console was made for; 0 unknown */
    uint32_t audio_frames;             /* stereo frames ready to drain */
    uint32_t audio_rate;               /* their rate in Hz, which travels with them (core ABI 6.7) */
    uint32_t play_audio;               /* 0 when the speed says the sound is not played: drained and dropped */
    uint32_t speed_percent;            /* the speed that applies; 0 unthrottled */
    uint64_t frame_interval_ns;        /* the frame's interval at that speed, for the client's pacer; 0 unthrottled */
    uint64_t total_frames;             /* frames the machine has completed */
    double frame_rate_hz;              /* the machine's own rate */
    uint32_t paused;                   /* run control's pause holds the session */
    uint32_t halted_processor;         /* the processor a halt stopped, an index of `processors`; 0 otherwise */
    uint64_t halted_address;           /* where it stopped */
    uint32_t picture_on_device;        /* since 1.1: the picture was taken on the device the session was lent, and the
                                        * presenter reads it from picture_lend's lend; picture_copy still reads it back */
    uint32_t reserved;                 /* 0 */
    uint32_t picture_in_ring;          /* since 1.4: the picture is in picture_ring's ring, at ring_slot, once its ready
                                        * reaches ring_value; picture_copy still reads it back */
    uint32_t ring_slot;
    uint64_t ring_value;
    uint32_t fields;                   /* since 1.4, with the picture in the ring: 0 a progressive frame, 1 one field,
                                        * 2 two fields woven; 0 otherwise */
    uint32_t field;                    /* with one field: 0 the top, 1 the bottom */
} emusen_crystal_frame_info;

typedef struct emusen_crystal_raw_info { /* 24 bytes, since 1.1; filled by raw_copy */
    uint32_t size;
    uint32_t element;                  /* bytes a value; the encoding is the one raw_select chose */
    int32_t width;
    int32_t height;
    int32_t stride;                    /* bytes a row */
    uint32_t reserved;                 /* 0 */
} emusen_crystal_raw_info;

/* ---- The library ------------------------------------------------------------------------------------------- */

/* EMUSEN_CRYSTAL_VERSION as built. */
uint32_t emusen_crystal_abi_version(void);

/* The implementation's words for itself: which one serves the API, and its build. */
int64_t emusen_crystal_version_text(uint8_t *out, size_t len);

/* The words for the last failing call on the calling thread; rt may be null. A session table's are its own last_error's. */
int64_t emusen_crystal_last_error(emusen_crystal_runtime *rt, uint8_t *out, size_t len);

/* A runtime; null on failure, the words in last_error. A null config is the default. */
emusen_crystal_runtime *emusen_crystal_new(const emusen_crystal_config *config);

/* Closes every session still open, ends every subscription and frees the runtime. */
int32_t emusen_crystal_free(emusen_crystal_runtime *rt);

/* ---- The systems and the engines (reads) ------------------------------------------------------------------- */

/* [{every key of the systems table as emusen-systems writes it, in its order (EmuSen_Backend.md 12.1), then
 * "engines":[name],"starts_empty","core","controllers"}], the table's systems first in its order, then those only an
 * engine's info names, with "id","name","extensions","engines","starts_empty","cover_aspect","look","core",
 * "controllers". Since 2.0 a row holds all 24 of the table's keys, "cheat_space", "media", "byte_order",
 * "libretro_databases", "libretro_systemids" and "firmware_note" among them, which emusen_platform.h's
 * emusen_systems_table answered before. Since 1.1 "core" is {"names":[name],"display_name","console",
 * "extensions":[ext]} or null: the engine's facts riding along, not a pack's, the names `core <name>` knows its core by
 * (section 14.11). Since 1.1 "controllers" is {"buttons":[control],"axes":[axis],"ports","players":[controller
 * id|null]}, the engine's too: the console's pad, its most ports, and each player's controller as the console's stored
 * settings stand at the call (section 14.13). Since 1.13 every row has "look" {"hue","body","palette":[colour],
 * "craters","bands","cap","moons"}, the systems table's or the one computed from the id, and a row only an engine names
 * has the table's default "cover_aspect" (EmuSen_RustPlatform.md section 28.12). */
int64_t emusen_crystal_systems(emusen_crystal_runtime *rt, uint8_t *out, size_t len);

/* reads. [{"id","engine","current"}]: every session the runtime has open, "current" true for the one session 0 names.
 * The runtime's listing, not a session table's sessions, which answers its own session only. */
int64_t emusen_crystal_sessions(emusen_crystal_runtime *rt, uint8_t *out, size_t len);

/* reads, since 1.1. [{"name","label","system","esde_system","esde_full_name","core":{"display_name","console",
 * "extensions":[ext]}}] in release order: the library's shelves, one for each system a console's core runs, named for
 * the core, a file's own in route's "shelf" (section 14.13). */
int64_t emusen_crystal_shelves(emusen_crystal_runtime *rt, uint8_t *out, size_t len);

/* [{"name","id","version","systems":[id],"available","report"}]: every engine discovery found. Since 1.3 "trust" is
 * "manifest", "approved" or "needs_approval", as cores says it. */
int64_t emusen_crystal_engines(emusen_crystal_runtime *rt, uint8_t *out, size_t len);

/* [{"library","sidecar","sha256","abi","development","engine","report"}]: each core library found, unloaded. Since 1.3
 * also "directory" ("installation" or "player"), "trust" ("manifest", "approved", "needs_approval" or "not_read"), and
 * from the sidecar's info "name", "authors":[text], "license" and "systems":[id], what the player's approval shows. The
 * libraries with a sidecar come first, then the player's not yet read, whose "sidecar", "abi", "engine", "name" and
 * "license" are null. "sha256" is the sidecar's in an installation directory and the file's own in the player's
 * (section 29.40.5). Since 1.5 every row has "libretro": null, or for an installed libretro core {"folder","file",
 * "sha256","corename","name","version","license","authors":[text],"source","build","trust"}, "sha256" the core file's
 * own and "trust" its standing, the hash an approval names; "trust" of the row may be "needs_refresh"; and an entry
 * whose core waits for that approval is a last row of its own, its "engine" null (EmuSen_LibretroHost.md 10m). Since 1.8
 * every row has "firmware": "required" where the sidecar's info declares firmware for its systems that one has no open
 * replacement for, "optional" where every one has, or null (EmuSen_Cores.md 3.9). */
int64_t emusen_crystal_cores(emusen_crystal_runtime *rt, uint8_t *out, size_t len);

/* changes. Lists the cores directories again, and reads the player's approvals again; a session already open keeps its
 * engine. */
int32_t emusen_crystal_rescan(emusen_crystal_runtime *rt);

/* changes, since 1.3. Records the player's one-time approval of the core library whose SHA-256 is `sha256` (64 hex
 * digits, either case), one that cores lists by its "sha256", in the approvals' file /etc/EmuSen/cores.approved. A
 * player's library not yet read that has the hash is then read: loaded once under the trust rules and its sidecar
 * written beside it. Lists the cores directories again; a session already open keeps its engine. 0, also for a hash
 * already approved; BAD_ARGUMENT for text that is not a hash; NO_SUCH_ID for a hash no listed library has; IO where the
 * approvals' file or the sidecar could not be written, with the system's words. A library refused when read is not a
 * failure of the call: the approval stands, and cores' "report" says why it was not loaded. */
int32_t emusen_crystal_core_approve(emusen_crystal_runtime *rt, const uint8_t *sha256, size_t sha256_len);

/* changes, since 1.3. Removes every approval of `sha256` from the approvals' file and lists the cores directories again.
 * A library this process already loaded stays loaded, as a library is never unloaded; the next open of it is refused.
 * 0; BAD_ARGUMENT for text that is not a hash; NO_SUCH_ID where nothing approves it; IO where the file could not be
 * written. */
int32_t emusen_crystal_core_revoke(emusen_crystal_runtime *rt, const uint8_t *sha256, size_t sha256_len);

/* {"system","console","engine","reason","cartridge","shelf"}: what opens the file and with which engine, as open would;
 * "system" and "engine" null with the decision's words in "reason" when nothing does. Since 1.1 "shelf" names the shelf
 * the file sits on, one that shelves lists, null where none. "cartridge" is what the system's pack reads of the file's
 * header, {"board","header_trust","prg_bytes","chr_bytes","battery"} for the NES, null where its pack reads
 * none. Since 1.6 "waiting_default" is null, or {"stem","sha256"} where the system's default libretro core is installed,
 * waits for the player's approval and takes the file, and neither the game's nor the console's settings chose an
 * engine: "engine" is then only the fallback, and "sha256" is the hash an approval names (EmuSen_LibretroHost.md 10t).
 * Since 1.7 "default_held" is null, or {"stem","sha256","reason"} where that default's core is approved but the entry
 * could not be read, its copies refreshed or its proxy loaded: "engine" is then the fallback, and "reason" says why
 * (EmuSen_LibretroHost.md 10v).
 * Reads the file; runs no core. */
int64_t emusen_crystal_route(emusen_crystal_runtime *rt, const uint8_t *path, size_t path_len, uint8_t *out, size_t len);

/* ---- A session's life ------------------------------------------------------------------------------------- */

/* changes. Opens a game, or starts a console empty, and returns the session's id: NO_ENGINE with the routing
 * decision's words when nothing opens it, FIRMWARE when required firmware is missing, FAULT with the engine's words
 * when it refuses the game, and since 1.3 NOT_APPROVED when the engine's library awaits the player's approval
 * (core_approve). Any thread; the session has no frame thread until one calls frame. */
int64_t emusen_crystal_open(emusen_crystal_runtime *rt, const emusen_crystal_open_params *params);

/* changes. Writes the battery save, lets go of the save media and ends the session. */
int32_t emusen_crystal_close(emusen_crystal_runtime *rt, uint32_t session);

/* changes. The console's reset, as its button; the game stays loaded. */
int32_t emusen_crystal_reset(emusen_crystal_runtime *rt, uint32_t session);

/* reads. {"id","engine","system","console","game":{"path","name"}|null,"run":"running"|"paused"|"halted",
 * "halted":{"processor","address"}|null,"ports":[{"port","controller"}],"buttons":[control],"axes":[axis],
 * "discs":{"count","index","can_change","lid_open"},"picture":{"width","height","row_repeat"},"frame_rate_hz","total_frames","state_version",
 * "rewind_capture","rewind_withheld","notices":{"engine","firmware","media"}}: the session as a frontend shows it. Since 1.1
 * also "state":{"format","version","loads_from":[version]} and "television_standard", "ntsc" or "pal" as machine info
 * gives the machine's signal, null where it gives none (section 14.13). */
int64_t emusen_crystal_session_info(emusen_crystal_runtime *rt, uint32_t session, uint8_t *out, size_t len);

/* ---- Discs ------------------------------------------------------------------------------------------------ */

/* changes. A disc by path into the drive, the lid opened and shut around it; NOT_SUPPORTED with no drive. */
int32_t emusen_crystal_disc_insert(emusen_crystal_runtime *rt, uint32_t session, const uint8_t *path, size_t path_len);

/* changes. The drive left empty. */
int32_t emusen_crystal_disc_eject(emusen_crystal_runtime *rt, uint32_t session);

/* changes. The game's disc of that index (session_info's discs) into the drive. */
int32_t emusen_crystal_disc_change(emusen_crystal_runtime *rt, uint32_t session, uint32_t index);

/* changes. Opens the lid (1) or shuts it (0). */
int32_t emusen_crystal_disc_lid(emusen_crystal_runtime *rt, uint32_t session, uint32_t open);

/* ---- The frame -------------------------------------------------------------------------------------------- */

/* changes; the frame thread's. Applies the input held, advances the machine one frame unless paused or halted, and
 * runs the frame-end order once: the debugger's snapshots, rewind's capture where the engine allows it, the
 * recorder, the core's log as events. Fills out, which may be null. A halt is a frame that did not complete:
 * ran 0, halted 1, and nothing advances until run control resumes it. */
int32_t emusen_crystal_frame(emusen_crystal_runtime *rt, uint32_t session, emusen_crystal_frame_info *out);

/* changes, since 1.4. One frame advance, for every engine: the session is paused, and the next frame runs one whole
 * frame and leaves it paused, with a run event {"op":"stepped"} (EmuSen_Backend.md section 33.3). */
int32_t emusen_crystal_frame_advance(emusen_crystal_runtime *rt, uint32_t session);

/* reads. The picture as RGBA8888, width * 4 bytes a row: the frame's width and height. */
int64_t emusen_crystal_picture_copy(emusen_crystal_runtime *rt, uint32_t session, uint8_t *out, size_t len);

/* reads, since 1.1. The current machine's lend, the platform library's emusen_serenity_lend, for the presenter to
 * borrow and not free; null when nothing is lent. A machine made again, by a reset or a state of another size, is
 * lent afresh, so it is asked again after a frame. */
const void *emusen_crystal_picture_lend(emusen_crystal_runtime *rt, uint32_t session);

/* reads, since 1.4. The current machine's ring, the platform library's emusen_serenity_ring, for the presenter to borrow,
 * or null where none is lent (no OPEN_RING, no device, a core without DEVICE_RING, or the C# implementation). A machine
 * made again, or one whose maxima grew, holds a new one, so it is asked again after a frame; the last one stays valid
 * until the next change. */
const void *emusen_crystal_picture_ring(emusen_crystal_runtime *rt, uint32_t session);

/* changes, since 1.1. The encoding of the picture before the core's palette that the session keeps from the next
 * frame, one the machine's raw_encodings names; absent keeps none. */
int32_t emusen_crystal_raw_select(emusen_crystal_runtime *rt, uint32_t session, const uint8_t *encoding, size_t len);

/* reads, since 1.1. The last frame's picture before its palette, in the encoding raw_select chose, its shape into
 * info; returns the bytes. ABSENT when none is kept. */
int64_t emusen_crystal_raw_copy(emusen_crystal_runtime *rt, uint32_t session, emusen_crystal_raw_info *info,
                                uint8_t *out, size_t len);

/* changes. Up to frames stereo frames of interleaved int16 into out, their rate into rate when it is not null; returns
 * the frames written. A null out asks how many are ready and drains none. */
int64_t emusen_crystal_audio_drain(emusen_crystal_runtime *rt, uint32_t session, int16_t *out, size_t frames, uint32_t *rate);

/* The frame thread's, while the client holds the session paused: returns 1 when a change was applied or run control
 * asked to run, 0 when timeout_ms passed first. A blocking wait, not a callback. */
int32_t emusen_crystal_wait(emusen_crystal_runtime *rt, uint32_t session, uint32_t timeout_ms);

/* reads. [name]: the phases the core times its frames in, "parent/child" for one inside another. */
int64_t emusen_crystal_phase_names(emusen_crystal_runtime *rt, uint32_t session, uint8_t *out, size_t len);

/* reads. The last frame's phase times in milliseconds, phase_names' order, at most count; returns how many there
 * are. A null out asks the count. */
int64_t emusen_crystal_phases(emusen_crystal_runtime *rt, uint32_t session, double *out, size_t count);

/* ---- Input (changes; held, and sent to the machine as the next frame starts) -------------------------------- */

/* A button of the core ABI's canonical controls, emusen_control, pressed (1) or released (0) on a port from 0. */
int32_t emusen_crystal_input_button(emusen_crystal_runtime *rt, uint32_t session, uint32_t port, uint32_t control, uint32_t pressed);

/* An axis of emusen_axis: sticks -1..1, right and down positive; triggers 0..1. */
int32_t emusen_crystal_input_axis(emusen_crystal_runtime *rt, uint32_t session, uint32_t port, uint32_t axis, double value);

/* A controller plugged into a port (1) or pulled out (0). */
int32_t emusen_crystal_input_connected(emusen_crystal_runtime *rt, uint32_t session, uint32_t port, uint32_t connected);

/* reads, since 1.10, from any thread. A port's motors as the core's OUTPUT events last set them, the strong (low-frequency)
 * and the weak, 0 to 65535 each, for the pad of the player the port hears; both zero for a port past 8 or with no motor,
 * and wherever the console's games drive none (the systems table's "rumble") or AppSettings.Rumble was off as the
 * session opened. A frontend hands them to the pad each frame it runs (EmuSen_Input.md 9). */
int32_t emusen_crystal_input_rumble(emusen_crystal_runtime *rt, uint32_t session, uint32_t port, uint16_t *strong, uint16_t *weak);

/* since 1.11, from any thread. The player's rumble switch turned while the session runs: 0 off, else on. The systems
 * table's half of the gate is unchanged, so on never opens it for a console whose games drive no motor (EmuSen_Input.md 9.7). */
int32_t emusen_crystal_input_rumble_switch(emusen_crystal_runtime *rt, uint32_t session, uint32_t on);

/* ---- Speed (changes) ------------------------------------------------------------------------------------- */

/* The three speeds in percent, the normal one 0 for unthrottled, turbo from 100, slow 1..100; EMUSEN_CRYSTAL_SPEED_*
 * flags. Rendering and sound follow from the speed that applies; the pacing is the client's. */
int32_t emusen_crystal_speed_set(emusen_crystal_runtime *rt, uint32_t session, uint32_t percent, uint32_t turbo_percent,
                                 uint32_t slow_percent, uint32_t flags);

/* Which speed applies: EMUSEN_CRYSTAL_SPEED_NORMAL, _TURBO or _SLOW. */
int32_t emusen_crystal_speed_mode(emusen_crystal_runtime *rt, uint32_t session, uint32_t mode);

/* ---- Rewind --------------------------------------------------------------------------------------------- */

/* changes. On (1) or off (0); a snapshot every interval_frames and at most budget_bytes of history, 0 keeping each as
 * it was; pictures thumbnail_width wide kept with the snapshots, 0 for none. Turning it on takes the first snapshot;
 * turning it off clears the history. */
int32_t emusen_crystal_rewind_set(emusen_crystal_runtime *rt, uint32_t session, uint32_t enabled, uint32_t interval_frames,
                                  uint64_t budget_bytes, uint32_t thumbnail_width);

/* changes; the frame thread's, in place of frame. Steps back up to count snapshots and returns how many it took, 0 when
 * the history is spent; fills out, which may be null, for the picture now on show. */
int64_t emusen_crystal_rewind_step(emusen_crystal_runtime *rt, uint32_t session, uint32_t count, emusen_crystal_frame_info *out);

/* reads. {"enabled","interval_frames","budget_bytes","depth","buffered_bytes","snapshot_bytes","seconds",
 * "moments":[{"frame","core_frames","picture":{"width","height"}|null}]}, the moments oldest first. */
int64_t emusen_crystal_rewind_reel(emusen_crystal_runtime *rt, uint32_t session, uint8_t *out, size_t len);

/* reads. The picture of the moment at frame, as tile_sheet gives one: width and height as two little-endian uint32,
 * then RGBA8888 rows; ABSENT where the moment has none. */
int64_t emusen_crystal_rewind_picture(emusen_crystal_runtime *rt, uint32_t session, uint64_t frame, uint8_t *out, size_t len);

/* changes. Back to the moment at frame, the history after it dropped. */
int32_t emusen_crystal_rewind_to(emusen_crystal_runtime *rt, uint32_t session, uint64_t frame);

/* ---- States (state_save, state_load and state_default_path by path are the session table's) ------------------ */

/* reads. [{"slot","path","exists","saved_at"}] for slots 1 to 8 and the resume state, slot 0, in the player's state
 * folder. "saved_at" is the file's time as ISO 8601, null where none exists. */
int64_t emusen_crystal_state_slots(emusen_crystal_runtime *rt, uint32_t session, uint8_t *out, size_t len);

/* changes. A slot from 1 to 8, with the picture on show beside it. */
int32_t emusen_crystal_state_slot_save(emusen_crystal_runtime *rt, uint32_t session, uint32_t slot);

/* changes. A slot's state loaded: the rewind history cleared, a STATE event raised. ABSENT where none exists. */
int32_t emusen_crystal_state_slot_load(emusen_crystal_runtime *rt, uint32_t session, uint32_t slot);

/* changes. The resume state, offered at the game's next start, with its picture. */
int32_t emusen_crystal_resume_save(emusen_crystal_runtime *rt, uint32_t session);

/* reads, since 1.1. The record the runtime keeps of the state at path in states.db, while the file is the one it was
 * written for: {"console","core","state_version","build","saved_at","rom_file","rom_md5","rom_bytes"}; ABSENT for
 * none. A state_save writes it; state_load refuses with STATE_REFUSED what it says another console wrote. */
int64_t emusen_crystal_state_record(emusen_crystal_runtime *rt, const uint8_t *path, size_t path_len, uint8_t *out,
                                    size_t len);

/* changes, since 1.1. The record of the state at path forgotten, as when the player deletes the state. */
int32_t emusen_crystal_state_forget(emusen_crystal_runtime *rt, const uint8_t *path, size_t path_len);

/* changes. As state_slot_load, for the resume state. */
int32_t emusen_crystal_resume_load(emusen_crystal_runtime *rt, uint32_t session);

/* ---- Settings ------------------------------------------------------------------------------------------- */

/* reads. [{"key","label","hint","kind":"switch"|"count"|"choice","default","min","max","choices","choice_labels",
 * "group"}]: the console's settings, the engine's choice among them as key "Engine", for the named engine or the one
 * the console runs; an absent console is the session's. Since 1.1 a row whose core words the value chosen has "note",
 * that sentence as the console's stored values stand or null when it says nothing, and every row has "shown_with",
 * {"key","value"} where the row means something only while another key holds that value, else null (section 14.13). */
int64_t emusen_crystal_settings_schema(emusen_crystal_runtime *rt, uint32_t session, const uint8_t *console,
                                       size_t console_len, const uint8_t *engine, size_t engine_len, uint8_t *out, size_t len);

/* reads. The value as the scope sees it, game over console over the default, in its text form; ABSENT for a key with
 * no value and no default. With session 0 the console is named and GAME is refused; with a session it is absent. */
int64_t emusen_crystal_settings_get(emusen_crystal_runtime *rt, uint32_t session, uint32_t scope, const uint8_t *console,
                                    size_t console_len, const uint8_t *key, size_t key_len, uint8_t *out, size_t len);

/* changes. Stores the value at the scope and applies it to every open session it reaches, between frames; an absent
 * value forgets the scope's own, so a lower one shows through. BAD_ARGUMENT for a value the schema refuses. */
int32_t emusen_crystal_settings_set(emusen_crystal_runtime *rt, uint32_t session, uint32_t scope, const uint8_t *console,
                                    size_t console_len, const uint8_t *key, size_t key_len, const uint8_t *value, size_t value_len);

/* reads, since 1.1. A game's own value of the key, from its file of settings, with no session: the library's view of
 * a game; ABSENT where it holds none. For the engine the key is "Engine". */
int64_t emusen_crystal_game_setting_get(emusen_crystal_runtime *rt, const uint8_t *path, size_t path_len,
                                        const uint8_t *key, size_t key_len, uint8_t *out, size_t len);

/* changes, since 1.1. A game's own value set, or forgotten with an absent value, with no session; checked against
 * the console's schema, and applied to an open session of the game between frames. */
int32_t emusen_crystal_game_setting_set(emusen_crystal_runtime *rt, const uint8_t *path, size_t path_len,
                                        const uint8_t *key, size_t key_len, const uint8_t *value, size_t value_len);

/* ---- Firmware ------------------------------------------------------------------------------------------- */

/* reads. [{"core","chip","file","size","purpose","alternate_names","parts","required","replacement":{"effect",
 * "cost"}|null}]: the required firmware the game needs on the engine and the library does not hold; an absent engine
 * is the settings' choice. Empty for a file nothing opens; open then says why. */
int64_t emusen_crystal_firmware_needed(emusen_crystal_runtime *rt, const uint8_t *path, size_t path_len, const uint8_t *engine,
                                       size_t engine_len, uint8_t *out, size_t len);

/* changes. Copies the file at source into EmuSen's firmware folder under the request's name, request being one
 * entry of firmware_needed; BAD_ARGUMENT when its size is not the request's, IO when the copy fails. */
int32_t emusen_crystal_firmware_install(emusen_crystal_runtime *rt, const uint8_t *request, size_t request_len,
                                        const uint8_t *source, size_t source_len);

/* reads. {"folder","folders":[path],"systems":[{"name","engine","items":[{"label","file","size","present",
 * "wrong_size","effect","cost","found_in","wrong_size_in"}]}]}: each discovered system's firmware. */
int64_t emusen_crystal_firmware_overview(emusen_crystal_runtime *rt, uint8_t *out, size_t len);

/* reads, since 1.9. {"system","firmware":[{"label","file","required","status","need"}]}: each firmware file the system's engines
 * declare, once, the native engines' where one lists the system, else the hosted cores'; "required" where one declares
 * it required or it has no open replacement; "status" "present" where the firmware folders hold it by name or content
 * as firmware_overview finds it, else "missing" where it is required and "not_needed" where it is not. An empty list
 * for a system that needs none; ABSENT for a system neither the table nor an engine names (EmuSen_Cores.md 3.11).
 * Since 1.11 "need" names the need the file meets, the declaration's need or "file:" and the file's name where it states none:
 * files that are alternatives, as a console's regional BIOSes, share it (EmuSen_Cores.md 3.13). */
int64_t emusen_crystal_firmware_status(emusen_crystal_runtime *rt, const uint8_t *system, size_t system_len, uint8_t *out,
                                       size_t len);

/* ---- Save media ------------------------------------------------------------------------------------------ */

/* reads. {"notice","media":[{"medium","title","noun","extension","kind":"removable"|"console","mode":"game"|"shared"|
 * "named","slots":[{"label","name","path"}]}]}: the session's save media, each slot's file as the medium's mode places it,
 * and why a slot runs on the game's own file this load where it does. */
int64_t emusen_crystal_save_media(emusen_crystal_runtime *rt, uint32_t session, uint8_t *out, size_t len);

/* The card manager over a console's cards, which needs no session (EmuSen_SaveMedia.md section 6). args and results
 * are JSON; a change answers {"done","words"}, its refusal an answer with done false. EMUSEN_CRYSTAL_CARDS_*. */
int64_t emusen_crystal_cards_op(emusen_crystal_runtime *rt, uint32_t op, const uint8_t *args, size_t args_len, uint8_t *out, size_t len);

#define EMUSEN_CRYSTAL_CARDS_LIST      0u /* reads. {"console"} -> [{"medium","title","noun","extension","cards":[{"path","name","label","games_own","held_by"}]}];
                                          * since 1.1 also "kind" ("removable"|"console"), "capacity" and "unit" (null where no format module
                                          * reads the medium), "mode" ("game"|"shared"|"named") and its settings key "mode_key", "slots":[{"label",
                                          * "name","path"}] as the mode places them, "next_name" (the first name no card has) and "import_due"
                                          * (the games' own saves wait for the shared card's first import) - section 14.13 */
#define EMUSEN_CRYSTAL_CARDS_OPEN      1u /* reads. {"path"} -> {"medium","exists","length","formatted","free","problem","saves":[{"id","title","units","game","damaged"}]}; ABSENT for a file of no medium */
#define EMUSEN_CRYSTAL_CARDS_COPY      2u /* changes. {"from","save","to"}: a save by its id or its place from 1 */
#define EMUSEN_CRYSTAL_CARDS_MOVE      3u /* changes. {"from","save","to"} */
#define EMUSEN_CRYSTAL_CARDS_DELETE    4u /* changes. {"card","save"} */
#define EMUSEN_CRYSTAL_CARDS_IMPORT    5u /* changes. {"to","from":[path]}: every save of the cards onto one, all or none */
#define EMUSEN_CRYSTAL_CARDS_NEW       6u /* changes. {"path"}: a formatted card where the medium's module can make one */
#define EMUSEN_CRYSTAL_CARDS_DUPLICATE 7u /* changes. {"from","to"}: a whole card to a new name */
#define EMUSEN_CRYSTAL_CARDS_RENAME    8u /* changes. {"from","to"} */

/* ---- Cheat codes (reads, since 2.0) ----------------------------------------------------------------------- */

/* reads. {"codecs":[{"id","name","kind","space"}],"systems":[{"id","auto","explicit"}]}: every cheat codec, "kind"
 * "RamPoke" or "RomPatch", "space" a memory space or null, and each system's pair, a codec's id or null; the
 * catalogue, where a session table's cheat_codecs answers that session's pair. */
int64_t emusen_crystal_cheat_codec_list(emusen_crystal_runtime *rt, uint8_t *out, size_t len);

/* reads. A code's answers from one codec: {"can","decode":{"address","value"},"compare","writes"}, "compare" a byte or
 * null, "writes" a list of {"space","address","value","width","type","bit","big_endian","repeat","repeat_address",
 * "repeat_value"} or null, and each of decode, compare and writes {"format"} or {"range"} with the codec's words where
 * it refused. ABSENT for a codec there is not. */
int64_t emusen_crystal_cheat_decode(emusen_crystal_runtime *rt, const uint8_t *codec, size_t codec_len, const uint8_t *code,
                                    size_t code_len, uint8_t *out, size_t len);

/* reads. The Game Genie code that patches value at address, for snes.gg and md.gg; ABSENT for a codec with no encoder. */
int64_t emusen_crystal_cheat_encode(emusen_crystal_runtime *rt, const uint8_t *codec, size_t codec_len, int32_t address,
                                    uint32_t value, uint8_t *out, size_t len);

/* ---- Events --------------------------------------------------------------------------------------------- */

/* A subscription to a session's events (0 for every session's) of the given kinds; null on failure. Each has its own
 * bounded queue; a full queue drops its oldest event and says so in the next drain. */
emusen_crystal_subscription *emusen_crystal_subscribe(emusen_crystal_runtime *rt, uint32_t session, uint32_t kinds);

/* reads. [event], oldest first, removed once a buffer has taken them all; {"kind":"dropped","count"} first when the
 * queue overflowed. */
int64_t emusen_crystal_subscription_drain(emusen_crystal_subscription *sub, uint8_t *out, size_t len);

/* 1 when events are waiting, 0 when timeout_ms passed first. A blocking wait, not a callback. */
int32_t emusen_crystal_subscription_wait(emusen_crystal_subscription *sub, uint32_t timeout_ms);

/* Ends a subscription. */
int32_t emusen_crystal_unsubscribe(emusen_crystal_subscription *sub);

/* ---- The recorder ---------------------------------------------------------------------------------------- */

/* changes. Records every stride-th frame drawn as a picture into a new folder under base, which it returns by the
 * copy-out rule, the frames drawn whatever the speed until record_stop. */
int64_t emusen_crystal_record_start(emusen_crystal_runtime *rt, uint32_t session, const uint8_t *base, size_t base_len,
                                    uint32_t stride, uint8_t *out, size_t len);

/* changes. Ends the recording and encodes its video where an encoder is installed; returns the folder. */
int64_t emusen_crystal_record_stop(emusen_crystal_runtime *rt, uint32_t session, uint8_t *out, size_t len);

/* ---- The debugger: the session service ---------------------------------------------------------------------- */

/* The session's emusen.session@1 table, for a host to hand to dianash_set_services and for a frontend's own calls of
 * the debugger: the one surface of the session service, its words read with the table's last_error. It lives as long
 * as the runtime; once its session is closed every call is EMUSEN_SVC_NO_SESSION.
 * Since 1.12, with EMUSEN_CRYSTAL_SERVICE_NO_GAME, session is 0 and the table is the runtime's with no game, living as
 * long as the runtime: the cheat ops that need no game (the database, a .cht file's parse, the cheat files) are
 * answered, run control's resume is raised as a run event for events_drain, since 2.0 log_switch sets the process's
 * switch (refused where the table is read-only) and log_status reads it, sessions is an empty list, games is
 * EMUSEN_SVC_NOT_SUPPORTED, and every other call is EMUSEN_SVC_NO_SESSION with the words "No ROM loaded - this
 * command needs an active debug target." (EmuSen_Backend.md section 40.7). */
const emusen_session_v1 *emusen_crystal_session_service(emusen_crystal_runtime *rt, uint32_t session, uint32_t flags);

#ifdef __cplusplus
}
#endif

#endif /* EMUSEN_CRYSTAL_H */
