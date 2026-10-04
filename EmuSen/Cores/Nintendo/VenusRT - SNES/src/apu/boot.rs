//! VenusRT's own SPC700 boot program at $FFC0-$FFFF, written from fullsnes's prose of the transfer protocol, not
//! from the console's 64 bytes, which VenusRT neither ships nor reads (VenusRT_Disputes.md, D-38).

/// The program, an instruction a line, offsets from $FFC0.
#[rustfmt::skip]
pub const BOOT: [u8; 64] = [
    0xCD, 0xEF,       // 00 MOV X,#$EF        blargg's SPC tests read $CD here (D-38)
    0xE8, 0x00,       // 02 MOV A,#$00
    0xBD,             // 04 MOV SP,X          SP = $EF, which gilyon's spctest expects
    0xD4, 0x00,       // 05 MOV $00+X,A       clear $EF down to $01
    0x1D,             // 07 DEC X
    0xD0, 0xFB,       // 08 BNE $05
    0xE8, 0xAA,       // 0A MOV A,#$AA
    0x8D, 0xBB,       // 0C MOV Y,#$BB
    0xDA, 0xF4,       // 0E MOVW $F4,YA       ready: $BBAA
    0xE8, 0xCC,       // 10 MOV A,#$CC
    0x2E, 0xF4, 0xFD, // 12 CBNE $F4,$12      the first kick
    0xBA, 0xF6,       // 15 MOVW YA,$F6       command: the address
    0xDA, 0x00,       // 17 MOVW $00,YA
    0xE4, 0xF4,       // 19 MOV A,$F4         the kick
    0xF8, 0xF5,       // 1B MOV X,$F5         the command byte
    0xC4, 0xF4,       // 1D MOV $F4,A         echo the kick
    0xD0, 0x03,       // 1F BNE $24
    0x1F, 0x00, 0x00, // 21 JMP [!$0000+X]    command zero: jump
    0x8D, 0x00,       // 24 MOV Y,#$00
    0x64, 0xF4,       // 26 CMP A,$F4         the kick still in port 0: wait
    0xF0, 0xFC,       // 28 BEQ $26
    0x7E, 0xF4,       // 2A CMP Y,$F4         one read: Y - port
    0x30, 0xE7,       // 2C BMI $15           port 1-127 past the index: the next command
    0xD0, 0xFA,       // 2E BNE $2A           behind it: wait
    0xE4, 0xF5,       // 30 MOV A,$F5         the index: data
    0xCC, 0xF4, 0x00, // 32 MOV !$00F4,Y      echo, then store
    0xD7, 0x00,       // 35 MOV [$00]+Y,A
    0xFC,             // 37 INC Y
    0xD0, 0xF0,       // 38 BNE $2A
    0xAB, 0x01,       // 3A INC $01
    0x2F, 0xEC,       // 3C BRA $2A
    0xC0, 0xFF,       // 3E reset vector
];

#[cfg(test)]
mod tests {
    use super::BOOT;
    use crate::apu::smp::Smp;

    // Each branch lands on an instruction of its own listing, and the reset vector is $FFC0.
    #[test]
    fn the_branches_land_where_the_listing_says() {
        let rel = |at: usize| (at as i32 + 2 + BOOT[at + 1] as i8 as i32) as usize;
        assert_eq!([rel(0x08), rel(0x1F), rel(0x28), rel(0x2C), rel(0x2E), rel(0x38), rel(0x3C)], [0x05, 0x24, 0x26, 0x15, 0x2A, 0x2A, 0x2A]);
        assert_eq!(3 + 0x12 + BOOT[0x14] as i8 as i32, 0x12);
        assert_eq!(u16::from_le_bytes([BOOT[62], BOOT[63]]), 0xFFC0);
    }

    /// Runs the S-SMP to `clock` master clocks, then lets the S-CPU write `port` at that clock.
    fn write(s: &mut Smp, clock: &mut u64, port: usize, value: u8) {
        *clock += 200;
        s.run_to(*clock);
        s.cpu_write(port, value, *clock);
    }

    /// Runs on until port 0 reads `value`, as an S-CPU polling it would, and fails after a frame's worth of polls.
    fn await_port0(s: &mut Smp, clock: &mut u64, value: u8) {
        for _ in 0..2_000 {
            *clock += 200;
            s.run_to(*clock);
            if s.cpu_read(0, *clock) == value {
                return;
            }
        }
        panic!("port 0 never read {value:02X}");
    }

    // D-38: the protocol of fullsnes's uploader, from the S-CPU's side: $BBAA with $02-$EF cleared, the $CC kick, two
    // blocks the second one zero bytes long, then the jump with [0000h..0001h] the entry point; the program run there
    // writes $5A to port 3.
    #[test]
    fn an_upload_of_two_blocks_and_a_jump_runs_through_the_ports_alone() {
        let mut s = Smp::new(false);
        s.ram[0x02..0xF0].fill(0x77);
        let mut clock = 0;
        await_port0(&mut s, &mut clock, 0xAA);
        clock += 200;
        s.run_to(clock);
        assert_eq!(s.cpu_read(1, clock), 0xBB);
        assert!(s.ram[0x02..0xF0].iter().all(|&b| b == 0));
        let program = [0x8F, 0x5A, 0xF7, 0x2F, 0xFE]; // MOV $F7,#$5A; BRA *
        let mut kick = 0xCC;
        for (address, block) in [(0x0300u16, &program[..]), (0x0400, &[][..])] {
            write(&mut s, &mut clock, 2, address as u8);
            write(&mut s, &mut clock, 3, (address >> 8) as u8);
            write(&mut s, &mut clock, 1, 1);
            write(&mut s, &mut clock, 0, kick);
            await_port0(&mut s, &mut clock, kick);
            for (i, &b) in block.iter().enumerate() {
                write(&mut s, &mut clock, 1, b);
                write(&mut s, &mut clock, 0, i as u8);
                await_port0(&mut s, &mut clock, i as u8);
            }
            kick = ((block.len() as u8).wrapping_add(2)) | 1;
        }
        assert_eq!(&s.ram[0x300..0x305], &program);
        write(&mut s, &mut clock, 2, 0x00);
        write(&mut s, &mut clock, 3, 0x03);
        write(&mut s, &mut clock, 1, 0);
        write(&mut s, &mut clock, 0, kick);
        await_port0(&mut s, &mut clock, kick);
        clock += 2_000;
        s.run_to(clock);
        assert_eq!((s.cpu_read(3, clock), s.ram[0], s.ram[1], s.cpu.sp), (0x5A, 0x00, 0x03, 0xEF));
    }

    // A block of 300 bytes carries its index past $FF into the address's high byte.
    #[test]
    fn a_block_longer_than_256_bytes_lands_whole() {
        let mut s = Smp::new(false);
        let mut clock = 0;
        await_port0(&mut s, &mut clock, 0xAA);
        write(&mut s, &mut clock, 2, 0x80);
        write(&mut s, &mut clock, 3, 0x02);
        write(&mut s, &mut clock, 1, 1);
        write(&mut s, &mut clock, 0, 0xCC);
        await_port0(&mut s, &mut clock, 0xCC);
        for i in 0..300u32 {
            write(&mut s, &mut clock, 1, (i * 7) as u8);
            write(&mut s, &mut clock, 0, i as u8);
            await_port0(&mut s, &mut clock, i as u8);
        }
        clock += 200;
        s.run_to(clock);
        assert!((0..300).all(|i| s.ram[0x280 + i] == (i * 7) as u8));
    }

    // D-38: Y - port decides: a value behind the index is ignored, and a kick 1-127 past it, here blargg's last echo
    // plus 2, is the next command.
    #[test]
    fn a_value_behind_the_index_waits_and_one_past_it_commands() {
        let mut s = Smp::new(false);
        let mut clock = 0;
        await_port0(&mut s, &mut clock, 0xAA);
        for (port, value) in [(2, 0x00), (3, 0x03), (1, 1), (0, 0xCC)] {
            write(&mut s, &mut clock, port, value);
        }
        await_port0(&mut s, &mut clock, 0xCC);
        for (i, b) in [0x8F, 0x5A, 0xF7, 0x2F, 0xFE].into_iter().enumerate() {
            write(&mut s, &mut clock, 1, b);
            write(&mut s, &mut clock, 0, i as u8);
            await_port0(&mut s, &mut clock, i as u8);
        }
        write(&mut s, &mut clock, 0, 2);
        for _ in 0..20 {
            clock += 200;
            s.run_to(clock);
        }
        assert_eq!((s.cpu.y, s.ram[0x305]), (5, 0), "a value behind the index was taken");
        for (port, value) in [(2, 0x00), (3, 0x03), (1, 0), (0, 4 + 2)] {
            write(&mut s, &mut clock, port, value);
        }
        clock += 2_000;
        s.run_to(clock);
        assert_eq!(s.cpu_read(3, clock), 0x5A);
    }
}
