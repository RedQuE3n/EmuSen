// A bench around a referee 68000, run as a black box: fx68k (the MegaCD MiSTer core's) by default, Nuked-MD's m68kcpu
// with -DNUKED. Each case on stdin is set up by a boot program, its instruction run, and its bus cycles and, on request,
// its final registers written to stdout. The case and output formats are referee.py's; Beryl_M68k.md section 6.

#include "verilated.h"
#ifdef NUKED
#include "Vm68kcpu.h"
#else
#include "Vfx68k.h"
#endif

#include <cstdint>
#include <cstdio>
#include <cstdlib>
#include <iostream>
#include <map>
#include <sstream>
#include <string>
#include <type_traits>
#include <vector>

struct Case {
    std::string id;
    uint32_t d[8], a[7], usp, ssp, sr, pc, ir = 0, irc = 0;
    std::map<uint32_t, uint8_t> ram;
    uint32_t window = 0;
    int64_t probe = -1;
};

struct Cycle {
    int64_t fall;  // half-clock of AS's assertion
    bool write;
    int fc;
    uint32_t address;
    bool word;
    uint32_t value;
    bool strobed;  // a data strobe asserted; without one the cycle is the abandoned cycle of an address error
};

static int64_t half = 0;
#ifdef NUKED
static const int RESET_HALVES = 400;
#else
static const int RESET_HALVES = 40;
#endif

#ifdef NUKED
// Nuked-MD's core samples its pins on a fast master clock; seven master clocks to each half of the 68000's clock.
static Vm68kcpu* cpu;
static bool clk_level = false;
static void tick(bool) {
    clk_level = !clk_level;
    cpu->CLK = clk_level;
    for (int i = 0; i < 7; i++) { cpu->MCLK = 0; cpu->eval(); cpu->MCLK = 1; cpu->eval(); }
    half++;
}
static void pins_idle() { cpu->VPA = 1; cpu->BR = 1; cpu->BGACK = 1; cpu->BERR = 1; cpu->IPL = 7; cpu->DTACK = 1; }
static void hold_reset(bool on) { cpu->RESET_i = !on; cpu->HALT_i = !on; }
static bool as_() { return !cpu->AS && !cpu->strobe_z; }
static bool uds_() { return !cpu->UDS && !cpu->strobe_z; }
static bool lds_() { return !cpu->LDS && !cpu->strobe_z; }
static uint32_t addr_() { return (uint32_t)cpu->ADDRESS << 1; }
static bool writing_() { return !cpu->RW; }
static int fc_() { return cpu->FC; }
static uint32_t data_out() { return cpu->DATA_o; }
static void data_in(uint16_t v) { cpu->DATA_i = v; }
static void dtack(bool on) { cpu->DTACK = !on; }
static bool halted_() { return cpu->HALT_pull; }
#else
// fx68k: one half of the nominal clock is two of the core's clocks, its enable for the next phase pulsed on the first.
static Vfx68k* cpu;
static void tick(bool phi1) {
    cpu->enPhi1 = phi1;
    cpu->enPhi2 = !phi1;
    cpu->clk = 0;
    cpu->eval();
    cpu->clk = 1;
    cpu->eval();
    cpu->enPhi1 = 0;
    cpu->enPhi2 = 0;
    cpu->clk = 0;
    cpu->eval();
    cpu->clk = 1;
    cpu->eval();
    half++;
}
static void pins_idle() {
    cpu->HALTn = 1; cpu->DTACKn = 1; cpu->VPAn = 1; cpu->BERRn = 1; cpu->BRn = 1; cpu->BGACKn = 1;
    cpu->IPL0n = 1; cpu->IPL1n = 1; cpu->IPL2n = 1;
}
static void hold_reset(bool on) { cpu->extReset = on; cpu->pwrUp = on; }
static bool as_() { return !cpu->ASn; }
static bool uds_() { return !cpu->UDSn; }
static bool lds_() { return !cpu->LDSn; }
static uint32_t addr_() { return (uint32_t)cpu->eab << 1; }
static bool writing_() { return !cpu->eRWn; }
static int fc_() { return cpu->FC2 << 2 | cpu->FC1 << 1 | cpu->FC0; }
static uint32_t data_out() { return cpu->oEdb; }
static void data_in(uint16_t v) { cpu->iEdb = v; }
static void dtack(bool on) { cpu->DTACKn = !on; }
static bool halted_() { return !cpu->oHALTEDn; }
#endif

static uint32_t rd32(const std::map<uint32_t, uint8_t>& m, uint32_t a) {
    auto b = [&](uint32_t x) { auto it = m.find(x & 0xFFFFFF); return it == m.end() ? 0u : (uint32_t)it->second; };
    return b(a) << 24 | b(a + 1) << 16 | b(a + 2) << 8 | b(a + 3);
}

static bool used(const Case& c, uint32_t lo, uint32_t hi) {
    auto it = c.ram.lower_bound(lo & 0xFFFFFF);
    return it != c.ram.end() && it->first <= (hi & 0xFFFFFF);
}

static void run(Case& c, bool probing) {
    std::map<uint32_t, uint8_t> mem = c.ram, boot;
    auto put16 = [](std::map<uint32_t, uint8_t>& m, uint32_t a, uint32_t v) { m[a & 0xFFFFFF] = v >> 8; m[(a + 1) & 0xFFFFFF] = v; };
    auto put32 = [&](std::map<uint32_t, uint8_t>& m, uint32_t a, uint32_t v) { put16(m, a, v >> 16); put16(m, a + 2, v); };
    // The boot program and its register table away from every byte the case names and from its stack frame.
    uint32_t frame = (c.ssp - 6) & 0xFFFFFF;
    uint32_t p = 0;
    for (uint32_t cand : {0x7F0000u, 0x5A0000u, 0x2E0000u, 0x930000u, 0xC40000u, 0x110000u}) {
        if (!used(c, cand - 0x100, cand + 0x200) && (frame + 0x200 < cand || frame > cand + 0x200)) { p = cand; break; }
    }
    uint32_t dump = 0;
    for (uint32_t cand : {0xE80000u, 0x3C0000u, 0x620000u, 0xA70000u}) {
        if (!used(c, cand - 0x100, cand + 0x100) && cand != p && (c.ssp + 0x100 < cand || c.ssp > cand + 0x200)) { dump = cand; break; }
    }
    if (!p || !dump || (c.ssp & 1) || (c.pc & 1)) { printf("case %s\nskipped\ndone\n", c.id.c_str()); return; }
    uint32_t table = p + 0x40;
    put32(boot, 0, frame);
    put32(boot, 4, p);
    uint16_t code[] = {0x207C, (uint16_t)(c.usp >> 16), (uint16_t)c.usp, 0x4E60, 0x4CF9, 0x7FFF, (uint16_t)(table >> 16), (uint16_t)table, 0x4E73};
    for (unsigned i = 0; i < sizeof code / 2; i++) put16(boot, p + 2 * i, code[i]);
    for (int i = 0; i < 8; i++) put32(boot, table + 4 * i, c.d[i]);
    for (int i = 0; i < 7; i++) put32(boot, table + 32 + 4 * i, c.a[i]);
    put16(boot, frame, c.sr);
    put32(boot, frame + 2, c.pc);
    put16(boot, c.pc, c.ir);  // the case's prefetch queue, which RTE's two prefetches load; memory keeps the case's own words
    put16(boot, c.pc + 2, c.irc);
    if (probing && c.probe >= 0) {
        uint16_t pr[] = {0x48F9, 0xFFFF, (uint16_t)(dump >> 16), (uint16_t)dump, 0x40F9, (uint16_t)(dump >> 16), (uint16_t)(dump + 0x40)};
        for (unsigned i = 0; i < 7; i++) put16(mem, c.probe + 2 * i, pr[i]);
        if (((c.probe - c.pc) & 0xFFFFFF) == 2) put16(boot, c.probe, pr[0]);  // the next opcode, already in the queue
    }

    pins_idle();
    hold_reset(true);
    for (int i = 0; i < RESET_HALVES; i++) tick(i & 1);
    hold_reset(false);

    bool booting = true, saw_frame = false;
    int64_t start = -1, limit = 4000000;
    std::vector<Cycle> cycles;
    bool in = false;
    Cycle cur{};
    int64_t guard = 0;
    std::map<uint32_t, uint8_t> written;
    auto finish = [&](const Cycle& cur) {
        if (cur.write && cur.strobed) {
            auto& dst = booting ? boot : mem;
            if (cur.word) { dst[cur.address & 0xFFFFFF] = cur.value >> 8; dst[(cur.address + 1) & 0xFFFFFF] = cur.value; }
            else dst[cur.address & 0xFFFFFF] = cur.value;
            if (!booting) { written[cur.address & 0xFFFFFF] = 1; if (cur.word) written[(cur.address + 1) & 0xFFFFFF] = 1; }
        }
        if (booting) {
            if (!cur.write && cur.address == ((frame + 4) & 0xFFFFFF)) saw_frame = true;
            if (saw_frame && !cur.write && (cur.fc & 3) == 2 && cur.address == ((c.pc + 2) & 0xFFFFFF)) {
                booting = false;
                start = cur.fall + 6;
            }
        } else {
            cycles.push_back(cur);
        }
    };
    while (guard++ < limit) {
        tick(half & 1);
        bool as = as_();
        uint32_t addr = addr_();
        bool uds = uds_(), lds = lds_();
        auto& m = booting ? boot : mem;
        auto get = [&](uint32_t x) -> uint32_t {
            if (booting) { auto it = boot.find(x); if (it != boot.end()) return it->second; }
            auto it = mem.find(x); return it == mem.end() ? 0u : it->second;
        };
        (void)m;
        data_in((uint16_t)(get(addr) << 8 | get(addr + 1)));
        dtack(as);
        bool turned = in && as && writing_() != cur.write;  // TAS: AS held from the read through the write
        if (in && (!as || turned)) {
            in = false;
            finish(cur);
        }
        if (as && !in) {
            in = true;
            cur = Cycle{half, writing_(), fc_(), addr, true, 0, false};
        }
        if (in && as && (uds || lds)) {
            cur.strobed = true;
            cur.word = uds && lds;
            cur.address = addr | (lds && !uds ? 1 : 0);
            if (cur.write) {
                uint32_t v = data_out();
                cur.value = cur.word ? v : (uds ? v >> 8 : v & 0xFF);
            } else {
                uint32_t v = get(addr) << 8 | get(addr + 1);
                cur.value = cur.word ? v : (uds ? v >> 8 : v & 0xFF);
            }
        }
        if (halted_() && !booting) { break; }
        if (!booting && !probing && half > start + 2 * (int64_t)c.window + 64) break;
        if (probing && !booting && (written.count(dump + 0x41) || half > start + 2 * (int64_t)c.window + 4000)) break;
    }
    printf("case %s\n", c.id.c_str());
    if (booting) { printf("unbooted\ndone\n"); return; }
    if (!probing) {
        for (auto& y : cycles) {
            int64_t rel = (y.fall - 2 - start);
            if (rel >= 2 * (int64_t)c.window + 16) break;
            printf("tx %lld %c%s %d %u %c %u\n", (long long)rel, y.write ? 'w' : 'r', y.strobed ? "" : "e", y.fc, y.address, y.word ? 'w' : 'b', y.value);
        }
        if (halted_()) printf("halted\n");
    } else if (getenv("TB_TRACE")) {
        for (auto& y : cycles) printf("ptx %lld %c %d %u %c %u\n", (long long)(y.fall - 2 - start), y.write ? 'w' : 'r', y.fc, y.address, y.word ? 'w' : 'b', y.value);
    }
    if (probing) {  // the instruction ends where the probe's first prefetch, a read of its third word, begins
        for (auto& y : cycles) if (!y.write && y.strobed && y.address == ((c.probe + 4) & 0xFFFFFF)) { printf("next %lld\n", (long long)(y.fall - 2 - start)); break; }
    }
    if (probing && written.count(dump + 0x41)) {
        printf("dump");
        for (int i = 0; i < 16; i++) printf(" %u", rd32(mem, dump + 4 * i));
        uint32_t sr = (uint32_t)mem[dump + 0x40] << 8 | mem[dump + 0x41];
        printf(" %u\n", sr);
    } else if (probing) {
        printf("nodump\n");
    }
    printf("done\n");
}

int main(int argc, char** argv) {
    Verilated::commandArgs(argc, argv);
    Verilated::assertOn(false);  // the ALU's unique-case checks fire on the reset state, before any microcode runs
    cpu = new std::remove_pointer<decltype(cpu)>::type;
    std::string line;
    Case c;
    bool probing = false;
    while (std::getline(std::cin, line)) {
        std::istringstream s(line);
        std::string k;
        s >> k;
        if (k == "case") { c = Case(); s >> c.id; }
        else if (k == "regs") {
            for (auto& x : c.d) s >> x;
            for (auto& x : c.a) s >> x;
            s >> c.usp >> c.ssp >> c.sr >> c.pc >> c.ir >> c.irc;
        } else if (k == "ram") { uint32_t a, v; while (s >> a >> v) c.ram[a & 0xFFFFFF] = v; }
        else if (k == "window") s >> c.window;
        else if (k == "probe") { s >> c.probe; probing = c.probe >= 0; }
        else if (k == "end") { run(c, probing); fflush(stdout); probing = false; }
    }
    delete cpu;
    return 0;
}
