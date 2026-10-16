/* dianash.h - DianaSh's host API, version 1.0: how a program embeds the shell, feeds it lines, takes what they
 * printed and asked, keeps its sessions, and provides its own commands. Exported by emusen_platform. See
 * EmuSen_DianaSh.md §10, §11 and §13.7.
 *
 * Normative, and stable from version 1.0 (§16 Q17): the major is expected never to change, a minor only adds, and
 * Platform/dianash/abi/v1/baseline.txt records every fact a minor released. The rules are dianash_module.h's and
 * the platform library's: the caller's buffers, the length-query idiom, nothing kept that was passed in, signed
 * statuses with the words in dianash_last_error on the calling thread, and nothing that unwinds. A shell is used by
 * one thread at a time: another thread's call waits, and a call made from inside one of the shell's own commands, on
 * the thread running it, is refused with DIANASH_BUSY. dianash_cancel may be called from any thread. A session is
 * named by its id, 0 being the current one. */
#ifndef DIANASH_H
#define DIANASH_H

#include "dianash_module.h"

#ifdef __cplusplus
extern "C" {
#endif

#define DIANASH_ABI_MAJOR 1u
#define DIANASH_ABI_MINOR 0u
#define DIANASH_ABI_VERSION ((DIANASH_ABI_MAJOR << 16) | DIANASH_ABI_MINOR)

/* What dianash_submit answers when the line ran. */
#define DIANASH_DONE 0
#define DIANASH_NEEDS_MORE 1           /* a block, a quote or a $(...) is open: the next line continues it */
#define DIANASH_EXIT 2                 /* the line ended the shell: shutdown, quit */

/* dianash_config.flags */
#define DIANASH_CONFIG_SESSIONS (1u << 0)      /* keep named sessions (tmux); without it the shell is one session with no manager */
#define DIANASH_CONFIG_INTERACTIVE (1u << 1)   /* the shell runs at a terminal, which a console window is not */

typedef struct dianash_shell dianash_shell;

typedef struct dianash_config {
    uint32_t size;
    uint32_t flags;
    dianash_bytes root;            /* the sandbox's real root; absent: Galaxia's home, read on every use */
    uint32_t budget_ms;            /* how long a line's loops may run; 0: ten seconds */
} dianash_config;

/* A host's result for one of its own commands: a module's, and a request of the host's own (§13.7). The shell gives
 * a host command a result of this size; request 0 is none, and any other value returns from dianash_requests_drain
 * as {"kind":"host","value":N} for the line that raised it last. */
typedef struct dianash_host_result {
    uint32_t size;
    int32_t exit_status;
    uint64_t request;
} dianash_host_result;

/* The commands a host provides in-process (§10.3): a manifest, the same format as a module's with "kind":"host",
 * whose "commands" each take the place of a command of the same name but the language's own (help, man, export,
 * unset, source, .), and the functions that serve them, called as a module's are with ctx first. run and kill are
 * given a dianash_host_result, its size in its first field. The manifest may also give "aliases", "jobs" (the id
 * prefixes ps lists and kill removes through jobs and kill), "order" (help's listing, by name) and "notes" (lines
 * help prints before its own last three). The pointers must stay valid as long as the shell. */
typedef struct dianash_host_commands {
    uint32_t size;
    dianash_bytes manifest;
    void *ctx;
    int32_t (*run)(void *ctx, const dianash_call *call, dianash_result *result);
    int32_t (*jobs)(void *ctx, const dianash_call *call);
    int32_t (*kill)(void *ctx, const dianash_call *call, const uint8_t *id, size_t id_len, dianash_result *result);
} dianash_host_commands;

uint32_t dianash_abi_version(void);
int64_t dianash_version_text(uint8_t *out, size_t len);
int64_t dianash_last_error(uint8_t *out, size_t len);

/* A shell, or null with the words in dianash_last_error; a null config is the defaults. */
dianash_shell *dianash_new(const dianash_config *config);
int32_t dianash_free(dianash_shell *sh);

int32_t dianash_add_commands(dianash_shell *sh, uint32_t session, const dianash_host_commands *commands);
/* The services copied into every call; each table must outlive the shell. */
int32_t dianash_set_services(dianash_shell *sh, const dianash_service *services, size_t count);

/* DIANASH_DONE, DIANASH_NEEDS_MORE or DIANASH_EXIT; never blocks on input. */
int32_t dianash_submit(dianash_shell *sh, uint32_t session, const uint8_t *line, size_t len);
/* What the last line printed, kept until a buffer takes all of it. */
int64_t dianash_output_take(dianash_shell *sh, uint8_t *out, size_t len);
/* What lines asked of the host since the last drain, as a JSON array of {"kind": "exit" | "continue" |
 * "session" (with "id" and "name") | "host" (with "value")}, removed once a buffer takes all of it. */
int64_t dianash_requests_drain(dianash_shell *sh, uint8_t *out, size_t len);
int32_t dianash_last_status(dianash_shell *sh, uint32_t session, int32_t *status);
/* 1 when the line is one read-only command a thread that must not wait may run (§6.5), else 0. */
int32_t dianash_is_read_only(dianash_shell *sh, uint32_t session, const uint8_t *line, size_t len);
/* 1 while the session holds an open block, else 0. */
int32_t dianash_awaiting(dianash_shell *sh, uint32_t session);
int32_t dianash_cancel(dianash_shell *sh);

int64_t dianash_prompt(dianash_shell *sh, uint32_t session, uint8_t *out, size_t len);
/* Entries, each ended by a NUL. */
int64_t dianash_history(dianash_shell *sh, uint32_t session, uint8_t *out, size_t len);
int64_t dianash_banner(dianash_shell *sh, uint8_t *out, size_t len);
/* JSON: [{"name", "kind" ("builtin" | "host"), "usage", "read_only"}], in help's order. */
int64_t dianash_commands(dianash_shell *sh, uint32_t session, uint8_t *out, size_t len);
int64_t dianash_user(dianash_shell *sh, uint32_t session, uint8_t *out, size_t len);
/* With no check that the account exists, as a host's rebuild restores it. */
int32_t dianash_set_user(dianash_shell *sh, uint32_t session, const uint8_t *name, size_t len);

/* JSON: [{"id", "name", "user", "current"}], the listed sessions in order. */
int64_t dianash_sessions(dianash_shell *sh, uint8_t *out, size_t len);
/* A new session, unlisted until attached, with the shell's own commands until the host adds its own; its id. */
int64_t dianash_session_new(dianash_shell *sh);
/* Lists a session under a name, or the next default one (session-N) when name is null, and makes it current. */
int32_t dianash_session_attach(dianash_shell *sh, uint32_t session, const uint8_t *name, size_t len);
int32_t dianash_session_switch(dianash_shell *sh, uint32_t session);
/* A listed session unless it is the last (DIANASH_LAST_SESSION), an unlisted one always. */
int32_t dianash_session_kill(dianash_shell *sh, uint32_t session);
/* An unlisted session takes a session's name, place and currency, and the old one ends: a host's rebuild. */
int32_t dianash_session_replace(dianash_shell *sh, uint32_t session, uint32_t replacement);

#ifdef __cplusplus
}
#endif

#endif
