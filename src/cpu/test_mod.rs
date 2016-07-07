// Copyright 2016 Peter Beard
// Distributed under the GNU GPL v3. For full terms, see the LICENSE file.
use super::*;

// Run a program consisting of a single instruction
fn run_program(program: &[u8], cpu: &mut Cpu) {
    let mut ram: [u8; 65536] = [0u8; 65536];
    if program[0] == 0 {
        ram = [80u8; 65536];
    }

    // Write the program to the reset location
    for addr in 0..program.len() {
        ram[super::RESET_VECTOR_ADDR as usize + addr] = program[addr];
    }

    cpu.reset();

    loop {
        let addr = cpu.addr_bus as usize;

        if (cpu.pc < super::RESET_VECTOR_ADDR || cpu.pc >= super::RESET_VECTOR_ADDR + program.len() as u16) && cpu.state == super::CpuState::Fetch {
            break;
        }

        if cpu.rw {
            cpu.data_in(ram[addr]);
        } else {
            ram[addr] = cpu.data_out();
        }
        println!("{:?}", cpu);
        cpu.cycle(false);
        println!("{:?}", cpu);

        if cpu.cycles > 20 {
            break;
        }
    }
}
// Test cycle-accuracy of instructions
// ADC
#[test]
fn adc_imm_cycles() {
    let mut cpu = Cpu::new();

    let program = [0x69, 0x10];
    run_program(&program[..], &mut cpu);

    assert_eq!(2, cpu.cycles)
}

#[test]
fn adc_zp_cycles() {
    let mut cpu = Cpu::new();

    let program = [0x65, 0x00];
    run_program(&program[..], &mut cpu);

    assert_eq!(3, cpu.cycles)
}

#[test]
fn adc_zpx_cycles() {
    let mut cpu = Cpu::new();

    let program = [0x75, 0x00];
    run_program(&program[..], &mut cpu);

    assert_eq!(4, cpu.cycles)
}

#[test]
fn adc_abs_cycles() {
    let mut cpu = Cpu::new();

    let program = [0x6d, 0x00, 0x0f];
    run_program(&program[..], &mut cpu);

    assert_eq!(4, cpu.cycles)
}

#[test]
fn adc_absx_cycles() {
    let mut cpu = Cpu::new();

    let program = [0x7d, 0x00, 0x0f];
    run_program(&program[..], &mut cpu);

    assert_eq!(4, cpu.cycles)
}

#[test]
fn adc_absy_cycles() {
    let mut cpu = Cpu::new();

    let program = [0x79, 0x00, 0x0f];
    run_program(&program[..], &mut cpu);

    assert_eq!(4, cpu.cycles)
}

#[test]
fn adc_indx_cycles() {
    let mut cpu = Cpu::new();

    let program = [0x61, 0x00];
    run_program(&program[..], &mut cpu);

    assert_eq!(6, cpu.cycles)
}

#[test]
fn adc_indy_cycles() {
    let mut cpu = Cpu::new();

    let program = [0x71, 0x00];
    run_program(&program[..], &mut cpu);

    assert_eq!(5, cpu.cycles)
}

// AND
#[test]
fn and_imm_cycles() {
    let mut cpu = Cpu::new();

    let program = [0x29, 0x00];
    run_program(&program[..], &mut cpu);

    assert_eq!(2, cpu.cycles)
}

#[test]
fn and_zp_cycles() {
    let mut cpu = Cpu::new();

    let program = [0x25, 0x00];
    run_program(&program[..], &mut cpu);

    assert_eq!(3, cpu.cycles)
}

#[test]
fn and_zpx_cycles() {
    let mut cpu = Cpu::new();

    let program = [0x35, 0x00];
    run_program(&program[..], &mut cpu);

    assert_eq!(4, cpu.cycles)
}

#[test]
fn and_abs_cycles() {
    let mut cpu = Cpu::new();

    let program = [0x2d, 0x00, 0x0f];
    run_program(&program[..], &mut cpu);

    assert_eq!(4, cpu.cycles)
}

#[test]
fn and_absx_cycles() {
    let mut cpu = Cpu::new();

    let program = [0x3d, 0x00, 0x0f];
    run_program(&program[..], &mut cpu);

    assert_eq!(4, cpu.cycles)
}

#[test]
fn and_absy_cycles() {
    let mut cpu = Cpu::new();

    let program = [0x39, 0x00];
    run_program(&program[..], &mut cpu);

    assert_eq!(4, cpu.cycles)
}

#[test]
fn and_indx_cycles() {
    let mut cpu = Cpu::new();

    let program = [0x21, 0x00];
    run_program(&program[..], &mut cpu);

    assert_eq!(6, cpu.cycles)
}

#[test]
fn and_indy_cycles() {
    let mut cpu = Cpu::new();

    let program = [0x31, 0x00];
    run_program(&program[..], &mut cpu);

    assert_eq!(5, cpu.cycles)
}

// ASL
#[test]
fn asl_impl_cycles() {
    let mut cpu = Cpu::new();

    let program = [0x0a];
    run_program(&program[..], &mut cpu);

    assert_eq!(2, cpu.cycles);
}

#[test]
fn asl_zp_cycles() {
    let mut cpu = Cpu::new();

    let program = [0x06, 0x00];
    run_program(&program[..], &mut cpu);

    assert_eq!(5, cpu.cycles);
}

#[test]
fn asl_zpx_cycles() {
    let mut cpu = Cpu::new();

    let program = [0x16, 0x00];
    run_program(&program[..], &mut cpu);

    assert_eq!(6, cpu.cycles);
}

#[test]
fn asl_abs_cycles() {
    let mut cpu = Cpu::new();

    let program = [0x0e, 0x00, 0x0f];
    run_program(&program[..], &mut cpu);

    assert_eq!(6, cpu.cycles);
}

#[test]
fn asl_absx_cycles() {
    let mut cpu = Cpu::new();

    let program = [0x1e, 0x00, 0x0f];
    run_program(&program[..], &mut cpu);

    assert_eq!(7, cpu.cycles);
}

#[test]
fn bcc_test_cycles() {
    let mut cpu = Cpu::new();

    let program = [0x90, 0x0f];
    run_program(&program[..], &mut cpu);

    assert_eq!(2, cpu.cycles);
}

#[test]
fn bcs_test_cycles() {
    let mut cpu = Cpu::new();

    let program = [0xb0, 0x0f];
    run_program(&program[..], &mut cpu);

    assert_eq!(2, cpu.cycles);
}

#[test]
fn beq_test_cycles() {
    let mut cpu = Cpu::new();

    let program = [0xf0, 0x0f];
    run_program(&program[..], &mut cpu);

    assert_eq!(2, cpu.cycles);
}

#[test]
fn bit_zp_test_cycles() {
    let mut cpu = Cpu::new();

    let program = [0x24, 0x80];
    run_program(&program[..], &mut cpu);

    assert_eq!(3, cpu.cycles);
}

#[test]
fn bit_abs_test_cycles() {
    let mut cpu = Cpu::new();

    let program = [0x2c, 0x00, 0xf0];
    run_program(&program[..], &mut cpu);

    assert_eq!(4, cpu.cycles);
}

#[test]
fn bmi_test_cycles() {
    let mut cpu = Cpu::new();

    let program = [0x30, 0x0f];
    run_program(&program[..], &mut cpu);

    assert_eq!(2, cpu.cycles);
}

#[test]
fn bne_test_cycles() {
    let mut cpu = Cpu::new();

    let program = [0xd0, 0x0f];
    run_program(&program[..], &mut cpu);

    assert_eq!(2, cpu.cycles);
}

#[test]
fn bpl_test_cycles() {
    let mut cpu = Cpu::new();

    let program = [0x10, 0x0f];
    run_program(&program[..], &mut cpu);

    assert_eq!(2, cpu.cycles);
}

//#[test]
fn brk_test_cycles() {
    let mut cpu = Cpu::new();

    let program = [0x00];
    run_program(&program[..], &mut cpu);

    assert_eq!(7, cpu.cycles);
}

#[test]
fn bvc_test_cycles() {
    let mut cpu = Cpu::new();

    let program = [0x50, 0x0f];
    run_program(&program[..], &mut cpu);

    assert_eq!(2, cpu.cycles);
}

#[test]
fn bvs_test_cycles() {
    let mut cpu = Cpu::new();

    let program = [0x70, 0x0f];
    run_program(&program[..], &mut cpu);

    assert_eq!(2, cpu.cycles);
}

#[test]
fn clc_test_cycles() {
    let mut cpu = Cpu::new();

    let program = [0x18];
    run_program(&program[..], &mut cpu);

    assert_eq!(2, cpu.cycles);
}

#[test]
fn cld_test_cycles() {
    let mut cpu = Cpu::new();

    let program = [0xd8];
    run_program(&program[..], &mut cpu);

    assert_eq!(2, cpu.cycles);
}

#[test]
fn cli_test_cycles() {
    let mut cpu = Cpu::new();

    let program = [0x58];
    run_program(&program[..], &mut cpu);

    assert_eq!(2, cpu.cycles);
}

#[test]
fn clv_test_cycles() {
    let mut cpu = Cpu::new();

    let program = [0xb8];
    run_program(&program[..], &mut cpu);

    assert_eq!(2, cpu.cycles);
}

#[test]
fn cmp_imm_test_cycles() {
    let mut cpu = Cpu::new();

    let program = [0xc9, 0x00];
    run_program(&program[..], &mut cpu);

    assert_eq!(2, cpu.cycles);
}

#[test]
fn cmp_zp_test_cycles() {
    let mut cpu = Cpu::new();

    let program = [0xc5, 0x00];
    run_program(&program[..], &mut cpu);

    assert_eq!(3, cpu.cycles);
}

#[test]
fn cmp_zpx_test_cycles() {
    let mut cpu = Cpu::new();

    let program = [0xd5, 0x00];
    run_program(&program[..], &mut cpu);

    assert_eq!(4, cpu.cycles);
}

#[test]
fn cmp_abs_test_cycles() {
    let mut cpu = Cpu::new();

    let program = [0xcd, 0x00, 0xf0];
    run_program(&program[..], &mut cpu);

    assert_eq!(4, cpu.cycles);
}

#[test]
fn cmp_absx_test_cycles() {
    let mut cpu = Cpu::new();

    let program = [0xdd, 0x00, 0x0f];
    run_program(&program[..], &mut cpu);

    assert_eq!(4, cpu.cycles);
}

#[test]
fn cmp_absy_test_cycles() {
    let mut cpu = Cpu::new();

    let program = [0xd9, 0x00, 0x0f];
    run_program(&program[..], &mut cpu);

    assert_eq!(4, cpu.cycles);
}

#[test]
fn cmp_indx_test_cycles() {
    let mut cpu = Cpu::new();

    let program = [0xc1, 0x00];
    run_program(&program[..], &mut cpu);

    assert_eq!(6, cpu.cycles);
}

#[test]
fn cmp_indy_test_cycles() {
    let mut cpu = Cpu::new();

    let program = [0xd1, 0x00];
    run_program(&program[..], &mut cpu);

    assert_eq!(5, cpu.cycles);
}

#[test]
fn cpx_imm_test_cycles() {
    let mut cpu = Cpu::new();

    let program = [0xe0, 0x00];
    run_program(&program[..], &mut cpu);

    assert_eq!(2, cpu.cycles);
}

#[test]
fn cpx_zp_test_cycles() {
    let mut cpu = Cpu::new();

    let program = [0xe4, 0x00];
    run_program(&program[..], &mut cpu);

    assert_eq!(3, cpu.cycles);
}

#[test]
fn cpx_abs_test_cycles() {
    let mut cpu = Cpu::new();

    let program = [0xec, 0x00, 0x0f];
    run_program(&program[..], &mut cpu);

    assert_eq!(4, cpu.cycles);
}

#[test]
fn cpy_imm_test_cycles() {
    let mut cpu = Cpu::new();

    let program = [0xc0, 0x00];
    run_program(&program[..], &mut cpu);

    assert_eq!(2, cpu.cycles);
}

#[test]
fn cpy_zp_test_cycles() {
    let mut cpu = Cpu::new();

    let program = [0xc4, 0x00];
    run_program(&program[..], &mut cpu);

    assert_eq!(3, cpu.cycles);
}

#[test]
fn cpy_abs_test_cycles() {
    let mut cpu = Cpu::new();

    let program = [0xcc, 0x00, 0x0f];
    run_program(&program[..], &mut cpu);

    assert_eq!(4, cpu.cycles);
}

#[test]
fn dec_zp_test_cycles() {
    let mut cpu = Cpu::new();

    let program = [0xc6, 0x00];
    run_program(&program[..], &mut cpu);

    assert_eq!(5, cpu.cycles);
}

#[test]
fn dec_zpx_test_cycles() {
    let mut cpu = Cpu::new();

    let program = [0xd6, 0x00];
    run_program(&program[..], &mut cpu);

    assert_eq!(6, cpu.cycles);
}

#[test]
fn dec_abs_test_cycles() {
    let mut cpu = Cpu::new();

    let program = [0xce, 0x00, 0x0f];
    run_program(&program[..], &mut cpu);

    assert_eq!(6, cpu.cycles);
}

#[test]
fn dec_absx_test_cycles() {
    let mut cpu = Cpu::new();

    let program = [0xde, 0x00, 0x0f];
    run_program(&program[..], &mut cpu);

    assert_eq!(7, cpu.cycles);
}

#[test]
fn dex_test_cycles() {
    let mut cpu = Cpu::new();

    let program = [0xca];
    run_program(&program[..], &mut cpu);

    assert_eq!(2, cpu.cycles);
}

#[test]
fn dey_test_cycles() {
    let mut cpu = Cpu::new();

    let program = [0x88];
    run_program(&program[..], &mut cpu);

    assert_eq!(2, cpu.cycles);
}

#[test]
fn eor_imm_test_cycles() {
    let mut cpu = Cpu::new();

    let program = [0x49, 0x00];
    run_program(&program[..], &mut cpu);

    assert_eq!(2, cpu.cycles);
}

#[test]
fn eor_zp_test_cycles() {
    let mut cpu = Cpu::new();

    let program = [0x45, 0x00];
    run_program(&program[..], &mut cpu);

    assert_eq!(3, cpu.cycles);
}

#[test]
fn eor_zpx_test_cycles() {
    let mut cpu = Cpu::new();

    let program = [0x55, 0x00];
    run_program(&program[..], &mut cpu);

    assert_eq!(4, cpu.cycles);
}

#[test]
fn eor_abs_test_cycles() {
    let mut cpu = Cpu::new();

    let program = [0x4d, 0x00, 0x0f];
    run_program(&program[..], &mut cpu);

    assert_eq!(4, cpu.cycles);
}

#[test]
fn eor_absx_test_cycles() {
    let mut cpu = Cpu::new();

    let program = [0x5d, 0x00, 0x0f];
    run_program(&program[..], &mut cpu);

    assert_eq!(4, cpu.cycles);
}

#[test]
fn eor_absy_test_cycles() {
    let mut cpu = Cpu::new();

    let program = [0x59, 0x00, 0x0f];
    run_program(&program[..], &mut cpu);

    assert_eq!(4, cpu.cycles);
}

#[test]
fn eor_indx_test_cycles() {
    let mut cpu = Cpu::new();

    let program = [0x41, 0x00];
    run_program(&program[..], &mut cpu);

    assert_eq!(6, cpu.cycles);
}

#[test]
fn eor_indy_test_cycles() {
    let mut cpu = Cpu::new();

    let program = [0x51, 0x00];
    run_program(&program[..], &mut cpu);

    assert_eq!(5, cpu.cycles);
}

#[test]
fn inc_zp_test_cycles() {
    let mut cpu = Cpu::new();

    let program = [0xe6, 0x00];
    run_program(&program[..], &mut cpu);

    assert_eq!(5, cpu.cycles);
}

#[test]
fn inc_zpx_test_cycles() {
    let mut cpu = Cpu::new();

    let program = [0xf6, 0x00];
    run_program(&program[..], &mut cpu);

    assert_eq!(6, cpu.cycles);
}

#[test]
fn inc_abs_test_cycles() {
    let mut cpu = Cpu::new();

    let program = [0xee, 0x00, 0x0f];
    run_program(&program[..], &mut cpu);

    assert_eq!(6, cpu.cycles);
}

#[test]
fn inc_absx_test_cycles() {
    let mut cpu = Cpu::new();

    let program = [0xfe, 0x00, 0x0f];
    run_program(&program[..], &mut cpu);

    assert_eq!(7, cpu.cycles);
}

#[test]
fn inx_test_cycles() {
    let mut cpu = Cpu::new();

    let program = [0xe8];
    run_program(&program[..], &mut cpu);

    assert_eq!(2, cpu.cycles);
}

#[test]
fn iny_test_cycles() {
    let mut cpu = Cpu::new();

    let program = [0xc8];
    run_program(&program[..], &mut cpu);

    assert_eq!(2, cpu.cycles);
}

#[test]
fn jmp_abs_test_cycles() {
    let mut cpu = Cpu::new();

    let program = [0x4c, 0x00, 0x0f];
    run_program(&program[..], &mut cpu);

    assert_eq!(3, cpu.cycles);
}

#[test]
fn jmp_ind_test_cycles() {
    let mut cpu = Cpu::new();

    let program = [0x6c, 0x00, 0x0f];
    run_program(&program[..], &mut cpu);

    assert_eq!(5, cpu.cycles);
}

#[test]
fn jsr_test_cycles() {
    let mut cpu = Cpu::new();

    let program = [0x20, 0x00, 0x0f];
    run_program(&program[..], &mut cpu);

    assert_eq!(6, cpu.cycles);
}

#[test]
fn lda_imm_test_cycles() {
    let mut cpu = Cpu::new();

    let program = [0xa9, 0x00];
    run_program(&program[..], &mut cpu);

    assert_eq!(2, cpu.cycles);
}

#[test]
fn lda_zp_test_cycles() {
    let mut cpu = Cpu::new();

    let program = [0xa5, 0x00];
    run_program(&program[..], &mut cpu);

    assert_eq!(3, cpu.cycles);
}

#[test]
fn lda_zpx_test_cycles() {
    let mut cpu = Cpu::new();

    let program = [0xb5, 0x00];
    run_program(&program[..], &mut cpu);

    assert_eq!(4, cpu.cycles);
}

#[test]
fn lda_abs_test_cycles() {
    let mut cpu = Cpu::new();

    let program = [0xad, 0x00, 0x0f];
    run_program(&program[..], &mut cpu);

    assert_eq!(4, cpu.cycles);
}

#[test]
fn lda_absx_test_cycles() {
    let mut cpu = Cpu::new();

    let program = [0xbd, 0x00, 0x0f];
    run_program(&program[..], &mut cpu);

    assert_eq!(4, cpu.cycles);
}

#[test]
fn lda_absy_test_cycles() {
    let mut cpu = Cpu::new();

    let program = [0xb9, 0x00, 0x0f];
    run_program(&program[..], &mut cpu);

    assert_eq!(4, cpu.cycles);
}

#[test]
fn lda_indx_test_cycles() {
    let mut cpu = Cpu::new();

    let program = [0xa1, 0x00];
    run_program(&program[..], &mut cpu);

    assert_eq!(6, cpu.cycles);
}

#[test]
fn lda_indy_test_cycles() {
    let mut cpu = Cpu::new();

    let program = [0xb1, 0x00];
    run_program(&program[..], &mut cpu);

    assert_eq!(5, cpu.cycles);
}

#[test]
fn ldx_imm_test_cycles() {
    let mut cpu = Cpu::new();

    let program = [0xa2, 0x00];
    run_program(&program[..], &mut cpu);

    assert_eq!(2, cpu.cycles);
}

#[test]
fn ldx_zp_test_cycles() {
    let mut cpu = Cpu::new();

    let program = [0xa6, 0x00];
    run_program(&program[..], &mut cpu);

    assert_eq!(3, cpu.cycles);
}

#[test]
fn ldx_zpy_test_cycles() {
    let mut cpu = Cpu::new();

    let program = [0xb6, 0x00];
    run_program(&program[..], &mut cpu);

    assert_eq!(4, cpu.cycles);
}

#[test]
fn ldx_abs_test_cycles() {
    let mut cpu = Cpu::new();

    let program = [0xae, 0x00, 0x0f];
    run_program(&program[..], &mut cpu);

    assert_eq!(4, cpu.cycles);
}

#[test]
fn ldx_absy_test_cycles() {
    let mut cpu = Cpu::new();

    let program = [0xbe, 0x00, 0x0f];
    run_program(&program[..], &mut cpu);

    assert_eq!(4, cpu.cycles);
}

#[test]
fn ldy_imm_test_cycles() {
    let mut cpu = Cpu::new();

    let program = [0xa0, 0x00];
    run_program(&program[..], &mut cpu);

    assert_eq!(2, cpu.cycles);
}

#[test]
fn ldy_zp_test_cycles() {
    let mut cpu = Cpu::new();

    let program = [0xa4, 0x00];
    run_program(&program[..], &mut cpu);

    assert_eq!(3, cpu.cycles);
}

#[test]
fn ldy_zpx_test_cycles() {
    let mut cpu = Cpu::new();

    let program = [0xb4, 0x00];
    run_program(&program[..], &mut cpu);

    assert_eq!(4, cpu.cycles);
}

#[test]
fn ldy_abs_test_cycles() {
    let mut cpu = Cpu::new();

    let program = [0xac, 0x00, 0x0f];
    run_program(&program[..], &mut cpu);

    assert_eq!(4, cpu.cycles);
}

#[test]
fn ldy_absx_test_cycles() {
    let mut cpu = Cpu::new();

    let program = [0xbc, 0x00, 0x0f];
    run_program(&program[..], &mut cpu);

    assert_eq!(4, cpu.cycles);
}

#[test]
fn lsr_impl_test_cycles() {
    let mut cpu = Cpu::new();

    let program = [0x4a];
    run_program(&program[..], &mut cpu);

    assert_eq!(2, cpu.cycles);
}

#[test]
fn lsr_zp_test_cycles() {
    let mut cpu = Cpu::new();

    let program = [0x46, 0x00];
    run_program(&program[..], &mut cpu);

    assert_eq!(5, cpu.cycles);
}

#[test]
fn lsr_zpx_test_cycles() {
    let mut cpu = Cpu::new();

    let program = [0x56, 0x00];
    run_program(&program[..], &mut cpu);

    assert_eq!(6, cpu.cycles);
}

#[test]
fn lsr_abs_test_cycles() {
    let mut cpu = Cpu::new();

    let program = [0x4e, 0x00, 0x0f];
    run_program(&program[..], &mut cpu);

    assert_eq!(6, cpu.cycles);
}

#[test]
fn lsr_absx_test_cycles() {
    let mut cpu = Cpu::new();

    let program = [0x5e, 0x00, 0x0f];
    run_program(&program[..], &mut cpu);

    assert_eq!(7, cpu.cycles);
}

#[test]
fn nop_test_cycles() {
    let mut cpu = Cpu::new();

    let program = [0xea];
    run_program(&program[..], &mut cpu);

    assert_eq!(2, cpu.cycles);
}

#[test]
fn ora_imm_test_cycles() {
    let mut cpu = Cpu::new();

    let program = [0x09];
    run_program(&program[..], &mut cpu);

    assert_eq!(2, cpu.cycles);
}

#[test]
fn ora_zp_test_cycles() {
    let mut cpu = Cpu::new();

    let program = [0x05, 0x00];
    run_program(&program[..], &mut cpu);

    assert_eq!(3, cpu.cycles);
}

#[test]
fn ora_zpx_test_cycles() {
    let mut cpu = Cpu::new();

    let program = [0x15, 0x00];
    run_program(&program[..], &mut cpu);

    assert_eq!(4, cpu.cycles);
}

#[test]
fn ora_abs_test_cycles() {
    let mut cpu = Cpu::new();

    let program = [0x0d, 0x00, 0x0f];
    run_program(&program[..], &mut cpu);

    assert_eq!(4, cpu.cycles);
}

#[test]
fn ora_absx_test_cycles() {
    let mut cpu = Cpu::new();

    let program = [0x1d, 0x00, 0x0f];
    run_program(&program[..], &mut cpu);

    assert_eq!(4, cpu.cycles);
}

#[test]
fn ora_absy_test_cycles() {
    let mut cpu = Cpu::new();

    let program = [0x19, 0x00, 0x0f];
    run_program(&program[..], &mut cpu);

    assert_eq!(4, cpu.cycles);
}

#[test]
fn ora_indx_test_cycles() {
    let mut cpu = Cpu::new();

    let program = [0x01, 0x00];
    run_program(&program[..], &mut cpu);

    assert_eq!(6, cpu.cycles);
}

#[test]
fn ora_indy_test_cycles() {
    let mut cpu = Cpu::new();

    let program = [0x11, 0x00];
    run_program(&program[..], &mut cpu);

    assert_eq!(5, cpu.cycles);
}

//#[test]
fn pha_test_cycles() {
    let mut cpu = Cpu::new();

    let program = [0x48];
    run_program(&program[..], &mut cpu);

    assert_eq!(3, cpu.cycles);
}

//#[test]
fn php_test_cycles() {
    let mut cpu = Cpu::new();

    let program = [0x08];
    run_program(&program[..], &mut cpu);

    assert_eq!(3, cpu.cycles);
}

//#[test]
fn pla_test_cycles() {
    let mut cpu = Cpu::new();

    let program = [0x68];
    run_program(&program[..], &mut cpu);

    assert_eq!(4, cpu.cycles);
}

//#[test]
fn plp_test_cycles() {
    let mut cpu = Cpu::new();

    let program = [0x28];
    run_program(&program[..], &mut cpu);

    assert_eq!(4, cpu.cycles);
}

#[test]
fn rol_impl_test_cycles() {
    let mut cpu = Cpu::new();

    let program = [0x2a];
    run_program(&program[..], &mut cpu);

    assert_eq!(2, cpu.cycles);
}

#[test]
fn rol_zp_test_cycles() {
    let mut cpu = Cpu::new();

    let program = [0x26, 0x00];
    run_program(&program[..], &mut cpu);

    assert_eq!(5, cpu.cycles);
}

#[test]
fn rol_zpx_test_cycles() {
    let mut cpu = Cpu::new();

    let program = [0x36, 0x00];
    run_program(&program[..], &mut cpu);

    assert_eq!(6, cpu.cycles);
}

#[test]
fn rol_abs_test_cycles() {
    let mut cpu = Cpu::new();

    let program = [0x2e, 0x00, 0x0f];
    run_program(&program[..], &mut cpu);

    assert_eq!(6, cpu.cycles);
}

#[test]
fn rol_absx_test_cycles() {
    let mut cpu = Cpu::new();

    let program = [0x3e, 0x00, 0x0f];
    run_program(&program[..], &mut cpu);

    assert_eq!(7, cpu.cycles);
}

#[test]
fn ror_impl_test_cycles() {
    let mut cpu = Cpu::new();

    let program = [0x6a];
    run_program(&program[..], &mut cpu);

    assert_eq!(2, cpu.cycles);
}

#[test]
fn ror_zp_test_cycles() {
    let mut cpu = Cpu::new();

    let program = [0x66, 0x00];
    run_program(&program[..], &mut cpu);

    assert_eq!(5, cpu.cycles);
}

#[test]
fn ror_zpx_test_cycles() {
    let mut cpu = Cpu::new();

    let program = [0x76, 0x00];
    run_program(&program[..], &mut cpu);

    assert_eq!(6, cpu.cycles);
}

#[test]
fn ror_abs_test_cycles() {
    let mut cpu = Cpu::new();

    let program = [0x6e, 0x00, 0x0f];
    run_program(&program[..], &mut cpu);

    assert_eq!(6, cpu.cycles);
}

#[test]
fn ror_absx_test_cycles() {
    let mut cpu = Cpu::new();

    let program = [0x7e, 0x00, 0x0f];
    run_program(&program[..], &mut cpu);

    assert_eq!(7, cpu.cycles);
}

//#[test]
fn rti_test_cycles() {
    let mut cpu = Cpu::new();

    let program = [0x40];
    run_program(&program[..], &mut cpu);

    assert_eq!(6, cpu.cycles);
}

#[test]
fn rts_test_cycles() {
    let mut cpu = Cpu::new();

    let program = [0x60];
    run_program(&program[..], &mut cpu);

    assert_eq!(6, cpu.cycles);
}

#[test]
fn sbc_imm_test_cycles() {
    let mut cpu = Cpu::new();

    let program = [0xe9, 0x00];
    run_program(&program[..], &mut cpu);

    assert_eq!(2, cpu.cycles);
}

#[test]
fn sbc_zp_test_cycles() {
    let mut cpu = Cpu::new();

    let program = [0xe5, 0x00];
    run_program(&program[..], &mut cpu);

    assert_eq!(3, cpu.cycles);
}

#[test]
fn sbc_zpx_test_cycles() {
    let mut cpu = Cpu::new();

    let program = [0xf5, 0x00];
    run_program(&program[..], &mut cpu);

    assert_eq!(4, cpu.cycles);
}

#[test]
fn sbc_abs_test_cycles() {
    let mut cpu = Cpu::new();

    let program = [0xed, 0x00, 0x0f];
    run_program(&program[..], &mut cpu);

    assert_eq!(4, cpu.cycles);
}

#[test]
fn sbc_absx_test_cycles() {
    let mut cpu = Cpu::new();

    let program = [0xfd, 0x00, 0x0f];
    run_program(&program[..], &mut cpu);

    assert_eq!(4, cpu.cycles);
}

#[test]
fn sbc_absy_test_cycles() {
    let mut cpu = Cpu::new();

    let program = [0xf9, 0x00, 0x0f];
    run_program(&program[..], &mut cpu);

    assert_eq!(4, cpu.cycles);
}

#[test]
fn sbc_indx_test_cycles() {
    let mut cpu = Cpu::new();

    let program = [0xe1, 0x00];
    run_program(&program[..], &mut cpu);

    assert_eq!(6, cpu.cycles);
}

#[test]
fn sbc_indy_test_cycles() {
    let mut cpu = Cpu::new();

    let program = [0xf1, 0x00];
    run_program(&program[..], &mut cpu);

    assert_eq!(5, cpu.cycles);
}

#[test]
fn sec_test_cycles() {
    let mut cpu = Cpu::new();

    let program = [0x38];
    run_program(&program[..], &mut cpu);

    assert_eq!(2, cpu.cycles);
}

#[test]
fn sed_test_cycles() {
    let mut cpu = Cpu::new();

    let program = [0xf8];
    run_program(&program[..], &mut cpu);

    assert_eq!(2, cpu.cycles);
}

#[test]
fn sei_test_cycles() {
    let mut cpu = Cpu::new();

    let program = [0x78];
    run_program(&program[..], &mut cpu);

    assert_eq!(2, cpu.cycles);
}

/*
#[test]
fn sta_zp_test_cycles() {
    let mut cpu = Cpu::new();

    let program = [0x85, 0x00];
    run_program(&program[..], &mut cpu);

    assert_eq!(3, cpu.cycles);
}

#[test]
fn sta_zpx_test_cycles() {
    let mut cpu = Cpu::new();

    let program = [0x95, 0x00];
    run_program(&program[..], &mut cpu);

    assert_eq!(4, cpu.cycles);
}

#[test]
fn sta_abs_test_cycles() {
    let mut cpu = Cpu::new();

    let program = [0x8d, 0x00, 0x0f];
    run_program(&program[..], &mut cpu);

    assert_eq!(4, cpu.cycles);
}

#[test]
fn sta_absx_test_cycles() {
    let mut cpu = Cpu::new();

    let program = [0x9d, 0x00, 0x0f];
    run_program(&program[..], &mut cpu);

    assert_eq!(5, cpu.cycles);
}

#[test]
fn sta_absy_test_cycles() {
    let mut cpu = Cpu::new();

    let program = [0x99, 0x00, 0x0f];
    run_program(&program[..], &mut cpu);

    assert_eq!(5, cpu.cycles);
}

#[test]
fn sta_indx_test_cycles() {
    let mut cpu = Cpu::new();

    let program = [0x81, 0x00];
    run_program(&program[..], &mut cpu);

    assert_eq!(6, cpu.cycles);
}

#[test]
fn sta_indy_test_cycles() {
    let mut cpu = Cpu::new();

    let program = [0x91, 0x00];
    run_program(&program[..], &mut cpu);

    assert_eq!(6, cpu.cycles);
}

#[test]
fn stx_zp_test_cycles() {
    let mut cpu = Cpu::new();

    let program = [0x86, 0x00];
    run_program(&program[..], &mut cpu);

    assert_eq!(3, cpu.cycles);
}

#[test]
fn stx_zpy_test_cycles() {
    let mut cpu = Cpu::new();

    let program = [0x96, 0x00];
    run_program(&program[..], &mut cpu);

    assert_eq!(4, cpu.cycles);
}

#[test]
fn stx_abs_test_cycles() {
    let mut cpu = Cpu::new();

    let program = [0x8e, 0x00, 0x0f];
    run_program(&program[..], &mut cpu);

    assert_eq!(4, cpu.cycles);
}

#[test]
fn sty_zp_test_cycles() {
    let mut cpu = Cpu::new();

    let program = [0x84, 0x00];
    run_program(&program[..], &mut cpu);

    assert_eq!(3, cpu.cycles);
}

#[test]
fn sty_zpx_test_cycles() {
    let mut cpu = Cpu::new();

    let program = [0x94, 0x00];
    run_program(&program[..], &mut cpu);

    assert_eq!(4, cpu.cycles);
}

#[test]
fn sty_abs_test_cycles() {
    let mut cpu = Cpu::new();

    let program = [0x8c, 0x00];
    run_program(&program[..], &mut cpu);

    assert_eq!(4, cpu.cycles);
}
*/

#[test]
fn tax_test_cycles() {
    let mut cpu = Cpu::new();

    let program = [0xaa];
    run_program(&program[..], &mut cpu);

    assert_eq!(2, cpu.cycles);
}

#[test]
fn tay_test_cycles() {
    let mut cpu = Cpu::new();

    let program = [0xa8];
    run_program(&program[..], &mut cpu);

    assert_eq!(2, cpu.cycles);
}

#[test]
fn tya_test_cycles() {
    let mut cpu = Cpu::new();

    let program = [0x98];
    run_program(&program[..], &mut cpu);

    assert_eq!(2, cpu.cycles);
}

#[test]
fn tsx_test_cycles() {
    let mut cpu = Cpu::new();

    let program = [0xba];
    run_program(&program[..], &mut cpu);

    assert_eq!(2, cpu.cycles);
}

#[test]
fn txa_test_cycles() {
    let mut cpu = Cpu::new();

    let program = [0x8a];
    run_program(&program[..], &mut cpu);

    assert_eq!(2, cpu.cycles);
}

#[test]
fn txs_test_cycles() {
    let mut cpu = Cpu::new();

    let program = [0x9a];
    run_program(&program[..], &mut cpu);

    assert_eq!(2, cpu.cycles);
}
