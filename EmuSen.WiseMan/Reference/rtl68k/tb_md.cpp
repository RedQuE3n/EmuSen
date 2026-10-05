// A bench around Nuked-MD's whole board (md_board), run as a black box: a cartridge image from argv[1] answered on
// the cartridge pins, the two RAMs answered on theirs, and every cartridge read and 68000 RAM write printed with the
// master clock it began on. Nephrite_Native.md §10 is the method; mdboard.py builds the programs and reads the output.
#include "Vmd_board.h"
#include "verilated.h"
#include <cstdio>
#include <cstdlib>
#include <vector>

int main(int argc, char** argv) {
    Verilated::commandArgs(argc, argv);
    Verilated::assertOn(false);
    FILE* f = fopen(argv[1], "rb");
    std::vector<uint8_t> rom(0x400000, 0xFF);
    size_t n = fread(rom.data(), 1, rom.size(), f);
    fclose(f);
    (void)n;
    uint64_t cycles = argc > 2 ? strtoull(argv[2], nullptr, 10) : 2000000;
    uint64_t reset_len = getenv("TB_RESET") ? strtoull(getenv("TB_RESET"), nullptr, 10) : 20000;
    int asserted = getenv("TB_RESET_LEVEL") ? atoi(getenv("TB_RESET_LEVEL")) : 1;
    // TB_TRACE=from:to prints the RAM pins on every cycle in that window.
    uint64_t trace_from = UINT64_MAX, trace_to = 0;
    if (const char* tr = getenv("TB_TRACE")) sscanf(tr, "%llu:%llu", (unsigned long long*)&trace_from, (unsigned long long*)&trace_to);
    std::vector<uint16_t> ram(0x8000, 0);
    std::vector<uint8_t> zram(0x2000, 0);
    auto* b = new Vmd_board;
    b->ext_reset = asserted; b->reset_button = 0; b->ext_vres = 0; b->ext_zres = 0;
    b->M3 = 1; b->cart_m3_pause = 0; b->ext_dtack = 0; b->pal = 0; b->jap = 0; b->tmss_enable = 0; b->tmss_data = 0;
    b->PA_i = 0x7F; b->PB_i = 0x7F; b->PC_i = 0x7F; b->vdp_cramdot_dis = 0; b->ym2612_status_enable = 1;
    b->dma_68k_req = 0; b->dma_z80_req = 0;
    bool cart_was = false, ram_was = false;
    uint16_t held = 0xFFFF;
    int hold = 0, hold_len = getenv("TB_HOLD") ? atoi(getenv("TB_HOLD")) : 8;
    for (uint64_t t = 0; t < cycles; t++) {
        if (t == reset_len) b->ext_reset = !asserted;
        uint32_t ca = b->cart_address;
        bool cart = !b->cart_cs && !b->cart_oe;
        // A ROM keeps its data a little after OE rises: the last word held for `hold` MCLK2 cycles.
        if (cart) { held = (uint16_t)(rom[(2 * ca) & 0x3FFFFF] << 8 | rom[(2 * ca + 1) & 0x3FFFFF]); hold = hold_len; }
        else if (hold > 0) hold--;
        b->cart_data = held;
        b->cart_data_en = cart || hold > 0;
        b->ram_68k_o = ram[b->ram_68k_address & 0x7FFF];
        b->ram_z80_o = zram[b->ram_z80_address & 0x1FFF];
        b->MCLK2 = 0; b->eval();
        b->MCLK2 = 1; b->eval();
        if (t >= trace_from && t < trace_to) printf("t %llu wren %d addr %05x be %d data %04x\n", (unsigned long long)t, b->ram_68k_wren, 2 * (b->ram_68k_address & 0x7FFF), b->ram_68k_byteena, b->ram_68k_data);
        if (b->ram_68k_wren) {
            uint16_t& w = ram[b->ram_68k_address & 0x7FFF];
            if (b->ram_68k_byteena & 2) w = (w & 0x00FF) | (b->ram_68k_data & 0xFF00);
            if (b->ram_68k_byteena & 1) w = (w & 0xFF00) | (b->ram_68k_data & 0x00FF);
            if (!ram_was) printf("w %llu %05x %04x\n", (unsigned long long)t, 2 * (b->ram_68k_address & 0x7FFF), b->ram_68k_data);
        }
        ram_was = b->ram_68k_wren;
        if (b->ram_z80_wren) zram[b->ram_z80_address & 0x1FFF] = b->ram_z80_data;
        if (cart && !cart_was) printf("c %llu %06x\n", (unsigned long long)t, 2 * ca);
        cart_was = cart;
    }
    delete b;
    return 0;
}
