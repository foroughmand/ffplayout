use std::{
    process,
    sync::{Arc, Mutex, atomic::{AtomicBool, AtomicUsize}},
    thread,
};

use simplelog::*;

use crate::utils::{GlobalConfig, Media, PlayoutStatus};

pub mod folder;
pub mod ingest;
pub mod playlist;

pub use folder::{watchman, Source};
pub use ingest::ingest_server;
pub use playlist::CurrentProgram;

pub fn source_generator(
    config: GlobalConfig,
    current_list: Arc<Mutex<Vec<Media>>>,
    index: Arc<AtomicUsize>,
    playout_stat: PlayoutStatus,
    is_terminated: Arc<AtomicBool>,
) -> Box<dyn Iterator<Item = Media>> {
    let get_source = match config.processing.clone().mode.as_str() {
        "folder" => {
            info!("Playout in folder mode");
            debug!("Monitor folder: <b><magenta>{}</></b>", &config.storage.path);

            let folder_source = Source::new(current_list, index);
            let node_clone = folder_source.nodes.clone();
            thread::spawn(move || watchman(node_clone));

            Box::new(folder_source) as Box<dyn Iterator<Item = Media>>
        }
        "playlist" => {
            info!("Playout in playlist mode");
            let program = CurrentProgram::new(
                playout_stat,
                is_terminated.clone(),
                current_list,
                index,
            );

            Box::new(program) as Box<dyn Iterator<Item = Media>>
        }
        _ => {
            error!("Process Mode not exists!");
            process::exit(0x0100);
        }
    };

    get_source
}