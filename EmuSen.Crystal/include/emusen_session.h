/* emusen_session.h - the session service, emusen.session@1, version 1.1. Owned by Crystal.
 *
 * What a command module calls to reach a running core, and what a frontend's own windows call for the same thing: one
 * C function table, emusen_session_v1, provided by Crystal and handed to a module by the shell, which never reads it.
 * The normative text is EmuSen_DianaSh.md section 9 in the EmuSen manual; this header is its machine-checked half,
 * and where the two disagree the page wins and the header is in error. Section 13.6 there records version 1.0 as
 * built, and the C# provider that serves it until Crystal is written in Rust.
 *
 * Everything here is @since 1.0 unless marked otherwise; section 13.11 there records what 1.1 added. Names: the table emusen_session_v1, constants
 * EMUSEN_SESSION_* and, for statuses, EMUSEN_SVC_*; nothing here collides with emusen_core.h or emusen_platform.h, so a
 * module may include all three.
 *
 * Rules every function follows (EmuSen_DianaSh.md sections 2, 6.0 and 9.6, the core ABI's one layer up):
 *  - Version. A caller checks that version >> 16 is EMUSEN_SESSION_MAJOR before calling anything, and that the
 *    table's size reaches a function before calling one a later minor appended (EMUSEN_SESSION_HAS). The table is
 *    only ever extended by appending function pointers.
 *  - Calls. Every function takes the table's ctx first, then the session it names: 0 for the session the host made
 *    current, or an id from `sessions`. A module calls the table only during its own call, on that call's thread, and
 *    keeps no pointer of the table's past it.
 *  - Statuses. A negative return is a status; zero or more is success: a length, an id, a count or a truth value, as
 *    each function says. The words for a failure are read with last_error on the same thread. EMUSEN_SVC_ABSENT is an
 *    answer, not a failure.
 *  - Text is UTF-8, passed as a pointer and a length in bytes and never assumed to end in a NUL. A null pointer with
 *    a zero length is the absent string, which a function that takes an optional name reads as "none".
 *  - Copy out. A function that fills a buffer takes (out, len) and returns the whole length in bytes; it writes only
 *    when the buffer holds all of it, and a null out asks the length. No pointer into a session is ever returned:
 *    memory, pictures and records are copied. Nothing passed in is kept after the call. A change answers with its
 *    id, its count or 0 as the return value and copies nothing, so asking a length never repeats it; the one change
 *    in 1.0 that copies a record, label_load, applies only on the call whose buffer holds all of it.
 *  - Shapes. Lists, records and descriptors are JSON (an object or an array, keys in snake_case, numbers integers
 *    except a "percent"); values read in bulk are byte spans; ids are 32-bit. An address is a 32-bit pattern, passed
 *    as uint32_t and written unsigned in JSON, as a MIPS kernel address needs. A caller skips keys it does not know.
 *  - Reads and changes. Each function's comment says which it is. A read is answered from the session as it stands,
 *    on the calling thread, never waiting for a frame. A change is applied at the next frame boundary, the call
 *    waiting for it; a provider with no frame loop of its own applies it at once, and its host calls the table only
 *    where a command may touch the session (section 13.6). A change is refused with EMUSEN_SVC_READ_ONLY by a table
 *    handed out for a read-only context (section 6.5).
 *  - Open enumerations. An op, kind or flag this version does not know is refused with EMUSEN_SVC_NOT_SUPPORTED;
 *    so is a function the session cannot answer, with the session's own words for it in last_error.
 *  - Nothing unwinds. A failure inside the session the provider adapts is returned as EMUSEN_SVC_FAULT.
 *  - The service exposes what a command asks of a session, never what a core reports to it (section 9.2).
 */
#ifndef EMUSEN_SESSION_H
#define EMUSEN_SESSION_H

#include <stddef.h>
#include <stdint.h>

#ifdef __cplusplus
extern "C" {
#endif

#define EMUSEN_SESSION_MAJOR 1u
#define EMUSEN_SESSION_MINOR 1u
#define EMUSEN_SESSION_VERSION ((EMUSEN_SESSION_MAJOR << 16) | EMUSEN_SESSION_MINOR)

/* The name a module asks the shell for, with the major after an @. */
#define EMUSEN_SESSION_SERVICE "emusen.session"

/* True when a table's provider has the function `field`: what a caller checks before calling one a later minor added. */
#define EMUSEN_SESSION_HAS(table, field) \
    ((table)->size >= offsetof(emusen_session_v1, field) + sizeof((table)->field) && (table)->field != NULL)

/* ---- Statuses (section 9.7). The band is the platform's, -1024 .. -1279; the two below it are the project's own. -- */
#define EMUSEN_SVC_NULL           (-1)    /* a null table, ctx or required pointer */
#define EMUSEN_SVC_BAD_STRING     (-5)    /* text that is not UTF-8 */
#define EMUSEN_SVC_NO_SESSION     (-1100) /* no such session, or none is running */
#define EMUSEN_SVC_NO_SUCH_SPACE  (-1101)
#define EMUSEN_SVC_READ_ONLY      (-1102) /* a change in a read-only context, or to a space that cannot be written */
#define EMUSEN_SVC_NOT_SUPPORTED  (-1103) /* an op this version does not know, or a feature the session lacks */
#define EMUSEN_SVC_BAD_ARGUMENT   (-1104) /* a number, a name or a JSON argument the call refuses */
#define EMUSEN_SVC_NO_SUCH_ID     (-1105) /* no breakpoint, watch, entry, register or label of that id or name */
#define EMUSEN_SVC_PARSE          (-1106) /* an expression or a code that does not parse; the parser's words follow */
#define EMUSEN_SVC_ABSENT         (-1107) /* what was asked for does not exist; an answer, not a failure */
#define EMUSEN_SVC_FAULT          (-1108) /* the session failed inside the call; the words say how */

/* ---- Open enumerations --------------------------------------------------------------------------------- */

/* A processor argument naming none (@since 1.1): the session's own registries, registers and expression context, as a
 * command that names no processor reaches them, which on a session listing processors need not be processor 0's. */
#define EMUSEN_SESSION_PROCESSOR_SESSION 4294967295u

/* run_control's op. A processor is an index of `processors`; 0 is the main one. */
#define EMUSEN_SESSION_RUN_PAUSE            0u
#define EMUSEN_SESSION_RUN_RESUME           1u
#define EMUSEN_SESSION_RUN_STEP             2u  /* argument: instructions, at least 1 */
#define EMUSEN_SESSION_RUN_STEP_OVER        3u
#define EMUSEN_SESSION_RUN_STEP_OUT         4u
#define EMUSEN_SESSION_RUN_TO_ADDRESS       5u  /* argument: the address; a breakpoint is added, its id returned */
#define EMUSEN_SESSION_RUN_TO_INTERRUPT     6u  /* argument: EMUSEN_SESSION_INTERRUPT_* */
#define EMUSEN_SESSION_RUN_TO_SCANLINE      7u  /* argument: the scanline */
#define EMUSEN_SESSION_RUN_TO_FRAME         8u  /* argument: the frame number */

/* The kinds of a call-stack frame, and the interrupts run_control can run to. */
#define EMUSEN_SESSION_FRAME_CALL           0u
#define EMUSEN_SESSION_INTERRUPT_NMI        1u
#define EMUSEN_SESSION_INTERRUPT_IRQ        2u
#define EMUSEN_SESSION_INTERRUPT_BRK        3u
#define EMUSEN_SESSION_INTERRUPT_COP        4u

/* watch_add's kind. */
#define EMUSEN_SESSION_WATCH_WRITE          0u
#define EMUSEN_SESSION_WATCH_READ           1u
#define EMUSEN_SESSION_WATCH_BOTH           2u

/* bp_arm_condition's flags. */
#define EMUSEN_SESSION_CONDITION_LOG_ONLY   1u  /* record the condition in the log and do not halt */
#define EMUSEN_SESSION_CONDITION_DISARM     2u

/* coverage_op's op. Args and results are JSON unless said otherwise. */
#define EMUSEN_SESSION_COVERAGE_ARM             0u  /* changes */
#define EMUSEN_SESSION_COVERAGE_DISARM          1u  /* changes */
#define EMUSEN_SESSION_COVERAGE_CLEAR           2u  /* changes */
#define EMUSEN_SESSION_COVERAGE_MARK            3u  /* changes */
#define EMUSEN_SESSION_COVERAGE_CLEAR_MARK      4u  /* changes */
#define EMUSEN_SESSION_COVERAGE_STATUS          5u  /* {"armed","instructions","has_mark","entry_points"} */
#define EMUSEN_SESSION_COVERAGE_QUERY           6u  /* {"address","length","limit"} -> {"executed","total","new_since_mark","addresses"} */
#define EMUSEN_SESSION_COVERAGE_NEW_SINCE_MARK  7u  /* {"address","length","limit"} -> [address] */
#define EMUSEN_SESSION_COVERAGE_ENTRY_POINTS    8u  /* {"limit"} -> [{"address","kind","entries"}] */
#define EMUSEN_SESSION_COVERAGE_EXPORT          9u  /* -> the coverage map's bytes */
#define EMUSEN_SESSION_COVERAGE_IMPORT          10u /* args: a map's bytes; changes */
#define EMUSEN_SESSION_COVERAGE_MERGE           11u /* args: 8 bytes of instructions, little-endian, then a bitmap; changes */

/* profile_op's op. */
#define EMUSEN_SESSION_PROFILE_ARM          0u  /* changes */
#define EMUSEN_SESSION_PROFILE_DISARM       1u  /* changes */
#define EMUSEN_SESSION_PROFILE_CLEAR        2u  /* changes */
#define EMUSEN_SESSION_PROFILE_STATUS       3u  /* {"profiling","instructions","unmatched_returns","depth"} */
#define EMUSEN_SESSION_PROFILE_HOTTEST      4u  /* {"limit"} -> [{"address","instructions","calls","percent"}] */
#define EMUSEN_SESSION_PROFILE_RESET_STACK  5u  /* changes; forgets the live call chain (@since 1.1) */

/* counters_op's op. */
#define EMUSEN_SESSION_COUNTERS_ARM         0u  /* {"space","size"}; changes */
#define EMUSEN_SESSION_COUNTERS_DISARM      1u  /* changes */
#define EMUSEN_SESSION_COUNTERS_CLEAR       2u  /* changes */
#define EMUSEN_SESSION_COUNTERS_STATUS      3u  /* {"armed","space","size"} */
#define EMUSEN_SESSION_COUNTERS_AT          4u  /* {"address"} -> {"reads","writes","executes","uninitialized_reads"} */
#define EMUSEN_SESSION_COUNTERS_TOTALS      5u  /* {"address","length"} -> the same with "touched_bytes" */
#define EMUSEN_SESSION_COUNTERS_HOTTEST     6u  /* {"sort":"reads"|"writes"|"executes"|"uninitialized","limit"} */
#define EMUSEN_SESSION_COUNTERS_UNTOUCHED   7u  /* {"address","length","limit"} -> [address] */

/* dma_log_op's op. */
#define EMUSEN_SESSION_DMA_LOG_ARM          0u  /* {"capacity"} or {}; changes */
#define EMUSEN_SESSION_DMA_LOG_DISARM       1u  /* changes */
#define EMUSEN_SESSION_DMA_LOG_CLEAR        2u  /* changes */
#define EMUSEN_SESSION_DMA_LOG_STATUS       3u  /* {"armed","capacity","transfers","bytes","first_frame","last_frame"} */
#define EMUSEN_SESSION_DMA_LOG_TAIL         4u  /* {"count"} -> [transfer] */
#define EMUSEN_SESSION_DMA_LOG_CHANNELS     5u  /* -> [{"channel","general_transfers","general_bytes","hdma_transfers","hdma_bytes"}] */
#define EMUSEN_SESSION_DMA_LOG_DESTINATIONS 6u  /* -> [{"destination","destination_name","transfers","bytes","channel_mask"}] */

/* register_flow_op's op. */
#define EMUSEN_SESSION_FLOW_ARM             0u  /* {"window","size","capacity"}; changes */
#define EMUSEN_SESSION_FLOW_DISARM          1u  /* changes */
#define EMUSEN_SESSION_FLOW_CLEAR           2u  /* changes */
#define EMUSEN_SESSION_FLOW_STATUS          3u  /* {"armed","window","size","capacity","accesses","longest_poll":{...},"current_poll":{...}} */
#define EMUSEN_SESSION_FLOW_TAIL            4u  /* {"count"} -> [{"sequence","frame","register","value","write","actor"}] */
#define EMUSEN_SESSION_FLOW_STATS           5u  /* -> [{"register","reads","writes","distinct_values","last_value"}] */

/* cheat_op's op. A write is {"space","address","value","width","type","bit","big_endian","repeat","repeat_address","repeat_value"}. */
#define EMUSEN_SESSION_CHEAT_LIST           0u  /* -> {"master","cheats":[{"id","kind","description","enabled","compare","writes"}]} */
#define EMUSEN_SESSION_CHEAT_ADD            1u  /* {"kind":"ram"|"rom","writes","compare","description","enabled"} -> the id; changes */
#define EMUSEN_SESSION_CHEAT_POKE           2u  /* {"space","address","value","description","enabled"} -> the id; changes */
#define EMUSEN_SESSION_CHEAT_ROM_PATCH      3u  /* {"address","value","compare","description","enabled"} -> the id; changes */
#define EMUSEN_SESSION_CHEAT_ENABLE         4u  /* {"id"}; changes */
#define EMUSEN_SESSION_CHEAT_DISABLE        5u  /* {"id"}; changes */
#define EMUSEN_SESSION_CHEAT_REMOVE         6u  /* {"id"}; changes */
#define EMUSEN_SESSION_CHEAT_CLEAR          7u  /* changes */
#define EMUSEN_SESSION_CHEAT_MASTER         8u  /* {} reads -> {"enabled"}; {"enabled":b} changes */
#define EMUSEN_SESSION_CHEAT_EXPORT         9u  /* -> the cheat file's JSON, as Galaxia's CheatFile */
#define EMUSEN_SESSION_CHEAT_IMPORT         10u /* args: a cheat file's JSON -> the number loaded, the rest skipped; changes */
#define EMUSEN_SESSION_CHEAT_DECODE         11u /* {"code","codec":"auto"|"explicit","unchecked"} -> {"codec","kind","space","address","value","compare","writes"};
                                                * "unchecked" (@since 1.1) decodes without asking the codec first, its own words refusing */
#define EMUSEN_SESSION_CHEAT_DB_SYSTEMS     12u /* -> [{"system","count"}]; NOT_SUPPORTED without a database */
#define EMUSEN_SESSION_CHEAT_DB_GAMES       13u /* {"system","filter"} -> [{"system","game","path"}] */
#define EMUSEN_SESSION_CHEAT_DB_FIND        14u /* {"text","limit"} -> [{"system","game","path"}] */
/* @since 1.1: what the cheat command needs beyond 1.0, the files and the database being the session's to reach. */
#define EMUSEN_SESSION_CHEAT_CLAIMS         15u /* {"code"} -> {"auto","explicit"}: whether each codec claims it, null with no codec */
#define EMUSEN_SESSION_CHEAT_CHT_PARSE      16u /* {"text"} -> {"game":null,"cheats":[{"kind","description","enabled","writes"}],"skipped"} */
#define EMUSEN_SESSION_CHEAT_CHT_EXPORT     17u /* -> {"text","exported","rom_patches","conditional"}: the RAM pokes as a .cht file */
#define EMUSEN_SESSION_CHEAT_FILES          18u /* -> {"directory","names"}: the saved cheat lists */
#define EMUSEN_SESSION_CHEAT_FILE_READ      19u /* {"name"} -> {"path","file"}: a list as IMPORT takes it, null where unreadable */
#define EMUSEN_SESSION_CHEAT_FILE_WRITE     20u /* {"name"} -> the number saved; BAD_ARGUMENT for a name no file may have */
#define EMUSEN_SESSION_CHEAT_DB_STATUS      21u /* -> {"directory","exists"} */
#define EMUSEN_SESSION_CHEAT_DB_PARSE       22u /* {"game"} -> CHT_PARSE's record for the best match, "game" its name or null */
#define EMUSEN_SESSION_CHEAT_DB_PRUNE       23u /* {"apply"} -> {"can_apply","reason","removing":[{"system","files","bytes"}],"keeping","pruned","failed"} */
#define EMUSEN_SESSION_CHEAT_DB_UPDATE      24u /* -> {"installed","directory","attribution"}: downloads and installs the database */

/* telemetry's kind. Registers are [{"name","value","bits"}]; the rest as sprites, palettes and audio_channels give them. */
#define EMUSEN_SESSION_TELEMETRY_CPU            0u
#define EMUSEN_SESSION_TELEMETRY_VIDEO          1u
#define EMUSEN_SESSION_TELEMETRY_APU            2u
#define EMUSEN_SESSION_TELEMETRY_COPROCESSOR    3u
#define EMUSEN_SESSION_TELEMETRY_SPRITES        4u
#define EMUSEN_SESSION_TELEMETRY_PALETTES       5u
#define EMUSEN_SESSION_TELEMETRY_AUDIO_CHANNELS 6u
#define EMUSEN_SESSION_TELEMETRY_HARDWARE_LOAD  7u  /* [{"name","percent","kind"}] */

/* events_drain's kinds, as bits; 0 asks for every kind. */
#define EMUSEN_SESSION_EVENT_RUN            1u  /* {"kind":"run","op":"resume"|"step"}: run control asked the loop to run */

/* ---- The table ------------------------------------------------------------------------------------------- */
typedef struct emusen_session_v1 {
    uint32_t size;      /* the provider's size of this table, in bytes */
    uint32_t version;   /* EMUSEN_SESSION_VERSION the provider was built with */
    void *ctx;          /* passed first to every function; the provider's, never read by the caller */

    /* The words for the last failing call on the calling thread. */
    int64_t (*last_error)(void *ctx, uint8_t *out, size_t len);

    /* -- Session ------------------------------------------------------------------------------------------- */
    /* reads. [{"id","engine","current"}]; empty when no session runs. */
    int64_t (*sessions)(void *ctx, uint8_t *out, size_t len);
    /* reads. {"id","engine","frame","max_sprites","tilemap_entry_stride","capabilities":[name]}. */
    int64_t (*info)(void *ctx, uint32_t session, uint8_t *out, size_t len);
    /* reads. Free text, the core's own (section 9.2). */
    int64_t (*summary)(void *ctx, uint32_t session, uint8_t *out, size_t len);

    /* -- Run control ------------------------------------------------------------------------------------- */
    /* changes. EMUSEN_SESSION_RUN_*; returns 0, or RUN_TO_ADDRESS's breakpoint id. RESUME asks the host's loop to run
     * and is answered with no session running (@since 1.1), as events_drain then is for session 0. */
    int64_t (*run_control)(void *ctx, uint32_t session, uint32_t op, uint32_t processor, int64_t argument);
    /* reads. {"requested","last_break","last_data_break","last_event_break","single_step","step_depth","pending"}. */
    int64_t (*halt_state)(void *ctx, uint32_t session, uint32_t processor, uint8_t *out, size_t len);

    /* -- Memory -------------------------------------------------------------------------------------------- */
    /* reads. [{"name","size","writable","side_effects"}]; a size of 0 is a space that does not say. */
    int64_t (*spaces)(void *ctx, uint32_t session, uint8_t *out, size_t len);
    /* reads. len bytes from address, which the space reads as it reads any address; returns len. */
    int64_t (*space_read)(void *ctx, uint32_t session, const uint8_t *space, size_t space_len, uint32_t address,
                          uint8_t *out, size_t len);
    /* changes. len bytes to address. */
    int32_t (*space_write)(void *ctx, uint32_t session, const uint8_t *space, size_t space_len, uint32_t address,
                           const uint8_t *data, size_t len);
    /* reads. {"space","offset","addressable"}: what a main-bus address decodes to; ABSENT where the core does not say. */
    int64_t (*resolve_physical)(void *ctx, uint32_t session, uint32_t address, uint8_t *out, size_t len);

    /* -- Processors and registers ------------------------------------------------------------------------ */
    /* reads. [{"index","name","description","can_halt","code_space","coverage","call_stack","registers","writable_registers",
     * "pc"}], "pc" (@since 1.1) where the processor is executing, null where the core does not say; empty for a session
     * that names none, which has processor 0 and, where it reports one, a coprocessor as 1. */
    int64_t (*processors)(void *ctx, uint32_t session, uint8_t *out, size_t len);
    /* reads. [{"name","value","bits"}]. */
    int64_t (*registers)(void *ctx, uint32_t session, uint32_t processor, uint8_t *out, size_t len);
    /* changes. NO_SUCH_ID for an unknown register, BAD_ARGUMENT for a value wider than it, READ_ONLY where refused. */
    int32_t (*set_register)(void *ctx, uint32_t session, uint32_t processor, const uint8_t *name, size_t name_len,
                            uint64_t value);

    /* -- Disassembly --------------------------------------------------------------------------------------- */
    /* reads. [{"address","bytes":[byte],"mnemonic","operands","reference":{"kind":"call"|"write"|"read","target"}|null}];
     * an absent space is the processor's code space; hints are a JSON array of the core's own words, or absent. */
    int64_t (*disassemble)(void *ctx, uint32_t session, uint32_t processor, const uint8_t *space, size_t space_len,
                           uint32_t address, uint32_t count, const uint8_t *hints, size_t hints_len,
                           uint8_t *out, size_t len);

    /* -- Breakpoints and halts --------------------------------------------------------------------------- */
    /* changes. {"address","end","condition","log"} -> the id. */
    int64_t (*bp_add)(void *ctx, uint32_t session, uint32_t processor, const uint8_t *spec, size_t spec_len);
    /* changes. {"space","address","end","value","on_read","changed_only","condition","log"} -> the id; value -1 is any. */
    int64_t (*bp_add_data)(void *ctx, uint32_t session, uint32_t processor, const uint8_t *spec, size_t spec_len);
    /* changes. */
    int32_t (*bp_set_enabled)(void *ctx, uint32_t session, uint32_t processor, uint32_t id, uint32_t enabled);
    /* changes. */
    int32_t (*bp_remove)(void *ctx, uint32_t session, uint32_t processor, uint32_t id);
    /* reads. {"code":[...],"data":[...],"forbid":[...],"depth_guard","uninitialized":{"space"}|null}. */
    int64_t (*bp_list)(void *ctx, uint32_t session, uint32_t processor, uint8_t *out, size_t len);
    /* reads. {"recorded","entries":[{"id","address","text"}]}, the last count entries. */
    int64_t (*bp_log)(void *ctx, uint32_t session, uint32_t processor, uint32_t count, uint8_t *out, size_t len);
    /* changes. */
    int32_t (*bp_log_clear)(void *ctx, uint32_t session, uint32_t processor);
    /* changes. EMUSEN_SESSION_CONDITION_*; NO_SUCH_ID for a condition the session does not detect, or one not armed. */
    int32_t (*bp_arm_condition)(void *ctx, uint32_t session, uint32_t processor, const uint8_t *name, size_t name_len,
                                uint32_t flags);
    /* reads. {"available":[{"name","description"}],"armed":[{"name","log_only","enabled","hits"}]}. */
    int64_t (*bp_conditions)(void *ctx, uint32_t session, uint32_t processor, uint8_t *out, size_t len);
    /* changes. A depth below 0 disarms the guard. */
    int32_t (*bp_depth_guard)(void *ctx, uint32_t session, uint32_t processor, int32_t depth);
    /* changes. A range execution must not enter -> the id. */
    int64_t (*bp_forbid_range)(void *ctx, uint32_t session, uint32_t processor, uint32_t address, uint32_t end);
    /* changes. Halts on a read of a byte of space not yet written; an absent space disarms. */
    int32_t (*bp_uninitialized)(void *ctx, uint32_t session, uint32_t processor, const uint8_t *space, size_t space_len,
                                uint32_t size);

    /* -- Watches and frame logs -------------------------------------------------------------------------- */
    /* changes. EMUSEN_SESSION_WATCH_* -> the id. */
    int64_t (*watch_add)(void *ctx, uint32_t session, const uint8_t *space, size_t space_len, uint32_t address,
                         uint32_t length, uint32_t kind);
    /* changes. */
    int32_t (*watch_remove)(void *ctx, uint32_t session, uint32_t id);
    /* reads. [{"id","space","address","length","kind"}]. */
    int64_t (*watch_list)(void *ctx, uint32_t session, uint8_t *out, size_t len);
    /* reads. [{"sequence","kind","address","value","context"}], the last count. */
    int64_t (*watch_events)(void *ctx, uint32_t session, uint32_t id, uint32_t count, uint8_t *out, size_t len);
    /* reads. {"total","retained","sites":[{"context","count"}]}. */
    int64_t (*watch_summary)(void *ctx, uint32_t session, uint32_t id, uint8_t *out, size_t len);
    /* changes. Forgets a watch's events and keeps the watch. */
    int32_t (*watch_clear)(void *ctx, uint32_t session, uint32_t id);
    /* changes. A value of width bytes, little-endian, sampled once a frame -> the id. */
    int64_t (*framelog_add)(void *ctx, uint32_t session, const uint8_t *space, size_t space_len, uint32_t address,
                            uint32_t width);
    /* changes. */
    int32_t (*framelog_remove)(void *ctx, uint32_t session, uint32_t id);
    /* reads. [{"id","space","address","width"}]. */
    int64_t (*framelog_list)(void *ctx, uint32_t session, uint8_t *out, size_t len);
    /* reads. [{"frame","value"}], the last count. */
    int64_t (*framelog_samples)(void *ctx, uint32_t session, uint32_t id, uint32_t count, uint8_t *out, size_t len);
    /* changes. */
    int32_t (*framelog_clear)(void *ctx, uint32_t session, uint32_t id);

    /* -- Coverage, profile, counters, call stack ------------------------------------------------------- */
    /* EMUSEN_SESSION_COVERAGE_*, on a processor's coverage. */
    int64_t (*coverage_op)(void *ctx, uint32_t session, uint32_t processor, uint32_t op, const uint8_t *args,
                           size_t args_len, uint8_t *out, size_t len);
    /* EMUSEN_SESSION_PROFILE_*, on a processor's call stack. */
    int64_t (*profile_op)(void *ctx, uint32_t session, uint32_t processor, uint32_t op, const uint8_t *args,
                          size_t args_len, uint8_t *out, size_t len);
    /* EMUSEN_SESSION_COUNTERS_*. */
    int64_t (*counters_op)(void *ctx, uint32_t session, uint32_t op, const uint8_t *args, size_t args_len,
                           uint8_t *out, size_t len);
    /* reads. [{"source","target","kind","frame"}], innermost first, at most count. */
    int64_t (*call_stack)(void *ctx, uint32_t session, uint32_t processor, uint32_t count, uint8_t *out, size_t len);

    /* -- Freezes and labels ------------------------------------------------------------------------------ */
    /* changes. -> the id. */
    int64_t (*freeze_add)(void *ctx, uint32_t session, const uint8_t *space, size_t space_len, uint32_t address,
                          uint32_t value);
    /* changes. */
    int32_t (*freeze_remove)(void *ctx, uint32_t session, uint32_t id);
    /* reads. [{"id","space","address","value","blocked"}]. */
    int64_t (*freeze_list)(void *ctx, uint32_t session, uint8_t *out, size_t len);
    /* changes. */
    int32_t (*freeze_clear)(void *ctx, uint32_t session);
    /* changes. An absent comment is none. */
    int32_t (*label_add)(void *ctx, uint32_t session, uint32_t address, const uint8_t *name, size_t name_len,
                         const uint8_t *comment, size_t comment_len);
    /* changes. By name, or by address when the name is absent. */
    int32_t (*label_remove)(void *ctx, uint32_t session, const uint8_t *name, size_t name_len, uint32_t address);
    /* reads. [{"address","name","comment"}]. */
    int64_t (*label_list)(void *ctx, uint32_t session, uint8_t *out, size_t len);
    /* reads. {"name","comment"}; ABSENT where no label is. */
    int64_t (*label_at)(void *ctx, uint32_t session, uint32_t address, uint8_t *out, size_t len);
    /* changes, on the call whose buffer holds the record. A label file's text -> {"added","errors":[text]}. */
    int64_t (*label_load)(void *ctx, uint32_t session, const uint8_t *text, size_t text_len, uint8_t *out, size_t len);
    /* reads. The labels as a label file's text. */
    int64_t (*label_save)(void *ctx, uint32_t session, uint8_t *out, size_t len);
    /* changes. */
    int32_t (*label_clear)(void *ctx, uint32_t session);

    /* -- Chip views ----------------------------------------------------------------------------------------- */
    /* reads. [{"index","general","hdma","control","destination","destination_name","source","size","hdma_active",
     * "table","line_counter","indirect"}]. */
    int64_t (*dma_channels)(void *ctx, uint32_t session, uint8_t *out, size_t len);
    /* EMUSEN_SESSION_DMA_LOG_*. A transfer is {"sequence","frame","scanline","channel","kind","control","destination",
     * "destination_name","source","length"}; "destination_name" (@since 1.1, here and in DESTINATIONS) is the core's own
     * name for the register, null where it gives none. */
    int64_t (*dma_log_op)(void *ctx, uint32_t session, uint32_t op, const uint8_t *args, size_t args_len,
                          uint8_t *out, size_t len);
    /* EMUSEN_SESSION_FLOW_*. */
    int64_t (*register_flow_op)(void *ctx, uint32_t session, uint32_t op, const uint8_t *args, size_t args_len,
                                uint8_t *out, size_t len);
    /* reads. {"capacity","refreshes","unchanged_for","frames":[[register]]}, the last count frames. */
    int64_t (*coprocessor_history)(void *ctx, uint32_t session, uint32_t count, uint8_t *out, size_t len);
    /* reads. [{"name","address","width","mode"}]: where each vector's pointer lives. */
    int64_t (*vectors)(void *ctx, uint32_t session, uint8_t *out, size_t len);
    /* reads. {"stride","text"}: the core's words for the tilemap entry at address. */
    int64_t (*tilemap_entry)(void *ctx, uint32_t session, const uint8_t *space, size_t space_len, uint32_t address,
                             uint8_t *out, size_t len);
    /* reads. 64 palette indices, or none where the core decodes no tiles. */
    int64_t (*tile_pixels)(void *ctx, uint32_t session, const uint8_t *space, size_t space_len, uint32_t address,
                           uint32_t bpp, uint8_t *out, size_t len);
    /* reads. {"max","sprites":[{"index","x","y","width","height","tile","palette","priority","flip_x","flip_y"}]}. */
    int64_t (*sprites)(void *ctx, uint32_t session, uint8_t *out, size_t len);
    /* reads. [{"index","colors":[[r,g,b]]}]. */
    int64_t (*palettes)(void *ctx, uint32_t session, uint8_t *out, size_t len);
    /* reads. [{"index","name","active","level","muted","info"}]. */
    int64_t (*audio_channels)(void *ctx, uint32_t session, uint8_t *out, size_t len);
    /* changes. The channel is a 32-bit pattern, read as the core's signed index (@since 1.1; 1.0 refused one above 2^31 - 1). */
    int32_t (*set_channel_muted)(void *ctx, uint32_t session, uint32_t channel, uint32_t muted);

    /* -- Expressions --------------------------------------------------------------------------------------- */
    /* reads. {"value"}; PARSE with the evaluator's words. */
    int64_t (*evaluate)(void *ctx, uint32_t session, uint32_t processor, const uint8_t *text, size_t text_len,
                        uint8_t *out, size_t len);
    /* reads. [name], those containing filter where one is given, ignoring case. */
    int64_t (*symbols)(void *ctx, uint32_t session, uint32_t processor, const uint8_t *filter, size_t filter_len,
                       uint8_t *out, size_t len);

    /* -- Cheats ------------------------------------------------------------------------------------------------ */
    /* EMUSEN_SESSION_CHEAT_*. DB_PRUNE with "apply" and DB_UPDATE change the database on disk and answer with a record:
     * the first call applies, and the record is kept until a buffer takes it, as events_drain keeps its events. */
    int64_t (*cheat_op)(void *ctx, uint32_t session, uint32_t op, const uint8_t *args, size_t args_len,
                        uint8_t *out, size_t len);
    /* reads. {"auto":codec|null,"explicit":codec|null,"systems":[name]}, a codec being {"name","kind","space"}. */
    int64_t (*cheat_codecs)(void *ctx, uint32_t session, uint8_t *out, size_t len);

    /* -- Trace ------------------------------------------------------------------------------------------------ */
    /* changes. Traces the next count instructions. */
    int32_t (*trace)(void *ctx, uint32_t session, uint32_t processor, uint32_t count);
    /* changes. */
    int32_t (*trace_off)(void *ctx, uint32_t session, uint32_t processor);

    /* -- Telemetry ------------------------------------------------------------------------------------------ */
    /* reads. EMUSEN_SESSION_TELEMETRY_*, as JSON. */
    int64_t (*telemetry)(void *ctx, uint32_t session, uint32_t kind, uint8_t *out, size_t len);
    /* reads. Width and height as two little-endian uint32, then RGBA8888 rows; 0 by 0 where the core has none. */
    int64_t (*tile_sheet)(void *ctx, uint32_t session, uint8_t *out, size_t len);
    /* reads. As tile_sheet. */
    int64_t (*palette_swatch)(void *ctx, uint32_t session, uint8_t *out, size_t len);
    /* reads. The sample rate as a little-endian uint32, then the buffered samples as little-endian int16. */
    int64_t (*audio_peek)(void *ctx, uint32_t session, uint8_t *out, size_t len);

    /* -- States ----------------------------------------------------------------------------------------------- */
    /* changes. An absent path is state_default_path's. */
    int32_t (*state_save)(void *ctx, uint32_t session, const uint8_t *path, size_t path_len);
    /* changes. */
    int32_t (*state_load)(void *ctx, uint32_t session, const uint8_t *path, size_t path_len);
    /* reads. */
    int64_t (*state_default_path)(void *ctx, uint32_t session, uint8_t *out, size_t len);

    /* -- Logs: the emulator's log writer's switch, the process's and no session's -------------------------- */
    /* changes. */
    int32_t (*log_switch)(void *ctx, uint32_t on);
    /* reads. 1 when on, 0 when off. */
    int32_t (*log_status)(void *ctx);

    /* -- Events -------------------------------------------------------------------------------------------- */
    /* reads. [event], each with its "kind"; removed once a buffer has taken them all, left waiting by a short one. */
    int64_t (*events_drain)(void *ctx, uint32_t session, uint32_t kinds, uint8_t *out, size_t len);
} emusen_session_v1;

#ifdef __cplusplus
}
#endif

#endif /* EMUSEN_SESSION_H */
