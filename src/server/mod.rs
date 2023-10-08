pub mod world;
#[derive(Debug)]
pub enum BlockUpdateMode {
    Destroy,
    Place
}
#[derive(Debug)]
pub enum Command {
    ChunkData([i32;3]),
    BlockUpdate([i32;3], BlockUpdateMode, u8)
}

/// Supposed to be started in a new tread
pub fn manage_chunk_data(mut input : tokio::sync::mpsc::Receiver<Command>, output : tokio::sync::broadcast::Sender<std::sync::Arc<[u8]>>, mut world : world::ServerWorld) {
    while let Some(command) = input.blocking_recv() {
        match command {
            Command::ChunkData(pos) => {
                output.send(world.get_chunk_data(&pos)).unwrap();
            }
            Command::BlockUpdate(pos, mode, block) => {

            }
        }
    }
}