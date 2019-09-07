use libc;

#[derive(Copy, Clone, Debug, PartialEq, Eq, Ord, PartialOrd, Hash)]
pub enum ResourceLimit {
    Infinity,
    Unknown,
    Value(u64),
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, Ord, PartialOrd, Hash)]
pub struct ResourceLimits {
    pub soft_limit: ResourceLimit,
    pub hard_limit: ResourceLimit,
}

impl From<libc::rlimit> for ResourceLimits {
    fn from(rs: libc::rlimit) -> Self {
        let soft_limit = match rs.rlim_cur {
            libc::RLIM_INFINITY => ResourceLimit::Infinity,
            other => {
                if libc::RLIM_SAVED_MAX != libc::RLIM_INFINITY && other == libc::RLIM_SAVED_MAX {
                    ResourceLimit::Unknown
                } else if libc::RLIM_SAVED_CUR != libc::RLIM_INFINITY
                    && other == libc::RLIM_SAVED_CUR
                {
                    ResourceLimit::Unknown
                } else {
                    ResourceLimit::Value(other)
                }
            }
        };
        let hard_limit = match rs.rlim_max {
            libc::RLIM_INFINITY => ResourceLimit::Infinity,
            other => {
                if libc::RLIM_SAVED_MAX != libc::RLIM_INFINITY && other == libc::RLIM_SAVED_MAX {
                    ResourceLimit::Unknown
                } else if libc::RLIM_SAVED_CUR != libc::RLIM_INFINITY
                    && other == libc::RLIM_SAVED_CUR
                {
                    ResourceLimit::Unknown
                } else {
                    ResourceLimit::Value(other)
                }
            }
        };
        ResourceLimits {
            soft_limit,
            hard_limit,
        }
    }
}

impl Into<libc::rlimit> for ResourceLimits {
    fn into(self: ResourceLimits) -> libc::rlimit {
        let rlim_cur = match self.soft_limit {
            ResourceLimit::Infinity => libc::RLIM_INFINITY,
            ResourceLimit::Unknown => libc::RLIM_SAVED_CUR,
            ResourceLimit::Value(n) => n,
        };
        let rlim_max = match self.hard_limit {
            ResourceLimit::Infinity => libc::RLIM_INFINITY,
            ResourceLimit::Unknown => libc::RLIM_SAVED_MAX,
            ResourceLimit::Value(n) => n,
        };
        libc::rlimit { rlim_cur, rlim_max }
    }
}

pub enum Resource {
    CoreFileSize,
    CPUTime,
    DataSize,
    FileSize,
    OpenFiles,
    StackSize,
    TotalMemory,
}

impl Into<libc::__rlimit_resource_t> for Resource {
    fn into(self: Resource) -> libc::__rlimit_resource_t {
        match self {
            Resource::CoreFileSize => libc::RLIMIT_CORE,
            Resource::CPUTime => libc::RLIMIT_CPU,
            Resource::DataSize => libc::RLIMIT_DATA,
            Resource::FileSize => libc::RLIMIT_FSIZE,
            Resource::OpenFiles => libc::RLIMIT_NOFILE,
            Resource::StackSize => libc::RLIMIT_STACK,
            Resource::TotalMemory => libc::RLIMIT_AS,
        }
    }
}

impl From<libc::__rlimit_resource_t> for Resource {
    fn from(r: libc::__rlimit_resource_t) -> Self {
        match r {
            libc::RLIMIT_CORE => Resource::CoreFileSize,
            libc::RLIMIT_CPU => Resource::CPUTime,
            libc::RLIMIT_DATA => Resource::DataSize,
            libc::RLIMIT_FSIZE => Resource::FileSize,
            libc::RLIMIT_NOFILE => Resource::OpenFiles,
            libc::RLIMIT_STACK => Resource::StackSize,
            libc::RLIMIT_AS => Resource::TotalMemory,
            _ => panic!("Invalid resource type code"),
        }
    }
}

/// An error value returned from the failure of ['get_resource_limit'].
#[derive(Copy, Clone, Debug, PartialEq, Eq, Ord, PartialOrd, Hash)]
pub enum GetRLimitError {
    /// [EINVAL] An invalid resource was specified; or in a setrlimit() call,
    /// the new rlim_cur exceeds the new rlim_max.
    Invalid,
    /// [EPERM] The limit specified to setrlimit() would have raised the maximum
    /// limit value, and the calling process does not have appropriate
    /// privileges.
    Permission,
}

/// Get the limit values for a particular resource.
pub fn get_resource_limit(resource: Resource) -> Result<ResourceLimits, GetRLimitError> {
    let mut rlimit: libc::rlimit = libc::rlimit {
        rlim_cur: 0_u64,
        rlim_max: 0_u64,
    };
    unsafe {
        match libc::getrlimit(resource.into(), &mut rlimit) {
            0 => Ok(rlimit.into()),
            -1 => {
                let errno: *mut libc::c_int = libc::__errno_location();
                Err(match *errno {
                    libc::EINVAL => GetRLimitError::Invalid,
                    libc::EPERM => GetRLimitError::Permission,
                    _ => panic!("Invalid error code"),
                })
            }
            _ => panic!("Invalid error return"),
        }
    }
}

/// An error value returned from the failure of ['set_resource_limit'].
#[derive(Copy, Clone, Debug, PartialEq, Eq, Ord, PartialOrd, Hash)]
pub enum SetRLimitError {
    /// [EINVAL] The limit specified cannot be lowered because current usage is
    /// already higher than the limit.
    Invalid,
}

/// Set the limit values for a particular resource.
pub fn set_resource_limit(
    resource: Resource,
    r_limit: ResourceLimits,
) -> Result<(), SetRLimitError> {
    unsafe {
        match libc::setrlimit(resource.into(), &r_limit.into()) {
            0 => Ok(()),
            -1 => {
                let errno: *mut libc::c_int = libc::__errno_location();
                Err(match *errno {
                    libc::EINVAL => SetRLimitError::Invalid,
                    _ => panic!("Invalid error code"),
                })
            }
            _ => panic!("Invalid error return"),
        }
    }
}
