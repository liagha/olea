use crate::file::NodeKind;

bitflags! {
    #[derive(Debug, Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash)]
    pub struct Permission: u16 {
        const OWNER_READ = 0o400;
        const OWNER_WRITE = 0o200;
        const OWNER_EXECUTE = 0o100;
        const GROUP_READ = 0o040;
        const GROUP_WRITE = 0o020;
        const GROUP_EXECUTE = 0o010;
        const OTHERS_READ = 0o004;
        const OTHERS_WRITE = 0o002;
        const OTHERS_EXECUTE = 0o001;
    }
}

impl Permission {
    pub fn from_mode(mode: u16) -> Self {
        Permission::from_bits_truncate(mode)
    }
    pub fn can_read(&self) -> bool {
        self.intersects(Permission::OWNER_READ | Permission::GROUP_READ | Permission::OTHERS_READ)
    }
    pub fn can_write(&self) -> bool {
        self.intersects(Permission::OWNER_WRITE | Permission::GROUP_WRITE | Permission::OTHERS_WRITE)
    }
    pub fn can_execute(&self) -> bool {
        self.intersects(Permission::OWNER_EXECUTE | Permission::GROUP_EXECUTE | Permission::OTHERS_EXECUTE)
    }
}

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub struct Metadata {
    pub permission: Permission,
    pub user_identifier: u32,
    pub group_identifier: u32,
    pub access_time: u64,
    pub modified_time: u64,
    pub change_time: u64,
    pub kind: NodeKind,
}

impl Metadata {
    pub fn new(kind: NodeKind) -> Self {
        Metadata {
            permission: Permission::from_mode(0o644),
            user_identifier: 0,
            group_identifier: 0,
            access_time: 0,
            modified_time: 0,
            change_time: 0,
            kind,
        }
    }
}