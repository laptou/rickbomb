#![windows_subsystem = "windows"]

use std::io::Write;

use anyhow::Context;
use gui::rickroll;
use log::*;

mod gui;
mod interop;
mod keylog;
mod window_target;

fn main() -> anyhow::Result<()> {
    pretty_env_logger::init();

    let video_bytes = include_bytes!("../rick.mkv");

    let video_path = std::env::current_dir()
        .map(|dir| dir.join("rick.mkv"))
        .context("could not canonicalize rick.mkv")?;

    {
        let mut rick_file =
            std::fs::File::create(&video_path).context("could not create rick file")?;

        rick_file
            .write(video_bytes)
            .context("failed to write rick bytes")?;

        rick_file.flush()?;
    }

    let terminate_tx = keylog::init_keylogger();

    let mut keystroke_rx =
        keylog::listen_keylogger().context("could not get keystroke receiver")?;

    for keystroke in keystroke_rx.iter() {
        trace!("received key: {:?}", keystroke);

        // R key
        if keystroke.vk_code == 0x52 {
            break;
        }
    }

    let _ = terminate_tx.send(());

    rickroll(&video_path);

    std::fs::remove_file(video_path);

    Ok(())
}
