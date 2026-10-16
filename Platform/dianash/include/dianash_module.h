/* dianash_module.h - the command module ABI of DianaSh, version 1.0. See EmuSen_DianaSh.md §5, §6 and §13.7.
 *
 * Normative. A command is called with a dianash_call and answers with a status and a dianash_result; everything it
 * may ask of the shell is in the io table it is handed, and it may call that table only before its call returns,
 * on the thread that called it, and keep no pointer it was given past that (§6.0). Every struct begins with its
 * size; a callee reads no field past the size it was given, and a later minor only appends (§11.2). Text is UTF-8
 * by pointer and length; a null pointer with a zero length is the absent string. The baseline of this header and
 * of dianash.h is Platform/dianash/abi/v1/baseline.txt. */
#ifndef DIANASH_MODULE_H
#define DIANASH_MODULE_H

#include <stddef.h>
#include <stdint.h>

#ifdef __cplusplus
extern "C" {
#endif

#define DIANASH_MODULE_ABI_MAJOR 1u
#define DIANASH_MODULE_ABI_MINOR 0u
#define DIANASH_MODULE_ABI_VERSION ((DIANASH_MODULE_ABI_MAJOR << 16) | DIANASH_MODULE_ABI_MINOR)

/* Statuses: the project's shared codes, then the band -256 to -319, then the fault (§6.9). */
#define DIANASH_OK 0
#define DIANASH_NULL (-1)
#define DIANASH_BAD_STRING (-5)
#define DIANASH_BUFFER_TOO_SMALL (-7)
#define DIANASH_NOT_SUPPORTED (-256)
#define DIANASH_BAD_STRUCT (-263)
#define DIANASH_UNKNOWN_COMMAND (-265)
#define DIANASH_SERVICE_MISSING (-266)
#define DIANASH_BUSY (-267)
#define DIANASH_ABSENT (-268)          /* an answer, not a failure: no such variable, not one of this command's jobs */
#define DIANASH_EXISTS (-269)          /* a session of that name exists already */
#define DIANASH_NO_SUCH_SESSION (-270)
#define DIANASH_LAST_SESSION (-271)    /* the last listed session is never killed */
#define DIANASH_BAD_MANIFEST (-272)
#define DIANASH_OUTSIDE (-273)         /* a path above the sandbox's root; the words that refuse it are given in its place */
#define DIANASH_FAULT (-320)

/* dianash_call.flags */
#define DIANASH_CALL_READ_ONLY_CONTEXT (1u << 0)
#define DIANASH_CALL_INTERACTIVE (1u << 1)

/* resolve's flags: what the path is for. Version 1.0 resolves every kind alike. */
#define DIANASH_RESOLVE_READ (1u << 0)
#define DIANASH_RESOLVE_WRITE (1u << 1)
#define DIANASH_RESOLVE_REMOVE (1u << 2)

typedef struct dianash_bytes {
    const uint8_t *data;
    size_t len;
} dianash_bytes;

typedef struct dianash_terminal {
    uint32_t size;
    uint32_t columns;
    uint32_t rows;
    uint32_t raw_capable;          /* 0 in a console window, which is not a terminal (§12.3) */
} dianash_terminal;

/* What the shell gives a call (§6.3). Each function takes ctx first; each that answers with text uses the length-query
 * idiom: *needed is always the whole length, and out is written only when cap holds all of it, else BUFFER_TOO_SMALL. */
typedef struct dianash_io {
    uint32_t size;
    void *ctx;
    /* Appends to standard output and to standard error; the shell prints both in the order written. */
    int32_t (*out)(void *ctx, const uint8_t *data, size_t len);
    int32_t (*err)(void *ctx, const uint8_t *data, size_t len);
    /* A shell path to a real one through the sandbox; OUTSIDE, with the refusal's words in out, above the root. */
    int32_t (*resolve)(void *ctx, const uint8_t *path, size_t len, uint32_t flags, uint8_t *out, size_t cap, int64_t *needed);
    /* A real path back to the shell's "/" form. */
    int32_t (*display)(void *ctx, const uint8_t *real, size_t len, uint8_t *out, size_t cap, int64_t *needed);
    /* 1 once the player cancelled or the line's budget is spent; poll it in any loop that may run long (§6.6). */
    int32_t (*cancelled)(void *ctx);
    /* The per-shell state: a null *state reads what is kept (null for none); a non-null one keeps it, with the release
     * the shell calls when that shell's session ends (§6.3). */
    int32_t (*slot)(void *ctx, void **state, void (**release)(void *state));
    /* An exported variable; ABSENT when it is not exported. */
    int32_t (*environment)(void *ctx, const uint8_t *name, size_t len, uint8_t *out, size_t cap, int64_t *needed);
    int32_t (*terminal)(void *ctx, dianash_terminal *out);
} dianash_io;

/* A service a host provides, copied into calls by name and version; the shell never reads table (§6.3, §9.8). */
typedef struct dianash_service {
    uint32_t size;
    dianash_bytes name;
    uint32_t major;
    uint32_t minor;
    const void *table;
} dianash_service;

typedef struct dianash_call {
    uint32_t size;
    dianash_bytes command;         /* the name it was called by, after alias mapping */
    const dianash_bytes *argv;     /* argv[0] is the name as typed */
    size_t argc;
    dianash_bytes stdin_data;
    uint32_t stdin_present;        /* 0: first in its pipeline or alone; 1: a stage before it ran, perhaps printing nothing */
    uint32_t flags;
    dianash_bytes cwd;             /* the working directory as the shell shows it, "/" rooted */
    dianash_bytes user;            /* the current account */
    const dianash_io *io;
    const dianash_service *services;
    size_t service_count;
} dianash_call;

typedef struct dianash_result {
    uint32_t size;
    int32_t exit_status;           /* $? */
} dianash_result;

/* A module's exports (§6.1). run returns OK when the command ran, whatever its exit status. */
uint32_t dianash_module_abi_version(void);
int64_t dianash_module_manifest(uint8_t *out, size_t len);
int32_t dianash_module_run(const dianash_call *call, dianash_result *result);
/* Optional, claimed by the manifest's "jobs": lines through call->io->out, each ended by a newline; a kill answers ABSENT for an id not its own. */
int32_t dianash_module_jobs(const dianash_call *call);
int32_t dianash_module_kill(const dianash_call *call, const uint8_t *id, size_t id_len, dianash_result *result);

#ifdef __cplusplus
}
#endif

#endif
