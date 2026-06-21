use std::sync::Arc;

use raft_client::client::raft_service_client::RaftServiceClient;
use tokio::sync::Mutex;

pub type ClusterMemberId = u64;

pub struct ClusterMember {
    pub id: ClusterMemberId,
    pub addr: String,
    pub rpc_client: Arc<Mutex<RaftServiceClient<tonic::transport::Channel>>>,
}

impl ClusterMember {
    pub async fn all_members_from_env() -> Result<Vec<Self>, String> {
        let var: String = crate::utils::get_from_env_unsafe("RAFT_MEMBERS")?;
        let mraw = var
            .split(';')
            .map(|e| {
                let parts: Vec<_> = e.split("::").collect();
                (parts[0].parse().unwrap(), parts[1].to_string())
            })
            .collect::<Vec<_>>();

        let mut members = Vec::new();
        for (id, addr) in mraw {
            let client = RaftServiceClient::connect(addr.clone())
                .await
                .map_err(|err| format!("Failed to connect to {addr}: {:?}", err))?;
            members.push(ClusterMember {
                id,
                addr,
                rpc_client: Arc::new(Mutex::new(client)),
            });
        }
        Ok(members)
    }
}

pub struct ClusterConfig {
    pub members: Vec<ClusterMember>,
    pub replication_factor: u64,
    pub my_id: ClusterMemberId,
}

impl ClusterConfig {
    pub async fn from_env() -> Result<Self, String> {
        let replication_factor = crate::utils::get_from_env_unsafe("REPLICATION_FACTOR")?;
        let my_id = crate::utils::get_from_env_unsafe("MY_ID")?;

        let members = ClusterMember::all_members_from_env().await?;

        Ok(ClusterConfig {
            members,
            replication_factor,
            my_id,
        })
    }
}
