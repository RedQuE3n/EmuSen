/* The bench for the YM2413 referee (Endou_Opll.md section 7): Nuked-OPLL built as a black box and driven through its
   header alone. Reads voices from stdin, each "voice NAME SAMPLES" then "w SAMPLE ADDRESS DATA" lines and "end", as
   Endou's examples/opllref.rs prints them; writes "NAME" then one line a sample: the sum of the chip's two outputs over
   the sample's calls, from the call at cycle counter 5 (OPLL_FIRST_CYCLE). The calls a sample are printed as "calls N". */
#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include "opll.h"

static opll_t chip;
static int data_call = 6, first_cycle = 5;

/* A write inside a sample's own calls: the address four calls (16 master clocks) before the data, the data at call 6
   (OPLL_DATA_CALL), where every channel takes it at the next sample (Endou_Opll.md section 7). The voices space their
   writes two samples apart. */
static long sample_with_write(int address, int data, int calls) {
    int32_t buf[2];
    long sum = 0;
    for (int i = 0; i < calls; i++) {
        if (i == data_call - 4 && address >= 0) OPLL_Write(&chip, 0, (uint8_t)address);
        if (i == data_call && address >= 0) OPLL_Write(&chip, 1, (uint8_t)data);
        OPLL_Clock(&chip, buf);
        sum += buf[0] + buf[1];
    }
    return sum;
}

/* Calls of OPLL_Clock a sample: the header's cycle counter runs from 0 to its last value and back, one a call. The
   bench then starts each sample's calls at counter 5, so that every channel's turn falls in the sample it belongs to. */
static int calibrate(void) {
    int32_t buf[2];
    uint32_t top = 0;
    OPLL_Reset(&chip, opll_type_ym2413);
    for (int i = 0; i < 1000; i++) { OPLL_Clock(&chip, buf); if (chip.cycles > top) top = chip.cycles; }
    return (int)top + 1;
}

int main(void) {
    if (getenv("OPLL_DATA_CALL")) data_call = atoi(getenv("OPLL_DATA_CALL"));
    if (getenv("OPLL_FIRST_CYCLE")) first_cycle = atoi(getenv("OPLL_FIRST_CYCLE"));
    int calls = calibrate();
    printf("calls %d\n", calls);
    char line[256], name[128];
    long samples = 0;
    long (*w)[3] = NULL;
    size_t nw = 0, cap = 0;
    while (fgets(line, sizeof line, stdin)) {
        long a, b, c;
        if (sscanf(line, "voice %127s %ld", name, &samples) == 2) { nw = 0; continue; }
        if (sscanf(line, "w %ld %ld %ld", &a, &b, &c) == 3) {
            if (nw == cap) { cap = cap ? cap * 2 : 64; w = realloc(w, cap * sizeof *w); }
            w[nw][0] = a; w[nw][1] = b; w[nw][2] = c; nw++;
            continue;
        }
        if (strncmp(line, "end", 3) != 0) continue;
        OPLL_Reset(&chip, opll_type_ym2413);
        { int32_t buf[2]; while (chip.cycles != (uint32_t)((first_cycle + calls - 1) % calls)) OPLL_Clock(&chip, buf); }
        printf("%s\n", name);
        size_t k = 0;
        for (long s = 0; s < samples; s++) {
            int a = -1, d = 0;
            if (k < nw && w[k][0] <= s) { a = (int)w[k][1]; d = (int)w[k][2]; k++; }
            printf("%ld\n", sample_with_write(a, d, calls));
        }
    }
    free(w);
    return 0;
}
