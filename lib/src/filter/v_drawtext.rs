use std::{
    ffi::OsStr,
    path::Path,
    sync::{Arc, Mutex},
};

use regex::Regex;
use simplelog::*;

use crate::utils::{controller::ProcessUnit::*, custom_format, Media, PlayoutConfig};
use crate::ADVANCED_CONFIG;

pub fn filter_node(
    config: &PlayoutConfig,
    node: Option<&Media>,
    filter_chain: &Option<Arc<Mutex<Vec<String>>>>,
) -> String {
    let mut filter = String::new();
    let mut font = String::new();

    if Path::new(&config.text.fontfile).is_file() {
        font = format!(":fontfile='{}'", config.text.fontfile)
    }

    let zmq_socket = match node.map(|n| n.unit) {
        Some(Ingest) => config.text.zmq_server_socket.clone(),
        _ => config.text.zmq_stream_socket.clone(),
    };

    if config.text.text_from_filename && node.is_some() {
        let source = node.unwrap_or(&Media::new(0, "", false)).source.clone();
        let text = match Regex::new(&config.text.regex)
            .ok()
            .and_then(|r| r.captures(&source))
        {
            Some(t) => t[1].to_string(),
            None => Path::new(&source)
                .file_stem()
                .unwrap_or_else(|| OsStr::new(&source))
                .to_string_lossy()
                .to_string(),
        };

        let escaped_text = text
            .replace('\'', "'\\\\\\''")
            .replace('%', "\\\\\\%")
            .replace(':', "\\:");

        filter = match &ADVANCED_CONFIG.decoder.filters.drawtext_from_file {
            Some(drawtext) => custom_format(drawtext, &[&escaped_text, &config.text.style, &font]),
            None => format!("drawtext=text='{escaped_text}':{}{font}", config.text.style),
        };
    // } else if let Some(socket) = zmq_socket {
    //     let mut filter_cmd = format!("text=''{font}");

    //     if let Some(chain) = filter_chain {
    //         if let Some(link) = chain.lock().unwrap().iter().find(|&l| l.contains("text")) {
    //             filter_cmd = link.to_string();
    //         }
    //     }

    //     filter = match &ADVANCED_CONFIG.decoder.filters.drawtext_from_zmq {
    //         Some(drawtext) => custom_format(drawtext, &[&socket.replace(':', "\\:"), &filter_cmd]),
    //         None => format!(
    //             "zmq=b=tcp\\\\://'{}',drawtext@dyntext={filter_cmd}",
    //             socket.replace(':', "\\:")
    //         ),
    //     };
    } else {
        // let text = config.text.regex.clone();
        let text = format!("{}", config.text.regex.clone()).to_string();
        if text.starts_with("==file==") {
            let file_name = text.replace("==file==", "");
            // let file_name = text.replace("%%file:", "");
            filter = match &ADVANCED_CONFIG.decoder.filters.drawtext_from_file {
                Some(drawtext) => custom_format(drawtext, &[&file_name, &config.text.style, &font]),
                None => format!("drawtext=textfile='{file_name}':{}{font}:reload=1", config.text.style),
            };    
        } else {
            filter = match &ADVANCED_CONFIG.decoder.filters.drawtext_from_file {
                Some(drawtext) => custom_format(drawtext, &[&text, &config.text.style, &font]),
                None => format!("drawtext=text='{text}':{}{font}", config.text.style),
            };
        }
    }


    filter
}
