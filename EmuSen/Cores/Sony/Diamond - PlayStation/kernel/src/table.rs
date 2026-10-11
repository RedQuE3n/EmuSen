//! The A0h, B0h and C0h function tables in psx-spx's order ("BIOS Function Summary"), one line a call, and their
//! installation: the A table at 200h, the C table followed by the B table in the kernel's RAM, as C(80h) and up
//! read B(00h) and up. Each line's status: D implemented, P phase 2 (the documented failure value), N no function
//! (psx-spx's return 0, jump to 0 or SystemError), U named but not described (returns 0). The harness reads these
//! lines to print Diamond_OpenKernel.md's call table. Diamond_OpenKernel.md §5.

use crate::layout::{self, A_ENTRIES, B_ENTRIES, C_ENTRIES};
use crate::mem::{rd, wr};
use crate::{
    blocks, convert, entry, event, exception, exec, file, gpu, heap, memory, misc, pad, phase2, string, thread, timer,
    tty,
};

pub struct Table<const N: usize>(pub [*const (); N]);
unsafe impl<const N: usize> Sync for Table<N> {}

macro_rules! e {
    ($status:ident, $name:literal, $f:path) => {
        $f as *const ()
    };
}

/// The B and C tables in RAM, C first; GetC0Table and GetB0Table give their addresses.
#[unsafe(no_mangle)]
pub static mut KERNEL_CB_TABLE: [u32; C_ENTRIES + B_ENTRIES] = [0; C_ENTRIES + B_ENTRIES];

pub fn c_table() -> u32 {
    &raw const KERNEL_CB_TABLE as u32
}

pub fn b_table() -> u32 {
    c_table() + 4 * C_ENTRIES as u32
}

pub static A: Table<A_ENTRIES> = Table(build_a());
pub static B: Table<B_ENTRIES> = Table(build_b());
pub static C: Table<C_ENTRIES> = Table(build_c());

const fn build_a() -> [*const (); A_ENTRIES] {
    let mut t = [entry::jump_zero as *const (); A_ENTRIES];
    t[0x00] = e!(D, "open(filename,accessmode)", file::open);
    t[0x01] = e!(D, "lseek(fd,offset,seektype)", file::lseek);
    t[0x02] = e!(D, "read(fd,dst,length)", file::read);
    t[0x03] = e!(D, "write(fd,src,length)", file::write);
    t[0x04] = e!(D, "close(fd)", file::close);
    t[0x05] = e!(D, "ioctl(fd,cmd,arg)", file::ioctl);
    t[0x06] = e!(D, "exit(exitcode)", misc::exit);
    t[0x07] = e!(D, "isatty(fd)", file::isatty);
    t[0x08] = e!(D, "getc(fd)", file::getc);
    t[0x09] = e!(D, "putc(char,fd)", file::putc);
    t[0x0A] = e!(D, "todigit(char)", convert::todigit);
    t[0x0B] = e!(N, "atof(src)", entry::cop1_fault);
    t[0x0C] = e!(D, "strtoul(src,src_end,base)", convert::strtoul);
    t[0x0D] = e!(D, "strtol(src,src_end,base)", convert::strtol);
    t[0x0E] = e!(D, "abs(val)", convert::abs);
    t[0x0F] = e!(D, "labs(val)", convert::abs);
    t[0x10] = e!(D, "atoi(src)", convert::atoi);
    t[0x11] = e!(D, "atol(src)", convert::atoi);
    t[0x12] = e!(D, "atob(src,num_dst)", convert::atob);
    t[0x13] = e!(D, "setjmp(buf)", entry::setjmp);
    t[0x14] = e!(D, "longjmp(buf,param)", entry::longjmp);
    t[0x15] = e!(D, "strcat(dst,src)", string::strcat);
    t[0x16] = e!(D, "strncat(dst,src,maxlen)", string::strncat);
    t[0x17] = e!(D, "strcmp(str1,str2)", string::strcmp);
    t[0x18] = e!(D, "strncmp(str1,str2,maxlen)", string::strncmp);
    t[0x19] = e!(D, "strcpy(dst,src)", string::strcpy);
    t[0x1A] = e!(D, "strncpy(dst,src,maxlen)", string::strncpy);
    t[0x1B] = e!(D, "strlen(src)", string::strlen);
    t[0x1C] = e!(D, "index(src,char)", string::index);
    t[0x1D] = e!(D, "rindex(src,char)", string::rindex);
    t[0x1E] = e!(D, "strchr(src,char)", string::index);
    t[0x1F] = e!(D, "strrchr(src,char)", string::rindex);
    t[0x20] = e!(D, "strpbrk(src,list)", string::strpbrk);
    t[0x21] = e!(D, "strspn(src,list)", string::strspn);
    t[0x22] = e!(D, "strcspn(src,list)", string::strcspn);
    t[0x23] = e!(D, "strtok(src,list)", string::strtok);
    t[0x24] = e!(D, "strstr(str,substr)", string::strstr);
    t[0x25] = e!(D, "toupper(char)", convert::toupper);
    t[0x26] = e!(D, "tolower(char)", convert::tolower);
    t[0x27] = e!(D, "bcopy(src,dst,len)", memory::bcopy);
    t[0x28] = e!(D, "bzero(dst,len)", memory::bzero);
    t[0x29] = e!(D, "bcmp(ptr1,ptr2,len)", memory::memcmp);
    t[0x2A] = e!(D, "memcpy(dst,src,len)", memory::memcpy);
    t[0x2B] = e!(D, "memset(dst,fillbyte,len)", memory::memset);
    t[0x2C] = e!(D, "memmove(dst,src,len)", memory::memmove);
    t[0x2D] = e!(D, "memcmp(src1,src2,len)", memory::memcmp);
    t[0x2E] = e!(D, "memchr(src,scanbyte,len)", memory::memchr);
    t[0x2F] = e!(D, "rand()", misc::rand);
    t[0x30] = e!(D, "srand(seed)", misc::srand);
    t[0x31] = e!(D, "qsort(base,nel,width,callback)", misc::qsort);
    t[0x32] = e!(N, "strtod(src,src_end)", entry::cop1_fault);
    t[0x33] = e!(D, "malloc(size)", heap::malloc);
    t[0x34] = e!(D, "free(buf)", heap::free);
    t[0x35] = e!(D, "lsearch(key,base,nel,width,callback)", misc::lsearch);
    t[0x36] = e!(D, "bsearch(key,base,nel,width,callback)", misc::bsearch);
    t[0x37] = e!(D, "calloc(sizx,sizy)", heap::calloc);
    t[0x38] = e!(D, "realloc(old_buf,new_siz)", heap::realloc);
    t[0x39] = e!(D, "InitHeap(addr,size)", heap::init_heap);
    t[0x3A] = e!(N, "_exit(exitcode)", entry::system_error);
    t[0x3B] = e!(D, "getchar()", tty::getchar);
    t[0x3C] = e!(D, "putchar(char)", tty::putchar);
    t[0x3D] = e!(D, "gets(dst)", tty::gets);
    t[0x3E] = e!(D, "puts(src)", tty::puts);
    t[0x3F] = e!(D, "printf(txt,param1,param2,etc.)", entry::printf);
    t[0x40] = e!(N, "SystemErrorUnresolvedException()", entry::system_error);
    t[0x41] = e!(D, "LoadTest(filename,headerbuf)", exec::load_test);
    t[0x42] = e!(D, "Load(filename,headerbuf)", exec::load);
    t[0x43] = e!(D, "Exec(headerbuf,param1,param2)", entry::exec);
    t[0x44] = e!(D, "FlushCache()", entry::flush_cache);
    t[0x45] = e!(D, "init_a0_b0_c0_vectors", install_vectors);
    t[0x46] = e!(D, "GPU_dw(Xdst,Ydst,Xsiz,Ysiz,src)", gpu::gpu_dw);
    t[0x47] = e!(D, "gpu_send_dma(Xdst,Ydst,Xsiz,Ysiz,src)", gpu::gpu_send_dma);
    t[0x48] = e!(D, "SendGP1Command(gp1cmd)", entry::send_gp1_command);
    t[0x49] = e!(D, "GPU_cw(gp0cmd)", gpu::gpu_cw);
    t[0x4A] = e!(D, "GPU_cwp(src,num)", gpu::gpu_cwp);
    t[0x4B] = e!(D, "send_gpu_linked_list(src)", gpu::send_gpu_linked_list);
    t[0x4C] = e!(D, "gpu_abort_dma()", gpu::gpu_abort_dma);
    t[0x4D] = e!(D, "GetGPUStatus()", gpu::get_gpu_status);
    t[0x4E] = e!(D, "gpu_sync()", gpu::gpu_sync);
    t[0x4F] = e!(N, "SystemError", entry::system_error);
    t[0x50] = e!(N, "SystemError", entry::system_error);
    t[0x51] = e!(D, "LoadExec(filename,stackbase,stackoffset)", exec::load_exec);
    t[0x52] = e!(N, "GetSysSp", entry::system_error);
    t[0x53] = e!(N, "SystemError", entry::system_error);
    t[0x54] = e!(P, "_96_init()", phase2::nothing);
    t[0x55] = e!(P, "_bu_init()", phase2::nothing);
    t[0x56] = e!(P, "_96_remove()", phase2::nothing);
    t[0x57] = e!(N, "return 0", entry::return_zero);
    t[0x58] = e!(N, "return 0", entry::return_zero);
    t[0x59] = e!(N, "return 0", entry::return_zero);
    t[0x5A] = e!(N, "return 0", entry::return_zero);
    t[0x5B] = e!(D, "dev_tty_init()", tty::dev_init);
    t[0x5C] = e!(D, "dev_tty_open(fcb,path,accessmode)", tty::dev_open);
    t[0x5D] = e!(D, "dev_tty_in_out(fcb,cmd)", tty::dev_in_out);
    t[0x5E] = e!(D, "dev_tty_ioctl(fcb,cmd,arg)", tty::dev_ioctl);
    t[0x5F] = e!(P, "dev_cd_open(fcb,path,accessmode)", phase2::failed);
    t[0x60] = e!(P, "dev_cd_read(fcb,dst,len)", phase2::failed);
    t[0x61] = e!(P, "dev_cd_close(fcb)", phase2::failed);
    t[0x62] = e!(P, "dev_cd_firstfile(fcb,path,direntry)", phase2::nothing);
    t[0x63] = e!(P, "dev_cd_nextfile(fcb,direntry)", phase2::nothing);
    t[0x64] = e!(P, "dev_cd_chdir(fcb,path)", phase2::nothing);
    t[0x65] = e!(P, "dev_card_open(fcb,path,accessmode)", phase2::failed);
    t[0x66] = e!(P, "dev_card_read(fcb,dst,len)", phase2::failed);
    t[0x67] = e!(P, "dev_card_write(fcb,src,len)", phase2::failed);
    t[0x68] = e!(P, "dev_card_close(fcb)", phase2::failed);
    t[0x69] = e!(P, "dev_card_firstfile(fcb,path,direntry)", phase2::nothing);
    t[0x6A] = e!(P, "dev_card_nextfile(fcb,direntry)", phase2::nothing);
    t[0x6B] = e!(P, "dev_card_erase(fcb,path)", phase2::nothing);
    t[0x6C] = e!(P, "dev_card_undelete(fcb,path)", phase2::nothing);
    t[0x6D] = e!(P, "dev_card_format(fcb)", phase2::nothing);
    t[0x6E] = e!(P, "dev_card_rename(fcb1,path1,fcb2,path2)", phase2::nothing);
    t[0x6F] = e!(P, "card_clear_error(fcb)", phase2::nothing);
    t[0x70] = e!(P, "_bu_init()", phase2::nothing);
    t[0x71] = e!(P, "_96_init()", phase2::nothing);
    t[0x72] = e!(P, "_96_remove()", phase2::nothing);
    t[0x73] = e!(N, "return 0", entry::return_zero);
    t[0x74] = e!(N, "return 0", entry::return_zero);
    t[0x75] = e!(N, "return 0", entry::return_zero);
    t[0x76] = e!(N, "return 0", entry::return_zero);
    t[0x77] = e!(N, "return 0", entry::return_zero);
    t[0x78] = e!(P, "CdAsyncSeekL(src)", phase2::nothing);
    t[0x79] = e!(N, "return 0", entry::return_zero);
    t[0x7A] = e!(N, "return 0", entry::return_zero);
    t[0x7B] = e!(N, "return 0", entry::return_zero);
    t[0x7C] = e!(P, "CdAsyncGetStatus(dst)", phase2::nothing);
    t[0x7D] = e!(N, "return 0", entry::return_zero);
    t[0x7E] = e!(P, "CdAsyncReadSector(count,dst,mode)", phase2::nothing);
    t[0x7F] = e!(N, "return 0", entry::return_zero);
    t[0x80] = e!(N, "return 0", entry::return_zero);
    t[0x81] = e!(P, "CdAsyncSetMode(mode)", phase2::nothing);
    t[0x82] = e!(N, "return 0", entry::return_zero);
    t[0x83] = e!(N, "return 0", entry::return_zero);
    t[0x84] = e!(N, "return 0", entry::return_zero);
    t[0x85] = e!(N, "return 0", entry::return_zero);
    t[0x86] = e!(N, "return 0", entry::return_zero);
    t[0x87] = e!(N, "return 0", entry::return_zero);
    t[0x88] = e!(N, "return 0", entry::return_zero);
    t[0x89] = e!(N, "return 0", entry::return_zero);
    t[0x8A] = e!(N, "return 0", entry::return_zero);
    t[0x8B] = e!(N, "return 0", entry::return_zero);
    t[0x8C] = e!(N, "return 0", entry::return_zero);
    t[0x8D] = e!(N, "return 0", entry::return_zero);
    t[0x8E] = e!(N, "return 0", entry::return_zero);
    t[0x8F] = e!(N, "return 0", entry::return_zero);
    t[0x90] = e!(P, "CdromIoIrqFunc1()", phase2::nothing);
    t[0x91] = e!(P, "CdromDmaIrqFunc1()", phase2::nothing);
    t[0x92] = e!(P, "CdromIoIrqFunc2()", phase2::nothing);
    t[0x93] = e!(P, "CdromDmaIrqFunc2()", phase2::nothing);
    t[0x94] = e!(P, "CdromGetInt5errCode(dst1,dst2)", phase2::nothing);
    t[0x95] = e!(P, "CdInitSubFunc()", phase2::nothing);
    t[0x96] = e!(P, "AddCDROMDevice()", phase2::nothing);
    t[0x97] = e!(P, "AddMemCardDevice()", phase2::nothing);
    t[0x98] = e!(D, "AddDuartTtyDevice()", file::add_duart_tty_device);
    t[0x99] = e!(D, "add_nullcon_driver()", file::add_nullcon_driver);
    t[0x9A] = e!(N, "SystemError", entry::system_error);
    t[0x9B] = e!(N, "SystemError", entry::system_error);
    t[0x9C] = e!(D, "SetConf(num_EvCB,num_TCB,stacktop)", blocks::set_conf);
    t[0x9D] = e!(D, "GetConf(num_EvCB_dst,num_TCB_dst,stacktop_dst)", blocks::get_conf);
    t[0x9E] = e!(P, "SetCdromIrqAutoAbort(type,flag)", phase2::nothing);
    t[0x9F] = e!(D, "SetMem(megabytes)", misc::set_mem);
    t[0xA0] = e!(P, "_boot()", phase2::boot);
    t[0xA1] = e!(N, "SystemError(type,errorcode)", entry::system_error);
    t[0xA2] = e!(P, "EnqueueCdIntr()", phase2::nothing);
    t[0xA3] = e!(P, "DequeueCdIntr()", phase2::nothing);
    t[0xA4] = e!(P, "CdGetLbn(filename)", phase2::failed);
    t[0xA5] = e!(P, "CdReadSector(count,sector,buffer)", phase2::failed);
    t[0xA6] = e!(P, "CdGetStatus()", phase2::failed);
    t[0xA7] = e!(P, "bufs_cb_0()", phase2::nothing);
    t[0xA8] = e!(P, "bufs_cb_1()", phase2::nothing);
    t[0xA9] = e!(P, "bufs_cb_2()", phase2::nothing);
    t[0xAA] = e!(P, "bufs_cb_3()", phase2::nothing);
    t[0xAB] = e!(P, "_card_info(port)", phase2::nothing);
    t[0xAC] = e!(P, "_card_load(port)", phase2::nothing);
    t[0xAD] = e!(P, "_card_auto(flag)", phase2::nothing);
    t[0xAE] = e!(P, "bufs_cb_4()", phase2::nothing);
    t[0xAF] = e!(P, "card_write_test(port)", phase2::nothing);
    t[0xB0] = e!(N, "return 0", entry::return_zero);
    t[0xB1] = e!(N, "return 0", entry::return_zero);
    t[0xB2] = e!(D, "ioabort_raw(param)", misc::ioabort_raw);
    t[0xB3] = e!(N, "return 0", entry::return_zero);
    t[0xB4] = e!(D, "GetSystemInfo(index)", misc::get_system_info);
    t
}

const fn build_b() -> [*const (); B_ENTRIES] {
    let mut t = [entry::jump_zero as *const (); B_ENTRIES];
    t[0x00] = e!(D, "alloc_kernel_memory(size)", heap::alloc_kernel_memory);
    t[0x01] = e!(D, "free_kernel_memory(buf)", heap::free);
    t[0x02] = e!(D, "init_timer(t,reload,flags)", timer::init_timer);
    t[0x03] = e!(D, "get_timer(t)", timer::get_timer);
    t[0x04] = e!(D, "enable_timer_irq(t)", timer::enable_timer_irq);
    t[0x05] = e!(D, "disable_timer_irq(t)", timer::disable_timer_irq);
    t[0x06] = e!(D, "restart_timer(t)", timer::restart_timer);
    t[0x07] = e!(D, "DeliverEvent(class,spec)", event::deliver_event);
    t[0x08] = e!(D, "OpenEvent(class,spec,mode,func)", event::open_event);
    t[0x09] = e!(D, "CloseEvent(event)", event::close_event);
    t[0x0A] = e!(D, "WaitEvent(event)", event::wait_event);
    t[0x0B] = e!(D, "TestEvent(event)", event::test_event);
    t[0x0C] = e!(D, "EnableEvent(event)", event::enable_event);
    t[0x0D] = e!(D, "DisableEvent(event)", event::disable_event);
    t[0x0E] = e!(D, "OpenTh(reg_PC,reg_SP_FP,reg_GP)", thread::open_th);
    t[0x0F] = e!(D, "CloseTh(handle)", thread::close_th);
    t[0x10] = e!(D, "ChangeTh(handle)", thread::change_th);
    t[0x11] = e!(N, "jump_to_00000000h", entry::jump_zero);
    t[0x12] = e!(D, "InitPAD2(buf1,siz1,buf2,siz2)", pad::init_pad2);
    t[0x13] = e!(D, "StartPAD2()", pad::start_pad2);
    t[0x14] = e!(D, "StopPAD2()", pad::stop_pad2);
    t[0x15] = e!(D, "PAD_init2(type,button_dest,unused,unused)", pad::pad_init2);
    t[0x16] = e!(D, "PAD_dr()", pad::pad_dr);
    t[0x17] = e!(D, "ReturnFromException()", entry::return_from_exception);
    t[0x18] = e!(D, "ResetEntryInt()", exception::reset_entry_int);
    t[0x19] = e!(D, "HookEntryInt(addr)", exception::hook_entry_int);
    t[0x1A] = e!(N, "SystemError", entry::system_error);
    t[0x1B] = e!(N, "SystemError", entry::system_error);
    t[0x1C] = e!(N, "SystemError", entry::system_error);
    t[0x1D] = e!(N, "SystemError", entry::system_error);
    t[0x1E] = e!(N, "SystemError", entry::system_error);
    t[0x1F] = e!(N, "SystemError", entry::system_error);
    t[0x20] = e!(D, "UnDeliverEvent(class,spec)", event::undeliver_event);
    t[0x21] = e!(N, "SystemError", entry::system_error);
    t[0x22] = e!(N, "SystemError", entry::system_error);
    t[0x23] = e!(N, "SystemError", entry::system_error);
    t[0x2A] = e!(N, "SystemError", entry::system_error);
    t[0x2B] = e!(N, "SystemError", entry::system_error);
    t[0x32] = e!(D, "open(filename,accessmode)", file::open);
    t[0x33] = e!(D, "lseek(fd,offset,seektype)", file::lseek);
    t[0x34] = e!(D, "read(fd,dst,length)", file::read);
    t[0x35] = e!(D, "write(fd,src,length)", file::write);
    t[0x36] = e!(D, "close(fd)", file::close);
    t[0x37] = e!(D, "ioctl(fd,cmd,arg)", file::ioctl);
    t[0x38] = e!(D, "exit(exitcode)", misc::exit);
    t[0x39] = e!(D, "isatty(fd)", file::isatty);
    t[0x3A] = e!(D, "getc(fd)", file::getc);
    t[0x3B] = e!(D, "putc(char,fd)", file::putc);
    t[0x3C] = e!(D, "getchar()", tty::getchar);
    t[0x3D] = e!(D, "putchar(char)", tty::putchar);
    t[0x3E] = e!(D, "gets(dst)", tty::gets);
    t[0x3F] = e!(D, "puts(src)", tty::puts);
    t[0x40] = e!(D, "cd(name)", file::cd);
    t[0x41] = e!(D, "format(devicename)", file::format);
    t[0x42] = e!(D, "firstfile2(filename,direntry)", file::firstfile2);
    t[0x43] = e!(D, "nextfile(direntry)", file::nextfile);
    t[0x44] = e!(D, "rename(old_filename,new_filename)", file::rename);
    t[0x45] = e!(D, "erase(filename)", file::erase);
    t[0x46] = e!(D, "undelete(filename)", file::undelete);
    t[0x47] = e!(D, "AddDrv(device_info)", file::add_drv);
    t[0x48] = e!(D, "DelDrv(device_name_lowercase)", file::del_drv);
    t[0x49] = e!(D, "PrintInstalledDevices()", file::print_installed_devices);
    t[0x4A] = e!(P, "InitCARD2(pad_enable)", pad::init_card2);
    t[0x4B] = e!(P, "StartCARD2()", phase2::nothing);
    t[0x4C] = e!(P, "StopCARD2()", phase2::nothing);
    t[0x4D] = e!(P, "_card_info_subfunc(port)", phase2::nothing);
    t[0x4E] = e!(P, "_card_write(port,sector,src)", phase2::nothing);
    t[0x4F] = e!(P, "_card_read(port,sector,dst)", phase2::nothing);
    t[0x50] = e!(P, "_new_card()", phase2::nothing);
    t[0x51] = e!(P, "Krom2RawAdd(shiftjis_code)", phase2::failed);
    t[0x52] = e!(N, "SystemError", entry::system_error);
    t[0x53] = e!(P, "Krom2Offset(shiftjis_code)", phase2::failed);
    t[0x54] = e!(D, "_get_errno()", file::get_errno);
    t[0x55] = e!(D, "_get_error(fd)", file::get_error);
    t[0x56] = e!(D, "GetC0Table", get_c0_table);
    t[0x57] = e!(D, "GetB0Table", get_b0_table);
    t[0x58] = e!(P, "_card_chan()", phase2::nothing);
    t[0x59] = e!(D, "testdevice(devicename)", file::testdevice);
    t[0x5A] = e!(N, "SystemError", entry::system_error);
    t[0x5B] = e!(D, "ChangeClearPAD(int)", pad::change_clear_pad);
    t[0x5C] = e!(P, "_card_status(slot)", phase2::card_status);
    t[0x5D] = e!(P, "_card_wait(slot)", phase2::card_status);
    t
}

const fn build_c() -> [*const (); C_ENTRIES] {
    let mut t = [entry::jump_zero as *const (); C_ENTRIES];
    t[0x00] = e!(D, "EnqueueTimerAndVblankIrqs(priority)", exception::enqueue_timer_and_vblank_irqs);
    t[0x01] = e!(D, "EnqueueSyscallHandler(priority)", exception::enqueue_syscall_handler);
    t[0x02] = e!(D, "SysEnqIntRP(priority,struc)", exception::sys_enq_int_rp);
    t[0x03] = e!(D, "SysDeqIntRP(priority,struc)", exception::sys_deq_int_rp);
    t[0x04] = e!(D, "get_free_EvCB_slot()", event::get_free_evcb_slot);
    t[0x05] = e!(D, "get_free_TCB_slot()", thread::get_free_tcb_slot);
    t[0x06] = e!(D, "ExceptionHandler()", entry::exception_entry);
    t[0x07] = e!(D, "InstallExceptionHandlers()", install_exception_handlers);
    t[0x08] = e!(D, "SysInitMemory(addr,size)", heap::sys_init_memory);
    t[0x09] = e!(U, "SysInitKernelVariables()", entry::return_zero);
    t[0x0A] = e!(D, "ChangeClearRCnt(t,flag)", exception::change_clear_rcnt);
    t[0x0B] = e!(N, "SystemError", entry::system_error);
    t[0x0C] = e!(D, "InitDefInt(priority)", exception::init_def_int);
    t[0x0D] = e!(D, "SetIrqAutoAck(irq,flag)", exception::set_irq_auto_ack);
    t[0x0E] = e!(N, "return 0", entry::return_zero);
    t[0x0F] = e!(N, "return 0", entry::return_zero);
    t[0x10] = e!(N, "return 0", entry::return_zero);
    t[0x11] = e!(N, "return 0", entry::return_zero);
    t[0x12] = e!(D, "InstallDevices(ttyflag)", file::install_devices);
    t[0x13] = e!(D, "FlushStdInOutPut()", file::flush_std_in_out_put);
    t[0x14] = e!(N, "return 0", entry::return_zero);
    t[0x15] = e!(U, "_cdevinput(circ,char)", entry::return_zero);
    t[0x16] = e!(U, "_cdevscan()", entry::return_zero);
    t[0x17] = e!(U, "_circgetc(circ)", entry::return_zero);
    t[0x18] = e!(U, "_circputc(char,circ)", entry::return_zero);
    t[0x19] = e!(D, "_ioabort(txt1,txt2)", misc::ioabort);
    t[0x1A] = e!(D, "set_card_find_mode(mode)", phase2::set_card_find_mode);
    t[0x1B] = e!(D, "KernelRedirect(ttyflag)", file::kernel_redirect);
    t[0x1C] = e!(U, "AdjustA0Table()", entry::return_zero);
    t[0x1D] = e!(D, "get_card_find_mode()", phase2::get_card_find_mode);
    t
}

fn copy(from: &[*const ()], to: u32) {
    for (i, f) in from.iter().enumerate() {
        wr(to + 4 * i as u32, *f as u32);
    }
}

fn copy_vector(template: unsafe extern "C" fn(), to: u32) {
    let from = template as *const () as u32;
    for i in 0..4 {
        wr(to + 4 * i, rd(from + 4 * i));
    }
}

/// The tables into RAM, and the four vectors; 00000000h gets a copy of the exception vector.
pub fn install() {
    copy(&A.0, layout::A_TABLE);
    copy(&C.0, c_table());
    copy(&B.0, b_table());
    install_exception_handlers();
    install_vectors();
    copy_vector(entry::vector_exception, layout::GARBAGE);
}

/// C(07h).
pub extern "C" fn install_exception_handlers() -> u32 {
    copy_vector(entry::vector_exception, layout::VECTOR_EXCEPTION);
    0
}

/// A(45h).
pub extern "C" fn install_vectors() -> u32 {
    copy_vector(entry::vector_a, layout::VECTOR_A);
    copy_vector(entry::vector_b, layout::VECTOR_B);
    copy_vector(entry::vector_c, layout::VECTOR_C);
    0
}

pub extern "C" fn get_c0_table() -> u32 {
    c_table()
}

pub extern "C" fn get_b0_table() -> u32 {
    b_table()
}
