/* emusen_core.h - the EmuSen core ABI, version 1.0.
 *
 * A core is a shared library exporting the functions below, C calling convention. The normative text is
 * EmuSen_CoreAPI.md in the EmuSen manual; this header is its machine-checked half, and where the two disagree the
 * page wins and the header is in error. Its facts are recorded in abi/v1/baseline.txt beside it, and CI fails a
 * change that removes or alters one (EmuSen_CoreAPI.md section 5).
 *
 * Everything here is @since 1.0 unless marked otherwise. Names: exports emusen_core_*, types emusen_*,
 * constants EMUSEN_*.
 *
 * Rules every export follows (section 6.0):
 *  - The library never calls the host. The host calls; the library returns.
 *  - A negative int32_t/int64_t return is a status (section 6.15); zero or more is success. A null machine
 *    always gives EMUSEN_NULL. Only abi_version and capabilities return unsigned values, and cannot fail.
 *  - Copy out: a call that fills a buffer takes (out, len) and returns the whole length; a null `out` asks
 *    only the length. No pointer into the core's memory is ever returned; the host owns every buffer, and a
 *    pointer passed in is valid for that call only.
 *  - A struct crossing the boundary begins with `size`, the bytes its allocator gave it. The callee reads and
 *    writes only the prefix both sides know, never `size` itself; a struct shorter than its 1.0 size is
 *    refused with EMUSEN_BAD_STRUCT. Fields are only appended, and a field's zero means "absent".
 *  - Enumerations are open: an unknown value is refused with EMUSEN_NOT_SUPPORTED, or ignored where this header
 *    says so; never undefined behaviour. A host skips event kinds and descriptor fields it does not know.
 *  - One thread at a time per machine; library-level calls are safe from any thread at any time.
 *  - Errors are values: no export unwinds, longjmps, raises a signal or calls exit.
 */
#ifndef EMUSEN_CORE_H
#define EMUSEN_CORE_H

#include <stddef.h>
#include <stdint.h>

#ifdef __cplusplus
extern "C" {
#endif

#define EMUSEN_CORE_ABI_MAJOR 1u
#define EMUSEN_CORE_ABI_MINOR 0u
#define EMUSEN_CORE_ABI_VERSION ((EMUSEN_CORE_ABI_MAJOR << 16) | EMUSEN_CORE_ABI_MINOR)

/* An opaque machine. Several may exist at once, from one library. */
typedef struct emusen_machine emusen_machine;

/* ---- Status codes (section 6.15). The bands are fixed for the life of major 1. ---------------- */
#define EMUSEN_NULL               (-1)   /* a null machine or required buffer */
#define EMUSEN_TRUNCATED          (-2)   /* a state ended early */
#define EMUSEN_FOREIGN            (-3)   /* not this core's state */
#define EMUSEN_STATE_VERSION      (-4)   /* a state version this core does not read */
#define EMUSEN_BAD_STRING         (-5)   /* a string length the state format refuses */
#define EMUSEN_BUFFER_TOO_SMALL   (-7)   /* the buffer is shorter than what must be written whole */
/* -6 and -8 are reserved. -9 .. -255 are the core's own; emusen_core_status_text names them. */
#define EMUSEN_CORE_BAND_FIRST    (-9)
#define EMUSEN_CORE_BAND_LAST     (-255)
#define EMUSEN_NOT_SUPPORTED      (-256) /* a capability not claimed, or an enumeration value not known */
#define EMUSEN_NO_SUCH_SPACE      (-257)
#define EMUSEN_READ_ONLY          (-258)
#define EMUSEN_UNKNOWN_SETTING    (-259) /* a key the schema does not have */
#define EMUSEN_BAD_SETTING        (-260) /* a value outside its domain, a malformed line, or a create key sent later */
#define EMUSEN_NO_SUCH_PORT       (-261)
#define EMUSEN_BAD_FILE           (-262)
#define EMUSEN_BAD_STRUCT         (-263) /* a struct or element size below its 1.0 size */
#define EMUSEN_BAD_IMAGE          (-264) /* the image refused, for a core without a finer code */
/* -265 .. -319 are the interface's, unassigned. -320 .. -383: a .NET exception reproduced by a port, -320 less
 * its kind (ports only). Below -383, reserved. */
#define EMUSEN_FAULT_BASE         (-320)

/* ---- Capability bits (section 6.1). Open: a host ignores bits it does not know. ------------ */
#define EMUSEN_CAP_RESET             (1ull << 0)   /* emusen_core_reset */
#define EMUSEN_CAP_PRESENT           (1ull << 1)   /* emusen_core_present, which the host must call (section 4.6) */
#define EMUSEN_CAP_SNAPSHOT          (1ull << 2)   /* state kind 1 */
#define EMUSEN_CAP_AXES              (1ull << 3)   /* emusen_core_set_axis */
#define EMUSEN_CAP_AUDIO_PEEK        (1ull << 4)   /* emusen_core_audio_peek */
#define EMUSEN_CAP_MUTES             (1ull << 5)   /* emusen_core_set_mutes */
#define EMUSEN_CAP_SETTINGS          (1ull << 6)   /* emusen_core_set_settings */
#define EMUSEN_CAP_PHASES            (1ull << 7)   /* emusen_core_phases */
#define EMUSEN_CAP_FRAME_SERIAL      (1ull << 8)   /* frame_info.serial moves only when the picture does */
#define EMUSEN_CAP_ROW_REPEAT        (1ull << 9)   /* frame_info.row_repeat may exceed 1 */
#define EMUSEN_CAP_BATTERY_DIRTY     (1ull << 10)  /* emusen_core_battery's flags track changes */
#define EMUSEN_CAP_ROM_PATCHES       (1ull << 11)  /* emusen_core_set_rom_patches */
#define EMUSEN_CAP_DEBUG             (1ull << 12)  /* the ten debug exports */
#define EMUSEN_CAP_DEBUG_STACK       (1ull << 13)  /* emusen_core_debug_set_stack */
#define EMUSEN_CAP_CHEAT_POKES       (1ull << 14)  /* emusen_core_set_cheat_pokes */
#define EMUSEN_CAP_SETTING_NOTES     (1ull << 15)  /* emusen_core_setting_notes */
#define EMUSEN_CAP_DEBUG_REGISTERS   (1ull << 16)  /* emusen_core_debug_registers */
#define EMUSEN_CAP_DEBUG_DISASSEMBLE (1ull << 17)  /* emusen_core_debug_disassemble */
/* Bits 18..47 are for later minors; 48..63 are never assigned, for a core's private use in tests. */

/* ---- Flags and small constants ------------------------------------------------------------------ */
#define EMUSEN_OPTION_SKIP_RENDERING 1u  /* set_options bit 0; every other bit is reserved and ignored */
#define EMUSEN_FRAME_WALKED          1u  /* frame_info.flags bit 0: the last present walked the picture */
#define EMUSEN_BATTERY_CHANGED       1u  /* battery flags bit 0: changed since battery_saved */
#define EMUSEN_BATTERY_TRACKED       2u  /* battery flags bit 1: the core tracks changes (BATTERY_DIRTY) */
#define EMUSEN_FILE_BATTERY          0u  /* emusen_file.which of the battery save */
#define EMUSEN_NO_COMPARE            0xFFFFFFFFu /* a patch or poke with no compare value */

/* Debug: the flags debug_set takes; coverage of processor n is bit (EMUSEN_DEBUG_COVERAGE + n), n < 24. */
#define EMUSEN_DEBUG_CALLS       1u
#define EMUSEN_DEBUG_WRITES      2u
#define EMUSEN_DEBUG_INTERRUPTS  4u
#define EMUSEN_DEBUG_EACH        8u
#define EMUSEN_DEBUG_PROFILING   16u
#define EMUSEN_DEBUG_COVERAGE    8u
/* Debug: why debug_run_frame stopped, as bits; zero is the frame's end. */
#define EMUSEN_STOP_BREAKPOINT   1u
#define EMUSEN_STOP_EACH         2u
#define EMUSEN_STOP_DEPTH        4u
#define EMUSEN_STOP_DATA         8u
#define EMUSEN_STOP_INTERRUPT    16u
#define EMUSEN_STOP_RING         32u  /* a log is full and must be drained */
/* Debug: the flags debug_run_frame takes. */
#define EMUSEN_RUN_UNCHECKED     1u   /* the first instruction is not checked against the tables */
#define EMUSEN_RUN_CONTINUE      2u   /* the frame a stop left open, its budget kept */
/* Debug: the kind word of a call-log entry. */
#define EMUSEN_CALL_RETURN       0u
#define EMUSEN_CALL_CALL         1u
#define EMUSEN_CALL_IRQ          2u
#define EMUSEN_CALL_NMI          3u
#define EMUSEN_CALL_BRK          4u
#define EMUSEN_CALL_COP          5u

/* ---- Open enumerations ----------------------------------------------------------------- */
enum emusen_pixel_format {          /* section 6.6 */
    EMUSEN_PIXEL_RGBA8888 = 0,      /* bytes R, G, B, A; every host accepts it */
    EMUSEN_PIXEL_BGRA8888 = 1,      /* bytes B, G, R, A */
    EMUSEN_PIXEL_RGB565   = 2       /* little-endian 16-bit words */
};

enum emusen_state_kind {            /* section 6.9 */
    EMUSEN_STATE_FULL     = 0,      /* the save state; written after the core's threads settle */
    EMUSEN_STATE_SNAPSHOT = 1       /* SNAPSHOT: for rewind and run-ahead */
};

enum emusen_event_kind {            /* section 6.19; a host skips kinds it does not know */
    EMUSEN_EVENT_AUDIO_RATE   = 1,  /* a: the new rate in Hz */
    EMUSEN_EVENT_GEOMETRY     = 2,  /* a: width, b: height of the picture now produced */
    EMUSEN_EVENT_STATE_SIZE   = 3,  /* a: the new size of state kind b */
    EMUSEN_EVENT_BATTERY      = 4,  /* a: the battery file that changed */
    EMUSEN_EVENT_LOG          = 5,  /* a: records waiting in emusen_core_log_drain */
    EMUSEN_EVENT_MACHINE_INFO = 6   /* the machine descriptor changed, or events were lost; read everything again */
};

/* The canonical controls a core maps its pad bits onto; append-only, PadButton's order (section 6.3). */
enum emusen_control {
    EMUSEN_CONTROL_B = 0, EMUSEN_CONTROL_Y = 1, EMUSEN_CONTROL_SELECT = 2, EMUSEN_CONTROL_START = 3,
    EMUSEN_CONTROL_UP = 4, EMUSEN_CONTROL_DOWN = 5, EMUSEN_CONTROL_LEFT = 6, EMUSEN_CONTROL_RIGHT = 7,
    EMUSEN_CONTROL_A = 8, EMUSEN_CONTROL_X = 9, EMUSEN_CONTROL_L = 10, EMUSEN_CONTROL_R = 11,
    EMUSEN_CONTROL_L2 = 12, EMUSEN_CONTROL_R2 = 13, EMUSEN_CONTROL_L3 = 14, EMUSEN_CONTROL_R3 = 15
};

/* The canonical axes, PadAxis's order: sticks -1..1 with right and down positive, triggers 0..1 (section 6.8). */
enum emusen_axis {
    EMUSEN_AXIS_LEFT_X = 0, EMUSEN_AXIS_LEFT_Y = 1, EMUSEN_AXIS_RIGHT_X = 2, EMUSEN_AXIS_RIGHT_Y = 3,
    EMUSEN_AXIS_LEFT_TRIGGER = 4, EMUSEN_AXIS_RIGHT_TRIGGER = 5
};

/* ---- Structures ------------------------------------------------------------------------ */
typedef struct emusen_file {        /* a file the host read for this game (section 6.2); 24 bytes */
    uint32_t size;                  /* unused: an array's element size is create_params.file_size */
    uint32_t which;                 /* 0 the battery save; others from machine info or firmware_for */
    const uint8_t *data;
    size_t len;
} emusen_file;

typedef struct emusen_create_params { /* 88 bytes */
    uint32_t size;
    uint32_t host_abi_version;      /* EMUSEN_CORE_ABI_VERSION the host was built with */
    const uint8_t *image;           /* the game file's bytes; the core parses it */
    size_t image_len;
    const uint8_t *settings;        /* UTF-8 "key=value" lines, create-time and run-time keys together */
    size_t settings_len;
    const emusen_file *files;
    size_t file_count;
    size_t file_size;               /* sizeof(emusen_file) as the host knows it */
    uint64_t pixel_formats;         /* bit n: emusen_pixel_format n accepted; bit 0 is always accepted */
    uint8_t *error;                 /* optional: a refusal's UTF-8 detail, NUL-terminated within error_len */
    size_t error_len;
} emusen_create_params;

typedef struct emusen_frame_info {  /* 56 bytes */
    uint32_t size;
    uint32_t format;                /* enum emusen_pixel_format, one the host accepted */
    int32_t width;
    int32_t height;
    int32_t stride;                 /* bytes per row in frame_copy's output */
    int32_t row_repeat;             /* each row is shown this many times; 1 without ROW_REPEAT */
    uint32_t flags;                 /* EMUSEN_FRAME_WALKED */
    uint32_t aspect_num;            /* display aspect of this picture; 0/0 for square pixels */
    uint32_t aspect_den;
    uint32_t reserved;
    int64_t serial;                 /* moves only when the picture does with FRAME_SERIAL; the frame count without */
    int64_t bytes;                  /* frame_copy's whole length */
} emusen_frame_info;

typedef struct emusen_event {       /* 24 bytes */
    uint32_t size;
    uint32_t kind;                  /* enum emusen_event_kind */
    int64_t a;
    int64_t b;
} emusen_event;

/* ---- Library-level calls: no machine, any thread, before and after any machine ----------------- */
uint32_t emusen_core_abi_version(void);                         /* (major << 16) | minor */
uint64_t emusen_core_capabilities(void);                        /* EMUSEN_CAP_* claimed */
int64_t  emusen_core_info(uint8_t *out, size_t len);            /* core info, JSON (section 6.3) */
int64_t  emusen_core_settings_schema(uint8_t *out, size_t len); /* the settings schema, a JSON array (section 6.13) */
/* What an image needs, from its bytes alone and without a machine: a JSON array of firmware entries. */
int64_t  emusen_core_firmware_for(const uint8_t *image, size_t image_len, uint8_t *out, size_t len);
/* Words for a status, UTF-8; EMUSEN_NOT_SUPPORTED for a code the core never returns. */
int64_t  emusen_core_status_text(int32_t status, uint8_t *out, size_t len);
/* Where a panic is written before the process aborts; the first path given stands. */
int32_t  emusen_core_set_crash_log(const char *path);
/* Log records as lines "level<TAB>category<TAB>text<LF>", whole records only, drained as copied; returns the
 * bytes copied, or with a null `out` the bytes waiting. A null machine drains the library's own records. */
int64_t  emusen_core_log_drain(emusen_machine *machine, uint8_t *out, size_t len);

/* ---- Lifecycle ------------------------------------------------------------------------- */
/* The machine for params' image; null on refusal with the reason in `status` and the detail in params' error.
 * A failed create leaves nothing to free. Nothing in params is kept past the call. */
emusen_machine *emusen_core_create(const emusen_create_params *params, int32_t *status);
/* From any thread, provided no other call on that machine is in flight; every thread of the machine has stopped
 * when it returns. */
int32_t emusen_core_free(emusen_machine *machine);
int32_t emusen_core_reset(emusen_machine *machine);                                        /* RESET */
int64_t emusen_core_machine_info(const emusen_machine *machine, uint8_t *out, size_t len); /* JSON (section 6.4) */
/* The detail of the last failing call on this machine, in words; never parsed. */
int64_t emusen_core_last_error(const emusen_machine *machine, uint8_t *out, size_t len);

/* ---- A frame --------------------------------------------------------------------------- */
int32_t emusen_core_advance(emusen_machine *machine, uint64_t *detail);
int32_t emusen_core_present(emusen_machine *machine);                          /* PRESENT */
int32_t emusen_core_set_options(emusen_machine *machine, uint32_t flags);      /* EMUSEN_OPTION_* */
int64_t emusen_core_frame_count(const emusen_machine *machine);
/* Nanoseconds per phase, in machine info's phases order; returns the count. */
int64_t emusen_core_phases(const emusen_machine *machine, int64_t *out, size_t len);  /* PHASES */
/* Queued events copied and removed, at most `count`; returns the number copied, or with a null `out` the number
 * waiting. Drained after advance, state_load and set_settings. */
int64_t emusen_core_events(emusen_machine *machine, emusen_event *out, size_t count, size_t event_size);

/* ---- Picture --------------------------------------------------------------------------- */
int32_t emusen_core_frame_info(const emusen_machine *machine, emusen_frame_info *out);
int64_t emusen_core_frame_copy(const emusen_machine *machine, uint8_t *out, size_t len);

/* ---- Sound: interleaved stereo int16 --------------------------------------------------- */
int32_t emusen_core_audio_rate(const emusen_machine *machine);           /* the rate of the next sample drained */
int64_t emusen_core_audio_buffered(const emusen_machine *machine);       /* samples, two per stereo frame */
/* Whole stereo frames, at most max_frames, never across a rate change; the samples written, and their rate in
 * `rate`. A null `out` asks how many are buffered. */
int64_t emusen_core_audio_drain(emusen_machine *machine, int16_t *out, size_t len, int64_t max_frames, int32_t *rate);
int32_t emusen_core_set_audio_limit(emusen_machine *machine, uint64_t samples);  /* drop-oldest bound */
int64_t emusen_core_audio_peek(const emusen_machine *machine, int16_t *out, size_t len); /* AUDIO_PEEK */
int32_t emusen_core_set_mutes(emusen_machine *machine, uint32_t mask);          /* MUTES; bit n, channel n */

/* ---- Input ----------------------------------------------------------------------------- */
/* Bits set in `changed` take mask's values; the bits are machine info's controller's. */
int32_t emusen_core_set_buttons(emusen_machine *machine, uint32_t port, uint32_t mask, uint32_t changed);
int32_t emusen_core_set_axis(emusen_machine *machine, uint32_t port, uint32_t axis, double value); /* AXES */

/* ---- State ----------------------------------------------------------------------------- */
int64_t emusen_core_state_size(const emusen_machine *machine, uint32_t kind);
int64_t emusen_core_state_save(emusen_machine *machine, uint32_t kind, uint8_t *out, size_t len);
/* Any kind's bytes; a failed load changes nothing. */
int32_t emusen_core_state_load(emusen_machine *machine, const uint8_t *data, size_t len);
/* One line per field: offset, length, type, path. */
int64_t emusen_core_state_layout(const emusen_machine *machine, uint32_t kind, uint8_t *out, size_t len);

/* ---- Memory spaces, by the ids machine info lists -------------------------------------- */
int64_t emusen_core_space_size(const emusen_machine *machine, uint32_t space);
int64_t emusen_core_space_read(emusen_machine *machine, uint32_t space, uint32_t address, uint8_t *out, size_t len);
int64_t emusen_core_space_write(emusen_machine *machine, uint32_t space, uint32_t address, const uint8_t *data, size_t len);

/* ---- Battery --------------------------------------------------------------------------- */
int64_t emusen_core_battery(const emusen_machine *machine, uint32_t which, uint8_t *out, size_t len, uint32_t *flags);
int32_t emusen_core_battery_saved(emusen_machine *machine, uint32_t which);

/* ---- Cheats: resolved by the host's codecs; each returns the entries taken ---------------- */
/* (address, value, compare) triples, EMUSEN_NO_COMPARE for none; zero clears them. */
int64_t emusen_core_set_rom_patches(emusen_machine *machine, const uint32_t *triples, size_t count); /* ROM_PATCHES */
/* (space, address, value, compare) quads the core applies at its frame's end; zero clears them. */
int64_t emusen_core_set_cheat_pokes(emusen_machine *machine, const uint32_t *quads, size_t count);   /* CHEAT_POKES */

/* ---- Settings, keys from the schema ---------------------------------------------------- */
/* "key=value" lines applied together; run-scope keys only. */
int32_t emusen_core_set_settings(emusen_machine *machine, const uint8_t *text, size_t len);          /* SETTINGS */
/* A JSON object of key to sentence, for the machine's current values. */
int64_t emusen_core_setting_notes(const emusen_machine *machine, uint8_t *out, size_t len);          /* SETTING_NOTES */

/* ---- Debug (section 6.14) -------------------------------------------------------------- */
/* DEBUG: the flags, the depth step over/out stop at (INT32_MIN unarmed) and the guard (-1 unarmed). */
int32_t emusen_core_debug_set(emusen_machine *machine, uint32_t flags, int32_t depth_target, int32_t depth_guard);
/* DEBUG_STACK: the registry's call stack as (source, target) pairs, innermost last. */
int32_t emusen_core_debug_set_stack(emusen_machine *machine, const uint32_t *pairs, size_t count);
/* DEBUG: processor's enabled breakpoints as pairs of first and last address. */
int32_t emusen_core_debug_set_breakpoints(emusen_machine *machine, uint32_t processor, const int32_t *pairs, size_t count);
/* DEBUG: stores to report as (space, first, last) triples; kind 0 the watches, 1 the data breakpoints. */
int32_t emusen_core_debug_set_ranges(emusen_machine *machine, uint32_t kind, const uint32_t *triples, size_t count);
/* DEBUG: the frame through the observed loop; its EMUSEN_STOP_* bits, zero at the frame's end, with the processor
 * that stopped and its program counter. */
int32_t emusen_core_debug_run_frame(emusen_machine *machine, uint32_t flags, uint32_t *processor, uint64_t *pc, uint64_t *detail);
/* DEBUG: the logs, copied and forgotten only when they all fit; each returns the entry count. */
int64_t emusen_core_debug_writes(emusen_machine *machine, uint32_t *out, size_t len);   /* (space, address, value, pc) */
int64_t emusen_core_debug_calls(emusen_machine *machine, uint32_t *out, size_t len);    /* (kind, source, target) */
int64_t emusen_core_debug_profile(emusen_machine *machine, int64_t *out, size_t len);   /* (owner, instructions) */
int64_t emusen_core_debug_coverage(emusen_machine *machine, uint32_t processor, uint8_t *out, size_t len, int64_t *recorded);
int64_t emusen_core_debug_counters(emusen_machine *machine, int64_t *out, size_t len);  /* depth, unmatched returns */
int32_t emusen_core_debug_pc(const emusen_machine *machine, uint32_t processor, uint64_t *pc);
/* DEBUG_REGISTERS: the values, in machine info's processors[].registers order; returns the count. */
int64_t emusen_core_debug_registers(const emusen_machine *machine, uint32_t processor, int64_t *out, size_t len);
/* DEBUG_DISASSEMBLE: `count` instructions from `address` as a JSON array of records. */
int64_t emusen_core_debug_disassemble(emusen_machine *machine, uint32_t processor, uint32_t space, uint32_t address,
                                      uint32_t count, uint8_t *out, size_t len);

/* ---- Function types, one per export, so a host or a check can name each signature ------ */
typedef uint32_t (*emusen_core_abi_version_fn)(void);
typedef uint64_t (*emusen_core_capabilities_fn)(void);
typedef int64_t (*emusen_core_info_fn)(uint8_t *, size_t);
typedef int64_t (*emusen_core_settings_schema_fn)(uint8_t *, size_t);
typedef int64_t (*emusen_core_firmware_for_fn)(const uint8_t *, size_t, uint8_t *, size_t);
typedef int64_t (*emusen_core_status_text_fn)(int32_t, uint8_t *, size_t);
typedef int32_t (*emusen_core_set_crash_log_fn)(const char *);
typedef int64_t (*emusen_core_log_drain_fn)(emusen_machine *, uint8_t *, size_t);
typedef emusen_machine *(*emusen_core_create_fn)(const emusen_create_params *, int32_t *);
typedef int32_t (*emusen_core_free_fn)(emusen_machine *);
typedef int32_t (*emusen_core_reset_fn)(emusen_machine *);
typedef int64_t (*emusen_core_machine_info_fn)(const emusen_machine *, uint8_t *, size_t);
typedef int64_t (*emusen_core_last_error_fn)(const emusen_machine *, uint8_t *, size_t);
typedef int32_t (*emusen_core_advance_fn)(emusen_machine *, uint64_t *);
typedef int32_t (*emusen_core_present_fn)(emusen_machine *);
typedef int32_t (*emusen_core_set_options_fn)(emusen_machine *, uint32_t);
typedef int64_t (*emusen_core_frame_count_fn)(const emusen_machine *);
typedef int64_t (*emusen_core_phases_fn)(const emusen_machine *, int64_t *, size_t);
typedef int64_t (*emusen_core_events_fn)(emusen_machine *, emusen_event *, size_t, size_t);
typedef int32_t (*emusen_core_frame_info_fn)(const emusen_machine *, emusen_frame_info *);
typedef int64_t (*emusen_core_frame_copy_fn)(const emusen_machine *, uint8_t *, size_t);
typedef int32_t (*emusen_core_audio_rate_fn)(const emusen_machine *);
typedef int64_t (*emusen_core_audio_buffered_fn)(const emusen_machine *);
typedef int64_t (*emusen_core_audio_drain_fn)(emusen_machine *, int16_t *, size_t, int64_t, int32_t *);
typedef int32_t (*emusen_core_set_audio_limit_fn)(emusen_machine *, uint64_t);
typedef int64_t (*emusen_core_audio_peek_fn)(const emusen_machine *, int16_t *, size_t);
typedef int32_t (*emusen_core_set_mutes_fn)(emusen_machine *, uint32_t);
typedef int32_t (*emusen_core_set_buttons_fn)(emusen_machine *, uint32_t, uint32_t, uint32_t);
typedef int32_t (*emusen_core_set_axis_fn)(emusen_machine *, uint32_t, uint32_t, double);
typedef int64_t (*emusen_core_state_size_fn)(const emusen_machine *, uint32_t);
typedef int64_t (*emusen_core_state_save_fn)(emusen_machine *, uint32_t, uint8_t *, size_t);
typedef int32_t (*emusen_core_state_load_fn)(emusen_machine *, const uint8_t *, size_t);
typedef int64_t (*emusen_core_state_layout_fn)(const emusen_machine *, uint32_t, uint8_t *, size_t);
typedef int64_t (*emusen_core_space_size_fn)(const emusen_machine *, uint32_t);
typedef int64_t (*emusen_core_space_read_fn)(emusen_machine *, uint32_t, uint32_t, uint8_t *, size_t);
typedef int64_t (*emusen_core_space_write_fn)(emusen_machine *, uint32_t, uint32_t, const uint8_t *, size_t);
typedef int64_t (*emusen_core_battery_fn)(const emusen_machine *, uint32_t, uint8_t *, size_t, uint32_t *);
typedef int32_t (*emusen_core_battery_saved_fn)(emusen_machine *, uint32_t);
typedef int64_t (*emusen_core_set_rom_patches_fn)(emusen_machine *, const uint32_t *, size_t);
typedef int64_t (*emusen_core_set_cheat_pokes_fn)(emusen_machine *, const uint32_t *, size_t);
typedef int32_t (*emusen_core_set_settings_fn)(emusen_machine *, const uint8_t *, size_t);
typedef int64_t (*emusen_core_setting_notes_fn)(const emusen_machine *, uint8_t *, size_t);
typedef int32_t (*emusen_core_debug_set_fn)(emusen_machine *, uint32_t, int32_t, int32_t);
typedef int32_t (*emusen_core_debug_set_stack_fn)(emusen_machine *, const uint32_t *, size_t);
typedef int32_t (*emusen_core_debug_set_breakpoints_fn)(emusen_machine *, uint32_t, const int32_t *, size_t);
typedef int32_t (*emusen_core_debug_set_ranges_fn)(emusen_machine *, uint32_t, const uint32_t *, size_t);
typedef int32_t (*emusen_core_debug_run_frame_fn)(emusen_machine *, uint32_t, uint32_t *, uint64_t *, uint64_t *);
typedef int64_t (*emusen_core_debug_writes_fn)(emusen_machine *, uint32_t *, size_t);
typedef int64_t (*emusen_core_debug_calls_fn)(emusen_machine *, uint32_t *, size_t);
typedef int64_t (*emusen_core_debug_profile_fn)(emusen_machine *, int64_t *, size_t);
typedef int64_t (*emusen_core_debug_coverage_fn)(emusen_machine *, uint32_t, uint8_t *, size_t, int64_t *);
typedef int64_t (*emusen_core_debug_counters_fn)(emusen_machine *, int64_t *, size_t);
typedef int32_t (*emusen_core_debug_pc_fn)(const emusen_machine *, uint32_t, uint64_t *);
typedef int64_t (*emusen_core_debug_registers_fn)(const emusen_machine *, uint32_t, int64_t *, size_t);
typedef int64_t (*emusen_core_debug_disassemble_fn)(emusen_machine *, uint32_t, uint32_t, uint32_t, uint32_t, uint8_t *, size_t);

#ifdef __cplusplus
}
#endif
#endif
