pub mod numbers {
	/// Operation not permitted
	pub const OPERATION_NOT_PERMITTED: i32 = 1;

	/// No such file or directory
	pub const FILE_NOT_FOUND: i32 = 2;

	/// No such process
	pub const PROCESS_NOT_FOUND: i32 = 3;

	/// Interrupted system invoke
	pub const INTERRUPTED: i32 = 4;

	/// I/O error
	pub const IO_ERROR: i32 = 5;

	/// No such device or address
	pub const DEVICE_NOT_FOUND: i32 = 6;

	/// Argument list too long
	pub const ARGUMENT_LIST_TOO_LONG: i32 = 7;

	/// Exec format error
	pub const INVALID_EXECUTABLE: i32 = 8;

	/// Bad file number
	pub const BAD_FILE_DESCRIPTOR: i32 = 9;

	/// No child processes
	pub const NO_CHILD_PROCESSES: i32 = 10;

	/// Try again
	pub const TRY_AGAIN: i32 = 11;

	/// Out of memory
	pub const OUT_OF_MEMORY: i32 = 12;

	/// Permission denied
	pub const PERMISSION_DENIED: i32 = 13;

	/// Bad address
	pub const BAD_ADDRESS: i32 = 14;

	/// Block device required
	pub const BLOCK_DEVICE_REQUIRED: i32 = 15;

	/// Device or resource busy
	pub const DEVICE_BUSY: i32 = 16;

	/// File exists
	pub const FILE_EXISTS: i32 = 17;

	/// Cross-device link
	pub const CROSS_DEVICE_LINK: i32 = 18;

	/// No such device
	pub const NO_SUCH_DEVICE: i32 = 19;

	/// Not a directory
	pub const NOT_A_DIRECTORY: i32 = 20;

	/// Is a directory
	pub const IS_A_DIRECTORY: i32 = 21;

	/// Invalid argument
	pub const INVALID_ARGUMENT: i32 = 22;

	/// File table overflow
	pub const FILE_TABLE_OVERFLOW: i32 = 23;

	/// Too many open files
	pub const TOO_MANY_OPEN_FILES: i32 = 24;

	/// Not a terminal
	pub const NOT_A_TERMINAL: i32 = 25;

	/// Text file busy
	pub const TEXT_FILE_BUSY: i32 = 26;

	/// File too large
	pub const FILE_TOO_LARGE: i32 = 27;

	/// No space left on device
	pub const NO_SPACE_LEFT: i32 = 28;

	/// Illegal seek
	pub const ILLEGAL_SEEK: i32 = 29;

	/// Read-only file system
	pub const READ_ONLY_FILESYSTEM: i32 = 30;

	/// Too many links
	pub const TOO_MANY_LINKS: i32 = 31;

	/// Broken pipe
	pub const BROKEN_PIPE: i32 = 32;

	/// Math argument out of domain of func
	pub const MATH_DOMAIN_ERROR: i32 = 33;

	/// Math result not representable
	pub const MATH_RANGE_ERROR: i32 = 34;

	/// Resource deadlock would occur
	pub const DEADLOCK: i32 = 35;

	/// File name too long
	pub const FILENAME_TOO_LONG: i32 = 36;

	/// No record locks available
	pub const NO_LOCKS_AVAILABLE: i32 = 37;

	/// Function not implemented
	pub const NOT_IMPLEMENTED: i32 = 38;

	/// Directory not empty
	pub const DIRECTORY_NOT_EMPTY: i32 = 39;

	/// Too many symbolic links encountered
	pub const TOO_MANY_SYMLINKS: i32 = 40;

	/// Operation not supported
	pub const OPERATION_NOT_PERMITTED_ON_OBJECT: i32 = 41;

	/// Operation would block
	pub const WOULD_BLOCK: i32 = TRY_AGAIN;

	/// No message of desired type
	pub const NO_MESSAGE: i32 = 42;

	/// Identifier removed
	pub const IDENTIFIER_REMOVED: i32 = 43;

	/// Channel number out of range
	pub const CHANNEL_OUT_OF_RANGE: i32 = 44;

	/// Level 2 not synchronized
	pub const LEVEL2_NOT_SYNCED: i32 = 45;

	/// Level 3 halted
	pub const LEVEL3_HALTED: i32 = 46;

	/// Level 3 reset
	pub const LEVEL3_RESET: i32 = 47;

	/// Link number out of range
	pub const LINK_OUT_OF_RANGE: i32 = 48;

	/// Protocol driver not attached
	pub const PROTOCOL_NOT_ATTACHED: i32 = 49;

	/// No CSI structure available
	pub const NO_CSI_AVAILABLE: i32 = 50;

	/// Level 2 halted
	pub const LEVEL2_HALTED: i32 = 51;

	/// Invalid exchange
	pub const INVALID_EXCHANGE: i32 = 52;

	/// Invalid request descriptor
	pub const INVALID_REQUEST_DESCRIPTOR: i32 = 53;

	/// Exchange full
	pub const EXCHANGE_FULL: i32 = 54;

	/// No anode
	pub const NO_ANODE: i32 = 55;

	/// Invalid request code
	pub const INVALID_REQUEST_CODE: i32 = 56;

	/// Invalid slot
	pub const INVALID_SLOT: i32 = 57;

	/// Invalid message
	pub const INVALID_MESSAGE: i32 = 58;

	pub const DEADLOCK_ALIAS: i32 = DEADLOCK;

	/// Bad font file format
	pub const BAD_FONT_FORMAT: i32 = 59;

	/// Device not a stream
	pub const NOT_A_STREAM: i32 = 60;

	/// No data available
	pub const NO_DATA_AVAILABLE: i32 = 61;

	/// Timer expired
	pub const TIMER_EXPIRED: i32 = 62;

	/// Out of streams resources
	pub const OUT_OF_STREAMS: i32 = 63;

	/// Machine is not on the network
	pub const NOT_ON_NETWORK: i32 = 64;

	/// Package not installed
	pub const PACKAGE_NOT_INSTALLED: i32 = 65;

	/// Object is remote
	pub const OBJECT_IS_REMOTE: i32 = 66;

	/// Link has been severed
	pub const LINK_SEVERED: i32 = 67;

	/// Advertise error
	pub const ADVERTISE_ERROR: i32 = 68;

	/// Srmount error
	pub const MOUNT_ERROR: i32 = 69;

	/// Communication error on send
	pub const COMMUNICATION_ERROR: i32 = 70;

	/// Protocol error
	pub const PROTOCOL_ERROR: i32 = 71;

	/// Multihop attempted
	pub const MULTIHOP_ATTEMPTED: i32 = 72;

	/// RFS specific error
	pub const RFS_ERROR: i32 = 73;

	/// Not a data message
	pub const NOT_DATA_MESSAGE: i32 = 74;

	/// Value too large for defined data type
	pub const VALUE_OVERFLOW: i32 = 75;

	/// Name not unique on network
	pub const NAME_NOT_UNIQUE: i32 = 76;

	/// File descriptor in bad state
	pub const BAD_FILE_DESCRIPTOR_STATE: i32 = 77;

	/// Remote address changed
	pub const REMOTE_ADDRESS_CHANGED: i32 = 78;

	/// Can not access a needed shared library
	pub const LIBRARY_ACCESS_ERROR: i32 = 79;

	/// Accessing a corrupted shared library
	pub const LIBRARY_CORRUPTED: i32 = 80;

	/// .lib section in a.out corrupted
	pub const LIBRARY_SECTION_CORRUPTED: i32 = 81;

	/// Attempting to link in too many shared libraries
	pub const TOO_MANY_LIBRARIES: i32 = 82;

	/// Cannot exec a shared library directly
	pub const CANNOT_EXEC_LIBRARY: i32 = 83;

	/// Illegal byte sequence
	pub const ILLEGAL_BYTE_SEQUENCE: i32 = 84;

	/// Interrupted system invoke should be restarted
	pub const RESTART_SYSCALL: i32 = 85;

	/// Streams pipe error
	pub const STREAM_PIPE_ERROR: i32 = 86;

	/// Too many users
	pub const TOO_MANY_USERS: i32 = 87;

	/// Socket operation on non-socket
	pub const NOT_A_SOCKET: i32 = 88;

	/// Destination address required
	pub const DESTINATION_ADDRESS_REQUIRED: i32 = 89;

	/// Message too long
	pub const MESSAGE_TOO_LONG: i32 = 90;

	/// Protocol wrong type for socket
	pub const WRONG_PROTOCOL_TYPE: i32 = 91;

	/// Protocol not available
	pub const PROTOCOL_NOT_AVAILABLE: i32 = 92;

	/// Protocol not supported
	pub const PROTOCOL_NOT_SUPPORTED: i32 = 93;

	/// Socket type not supported
	pub const SOCKET_TYPE_NOT_SUPPORTED: i32 = 94;

	/// Operation not supported on transport endpoint
	pub const OPERATION_NOT_SUPPORTED: i32 = 95;

	/// Protocol family not supported
	pub const PROTOCOL_FAMILY_NOT_SUPPORTED: i32 = 96;

	/// Address family not supported by protocol
	pub const ADDRESS_FAMILY_NOT_SUPPORTED: i32 = 97;

	/// Address already in use
	pub const ADDRESS_IN_USE: i32 = 98;

	/// Cannot assign requested address
	pub const ADDRESS_NOT_AVAILABLE: i32 = 99;

	/// Network is down
	pub const NETWORK_DOWN: i32 = 100;

	/// Network is unreachable
	pub const NETWORK_UNREACHABLE: i32 = 101;

	/// Network dropped connection because of reset
	pub const NETWORK_RESET: i32 = 102;

	/// Software caused connection abort
	pub const CONNECTION_ABORTED: i32 = 103;

	/// Connection reset by peer
	pub const CONNECTION_RESET: i32 = 104;

	/// No buffer space available
	pub const NO_BUFFER_SPACE: i32 = 105;

	/// Transport endpoint is already connected
	pub const ALREADY_CONNECTED: i32 = 106;

	/// Transport endpoint is not connected
	pub const NOT_CONNECTED: i32 = 107;

	/// Cannot send after transport endpoint shutdown
	pub const ENDPOINT_SHUTDOWN: i32 = 108;

	/// Too many references: cannot splice
	pub const TOO_MANY_REFERENCES: i32 = 109;

	/// Connection timed out
	pub const CONNECTION_TIMEOUT: i32 = 110;

	/// Connection refused
	pub const CONNECTION_REFUSED: i32 = 111;

	/// Host is down
	pub const HOST_DOWN: i32 = 112;

	/// No route to host
	pub const HOST_UNREACHABLE: i32 = 113;

	/// Operation already in progress
	pub const ALREADY_IN_PROGRESS: i32 = 114;

	/// Operation now in progress
	pub const IN_PROGRESS: i32 = 115;

	/// Stale file handle
	pub const STALE_FILE_HANDLE: i32 = 116;

	/// Structure needs cleaning
	pub const STRUCTURE_NEEDS_CLEANING: i32 = 117;

	/// Not a XENIX named type file
	pub const NOT_XENIX_FILE: i32 = 118;

	/// No XENIX semaphores available
	pub const NO_XENIX_SEMAPHORES: i32 = 119;

	/// Is a named type file
	pub const IS_NAMED_FILE: i32 = 120;

	/// Remote I/O error
	pub const REMOTE_IO_ERROR: i32 = 121;

	/// Quota exceeded
	pub const QUOTA_EXCEEDED: i32 = 122;

	/// No medium found
	pub const NO_MEDIUM_FOUND: i32 = 123;

	/// Wrong medium type
	pub const WRONG_MEDIUM_TYPE: i32 = 124;

	/// Operation Canceled
	pub const OPERATION_CANCELED: i32 = 125;

	/// Required key not available
	pub const KEY_NOT_AVAILABLE: i32 = 126;

	/// Key has expired
	pub const KEY_EXPIRED: i32 = 127;

	/// Key has been revoked
	pub const KEY_REVOKED: i32 = 128;

	/// Key was rejected by service
	pub const KEY_REJECTED: i32 = 129;

	/// Robust mutexes: Owner died
	pub const MUTEX_OWNER_DIED: i32 = 130;

	/// Robust mutexes: State not recoverable
	pub const MUTEX_NOT_RECOVERABLE: i32 = 131;

	/// Robust mutexes: Operation not possible due to RF-kill
	pub const RF_KILL: i32 = 132;

	/// Robust mutexes: Memory page has hardware error
	pub const HARDWARE_POISON: i32 = 133;

	/// Operation not possible due to inline data
	pub const INLINE_DATA_ERROR: i32 = 134;

	/// Filesystem quota exceeded for user
	pub const USER_QUOTA_EXCEEDED: i32 = 135;

	/// Filesystem quota exceeded for group
	pub const GROUP_QUOTA_EXCEEDED: i32 = 136;

	/// Filesystem quota exceeded for project
	pub const PROJECT_QUOTA_EXCEEDED: i32 = 137;

	/// Operation not supported on socket
	pub const SOCKET_OPERATION_NOT_SUPPORTED: i32 = 138;

	/// Inappropriate ioctl for device
	pub const INAPPROPRIATE_IOCTL: i32 = 139;

	/// No such attribute
	pub const NO_SUCH_ATTRIBUTE: i32 = 140;

	/// Attribute not found
	pub const ATTRIBUTE_NOT_FOUND: i32 = 141;

	/// Directory entry too large
	pub const DIRECTORY_ENTRY_TOO_LARGE: i32 = 142;

	/// Encryption not supported
	pub const ENCRYPTION_NOT_SUPPORTED: i32 = 143;

	/// Snapshot not supported
	pub const SNAPSHOT_NOT_SUPPORTED: i32 = 144;

	/// Filesystem does not support compression
	pub const COMPRESSION_NOT_SUPPORTED: i32 = 145;

	/// No data verification key
	pub const NO_DATA_VERIFICATION_KEY: i32 = 146;

	/// Filesystem does not support verity
	pub const VERITY_NOT_SUPPORTED: i32 = 147;

	/// Corrupted verity data
	pub const VERITY_DATA_CORRUPTED: i32 = 148;

	/// User not authorized for verity operation
	pub const VERITY_NOT_AUTHORIZED: i32 = 149;

	/// Missing verity file descriptor
	pub const NO_VERITY_FILE_DESCRIPTOR: i32 = 150;

	/// Operation not supported on filesystem
	pub const FILESYSTEM_OPERATION_NOT_SUPPORTED: i32 = 151;
}