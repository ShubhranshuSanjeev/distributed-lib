pub type ClusterMemberId = u64;

pub struct ClusterMember {
    pub id: ClusterMemberId,
    pub addr: String,
    // TODO: raft-core will only contain the member id and addr
    // its raft-core task to maintain the cluster membership state
    // and then communicate with the raft-server crate about the members
    // raft-service should keep the rpc client connections according to the
    // cluster membership state
    // pub rpc_client: Arc<Mutex<RaftServiceClient<tonic::transport::Channel>>>,
}

impl ClusterMember {
    pub fn all_members_from_env() -> Result<Vec<Self>, String> {
        let var: String = crate::utils::get_from_env_unsafe("RAFT_MEMBERS")?;
        let mraw = var
            .split(';')
            .map(|e| {
                let parts: Vec<_> = e.split("::").collect();
                (parts[0].parse::<u64>().unwrap(), parts[1].to_string())
            })
            .collect::<Vec<_>>();

        let mut members = Vec::new();
        for (id, addr) in mraw {
            // TODO: replace this is channels
            // RPC send and receive will be handled
            // by the raft-server crate
            // let client = RaftServiceClient::connect(addr.clone())
            //     .await
            //     .map_err(|err| format!("Failed to connect to {addr}: {:?}", err))?;
            members.push(ClusterMember {
                id,
                addr,
                // rpc_client: Arc::new(Mutex::new(client)),
            });
        }
        Ok(members)
    }
}
