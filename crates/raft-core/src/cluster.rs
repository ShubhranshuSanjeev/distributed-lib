pub mod member;

pub struct ClusterConfig {
    pub members: Vec<member::ClusterMember>,
    pub quorum_size: u64,
    pub my_id: member::ClusterMemberId,
}

impl ClusterConfig {
    pub fn from_env() -> Result<Self, String> {
        let replication_factor = crate::utils::get_from_env_unsafe("REPLICATION_FACTOR")?;
        let my_id = crate::utils::get_from_env_unsafe("MY_ID")?;

        let members = member::ClusterMember::all_members_from_env()?;

        Ok(ClusterConfig {
            members,
            quorum_size: replication_factor,
            my_id,
        })
    }
}
