use crate::file::descriptor::FileDescriptor;
use crate::logging::*;

#[repr(C)]
pub struct IoVec {
	pub iov_base: *const u8,
	pub iov_len: usize,
}

pub(crate) unsafe extern "C" fn sys_writev(
	fd: FileDescriptor,
	ptr: *const IoVec,
	cnt: i32,
) -> isize {
	debug!("Enter syscall writev");
	let mut len: isize = 0;
	info!("slice 2");

	let iovec = core::slice::from_raw_parts(ptr, cnt as usize);

	for i in iovec {
		info!("slice 3");
		let slice = core::slice::from_raw_parts(i.iov_base, i.iov_len);

		let tmp: isize = crate::file::descriptor::write(fd, slice).map_or_else(
			|e| -num::ToPrimitive::to_isize(&e).unwrap(),
			|v| v.try_into().unwrap(),
		);

		len += tmp;
		if tmp < i.iov_len as isize {
			break;
		}
	}

	len
}

pub(crate) unsafe extern "C" fn sys_write(fd: FileDescriptor, buf: *mut u8, len: usize) -> isize {
	debug!("Enter syscall write");
	info!("slice 4");
	
	let slice = unsafe { core::slice::from_raw_parts(buf, len) };
	crate::file::descriptor::write(fd, slice).map_or_else(
		|e| -num::ToPrimitive::to_isize(&e).unwrap(),
		|v| v.try_into().unwrap(),
	)
}
