use paxos::SystemHandles;


fn main() {
        let client_count = 5 as usize;
        let replica_count = 3 as usize;
        let leader_count = 3 as usize;
        let acceptor_count = 3 as usize;
        let num_msgs = 10u32;
        let system_handles = SystemHandles::system_handle_management(
            client_count,
            replica_count,
            leader_count,
            acceptor_count,
        );
        system_handles.operation_control(
            num_msgs,
            client_count as u32,
            replica_count as u32,
            leader_count as u32,
        );
}
