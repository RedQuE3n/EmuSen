/* The bench for the YM2413 referee (Endou_Opll.md section 7): Nuked-OPLL built as a black box and driven through its
   header alone. Reads voices from stdin, each "voice NAME SAMPLES" then "w SAMPLE ADDRESS DATA" lines and "end", as
   Endou's examples/opllref.rs prints them; writes "NAME" then one line a sample: the sum of the chip's two outputs over
   the sample's calls. The calls a sample are found first from a sine of known frequency and printed as "calls N". */
#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include "opll.h"

static opll_t chip;

static void writes(int address, int data, int calls) {
    int32_t buf[2];
    OPLL_Write(&chip, 0, (uint8_t)address);
    for (int i = 0; i < calls / 6 + 1; i++) OPLL_Clock(&chip, buf);
    OPLL_Write(&chip, 1, (uint8_t)data);
}

/* Calls of OPLL_Clock a sample: a carrier sine at F-number 288, block 4, is 437.0 Hz at the chip's 49,716 samples a
   second, so its period in calls over its period in samples is the answer, rounded. */
static int calibrate(void) {
    const int setup[][2] = {{0, 0x21}, {1, 0x21}, {2, 0x3F}, {3, 0}, {4, 0xF0}, {5, 0xF0}, {6, 0x0F}, {7, 0x0F}, {0x30, 0}, {0x10, 0x20}, {0x20, 0x19}};
    int32_t buf[2];
    OPLL_Reset(&chip, opll_type_ym2413);
    for (unsigned i = 0; i < sizeof setup / sizeof setup[0]; i++) {
        OPLL_Write(&chip, 0, (uint8_t)setup[i][0]);
        for (int k = 0; k < 200; k++) OPLL_Clock(&chip, buf);
        OPLL_Write(&chip, 1, (uint8_t)setup[i][1]);
        for (int k = 0; k < 2000; k++) OPLL_Clock(&chip, buf);
    }
    long first = -1, last = -1, crossings = 0, prev = 0;
    for (long n = 0; n < 4000000; n++) {
        OPLL_Clock(&chip, buf);
        long v = buf[0] + buf[1];
        if (v == 0) continue;
        if (prev < 0 && v > 0) {
            if (first < 0) first = n; else { last = n; crossings++; }
        }
        prev = v;
    }
    double per_cycle = (double)(last - first) / crossings;
    return (int)(per_cycle / (49715.9 / 437.0) + 0.5);
}

int main(void) {
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
        printf("%s\n", name);
        size_t k = 0;
        int32_t buf[2];
        for (long s = 0; s < samples; s++) {
            while (k < nw && w[k][0] <= s) { writes((int)w[k][1], (int)w[k][2], calls); k++; }
            long sum = 0;
            for (int i = 0; i < calls; i++) { OPLL_Clock(&chip, buf); sum += buf[0] + buf[1]; }
            printf("%ld\n", sum);
        }
    }
    free(w);
    return 0;
}
