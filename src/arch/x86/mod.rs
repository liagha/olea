pub mod kernel;
pub mod memory;

use self::memory::paging;
use self::memory::paging::{BasePageSize, PageSize, PageTableEntryFlags};
use self::memory::physical;
use crate::consts::*;
use crate::file;
use crate::io::{self, Read};
use crate::logging::*;
use alloc::string::String;
use alloc::vec::Vec;
use core::ptr::write_bytes;
use core::slice;
use goblin::elf::program_header::{PT_DYNAMIC, PT_GNU_RELRO, PT_LOAD};
use goblin::elf64::dynamic::{DT_RELA, DT_RELASZ};
use goblin::elf64::reloc::{R_386_GLOB_DAT, R_386_RELATIVE};
use goblin::{elf, elf64};
use x86::controlregs;
use crate::arch::x86::kernel::calls::transition::to_user_mode;

pub fn load_application(path: &String) -> io::Result<()> {
	debug!("attempting to load application from path.");

	unsafe {
		controlregs::cr3_write(paging::create_usr_pgd().as_u64());
	}

	let mut file = file::File::open(path)?;
	let len = file.len()?;

	if len == 0 {
		error!("file is empty.");
		return Err(io::Error::InvalidArgument);
	}

	if len > isize::MAX as usize {
		error!("file size exceeds maximum supported size.");
		return Err(io::Error::ValueOverflow);
	}

	debug!("file size is {} bytes.", len);
	let mut buffer: Vec<u8> = Vec::new();

	buffer.resize(len, 0);
	file.read(&mut buffer)?;

	let elf = match elf::Elf::parse(&buffer) {
		Ok(n) => n,
		Err(_) => {
			error!("failed to parse elf file format.");
			return Err(io::Error::InvalidArgument);
		}
	};

	drop(file);
	debug!("successfully parsed elf file.");

	if elf.is_lib {
		error!("file is a shared library, not an executable.");
		return Err(io::Error::InvalidArgument);
	}

	if !elf.is_64 {
		error!("file is not a 64-bit executable.");
		return Err(io::Error::InvalidArgument);
	}

	if !elf.libraries.is_empty() {
		error!("file has library dependencies which are not supported.");
		return Err(io::Error::InvalidArgument);
	}

	let vstart: usize = 0;
	let mut exec_size: usize = 0;
	let mut has_loadable_segment = false;

	for header in &elf.program_headers {
		if header.p_type == PT_LOAD {
			has_loadable_segment = true;

			if header.p_vaddr > usize::MAX as u64 || header.p_memsz > usize::MAX as u64 {
				error!("program header addresses exceed supported range.");
				return Err(io::Error::ValueOverflow);
			}

			let segment_end = header.p_vaddr as usize + header.p_memsz as usize;
			if segment_end < header.p_vaddr as usize {
				error!("program header size causes integer overflow.");
				return Err(io::Error::ValueOverflow);
			}

			exec_size = align_up!(
                header.p_vaddr as usize - vstart + header.p_memsz as usize,
                BasePageSize::SIZE
            );
		}
	}

	debug!("virtual start address is 0x{:x}.", vstart);
	debug!("required memory size is 0x{:x} bytes.", exec_size);

	if !has_loadable_segment || exec_size == 0 {
		error!("no loadable program segments found in elf file.");
		return Err(io::Error::InvalidArgument);
	}

	let physical_address = physical::allocate(exec_size);
	if physical_address.as_u64() == 0 {
		error!("failed to allocate physical memory for executable.");
		return Err(io::Error::NoBufferSpace);
	}

	paging::map::<BasePageSize>(
		USER_ENTRY,
		physical_address,
		exec_size / BasePageSize::SIZE,
		PageTableEntryFlags::WRITABLE | PageTableEntryFlags::USER_ACCESSIBLE,
	);

	unsafe {
		write_bytes(USER_ENTRY.as_mut_ptr::<u8>(), 0x00, exec_size);
	}

	let mut rela_addr: u64 = 0;
	let mut relasz: u64 = 0;

	for header in &elf.program_headers {
		if header.p_type == PT_LOAD {
			debug!("loading segment at virtual address 0x{:x}.", header.p_vaddr);

			if header.p_offset > buffer.len() as u64 ||
				header.p_filesz > buffer.len() as u64 ||
				header.p_offset + header.p_filesz > buffer.len() as u64 {
				error!("program header references data beyond file boundaries.");
				return Err(io::Error::InvalidArgument);
			}

			let mem_offset = header.p_vaddr as usize - vstart;
			if mem_offset >= exec_size {
				error!("program header virtual address is outside allocated memory.");
				return Err(io::Error::InvalidArgument);
			}

			let mem = (USER_ENTRY.as_usize() + mem_offset) as *mut u8;

			if header.p_filesz > 0 {
				let mem_slice = unsafe {
					slice::from_raw_parts_mut(mem, header.p_filesz as usize)
				};

				let file_start = header.p_offset as usize;
				let file_end = (header.p_offset + header.p_filesz) as usize;
				mem_slice.copy_from_slice(&buffer[file_start..file_end]);
			}

		} else if header.p_type == PT_GNU_RELRO {
			debug!("found gnu relro segment at 0x{:x}.", header.p_vaddr);

		} else if header.p_type == PT_DYNAMIC {
			debug!("processing dynamic segment at 0x{:x}.", header.p_vaddr);

			if header.p_vaddr > usize::MAX as u64 {
				error!("dynamic segment address exceeds supported range.");
				return Err(io::Error::ValueOverflow);
			}

			let mem_offset = header.p_vaddr as usize - vstart;
			if mem_offset >= exec_size {
				error!("dynamic segment is outside allocated memory.");
				return Err(io::Error::InvalidArgument);
			}

			let mem = (USER_ENTRY.as_u64() + mem_offset as u64) as *mut u8;
			let dynamic_entries = unsafe {
				elf::dynamic::dyn64::from_raw(0, mem as usize)
			};

			for entry in dynamic_entries {
				match entry.d_tag {
					DT_RELA => rela_addr = USER_ENTRY.as_u64() + entry.d_val,
					DT_RELASZ => relasz = entry.d_val,
					_ => {}
				}
			}
		}
	}

	if rela_addr != 0 && relasz > 0 {
		debug!("processing relocations at 0x{:x}.", rela_addr);

		let relocations = unsafe {
			elf64::reloc::from_raw_rela(rela_addr as *const elf64::reloc::Rela, relasz as usize)
		};

		for reloc in relocations {
			if reloc.r_offset > usize::MAX as u64 {
				error!("relocation offset exceeds supported range.");
				return Err(io::Error::ValueOverflow);
			}

			let offset_addr = USER_ENTRY.as_usize() - vstart + reloc.r_offset as usize;
			if offset_addr + 8 > USER_ENTRY.as_usize() + exec_size {
				error!("relocation target is outside allocated memory.");
				return Err(io::Error::InvalidArgument);
			}

			let offset = offset_addr as *mut u64;
			let reloc_type = reloc.r_info & 0xF;

			match reloc_type {
				r if r == R_386_RELATIVE as u64 => {
					unsafe {
						*offset = (USER_ENTRY.as_usize() as i64 - vstart as i64 + reloc.r_addend) as u64;
					}
				}
				r if r == R_386_GLOB_DAT as u64 => {
					debug!("skipping global data relocation.");
				}
				_ => {
					error!("unsupported relocation type {}.", reloc_type);
					return Err(io::Error::InvalidArgument);
				}
			}
		}
	}

	if elf.entry > usize::MAX as u64 {
		error!("entry point address exceeds supported range.");
		return Err(io::Error::ValueOverflow);
	}

	let entry = elf.entry as usize - vstart + USER_ENTRY.as_usize();
	if entry < USER_ENTRY.as_usize() || entry >= USER_ENTRY.as_usize() + exec_size {
		error!("entry point is outside loaded executable memory.");
		return Err(io::Error::InvalidArgument);
	}

	drop(buffer);

	debug!("transferring control to user application at 0x{:x}.", entry);
	unsafe {
		to_user_mode(entry);
	}
}