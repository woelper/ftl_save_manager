#![windows_subsystem = "windows"]

use chrono::DateTime;
use chrono::Local;
use imgui::StyleColor::*;
use imgui::*;
use std::{
    fs::copy,
    fs::remove_file,
    io::Read,
    path::{Path, PathBuf},
    sync::mpsc::{self, Receiver, Sender},
    time::SystemTime,
};
mod support;
use anyhow::{Error, Result};
use std::ffi::OsStr;
mod ftldata;
mod update;

#[derive(Clone, Debug)]

struct SaveGame {
    path: PathBuf,
    mtime: SystemTime
}

impl SaveGame {
    fn age(&self) -> String {
        let mtime: DateTime<Local> = self.mtime.into();
        chrono_humanize::HumanTime::from(mtime).to_string()
    }
}

/// Get the FTL save directory for all platforms
fn get_save_directory() -> PathBuf {
    #[cfg(target_os = "linux")]
    {
        dirs::data_dir().unwrap_or_default().join("FasterThanLight")
    }
    #[cfg(target_os = "windows")]
    {
        dirs::document_dir()
            .unwrap_or_default()
            .join("My Games")
            .join("FasterThanLight")
    }
    #[cfg(target_os = "macos")]
    {
        dirs::config_dir()
            .unwrap_or_default()
            .join("FasterThanLight")
    }
}

/// Get the FTL save file
fn get_save_file() -> PathBuf {
    get_save_directory().join("continue.sav")
}

/// Get the FTL save file
fn get_save_info(filepath: &PathBuf) -> Result<String> {
    let mut file = std::fs::File::open(&filepath)?;
    // let mut buf = vec![];

    // file.read_to_end(&mut buf)?;

    use bincode::Options;
    let my_options = bincode::DefaultOptions::new()
        .with_fixint_encoding()
        .allow_trailing_bytes();

    let decoded: ftldata::Sav = my_options.deserialize_from(file)?;

    // let decoded: ftldata::Sav = bincode::deserialize(&buf)?;
    Ok(format!("{:?}", decoded))
}

/// Get modification time
fn get_mtime(p: &Path) -> SystemTime {
    if let Ok(meta) = p.metadata() {
        if let Ok(modified) = meta.modified() {
            return modified;
        }
    }
    SystemTime::now()
}

/// List available savegames
fn get_available_saves() -> Vec<SaveGame> {
    let mut s: Vec<SaveGame> = std::fs::read_dir(get_save_directory())
        .unwrap()
        .filter_map(|e| e.ok())
        .map(|f| f.path().to_path_buf())
        .filter(|f| f.extension() == Some(std::ffi::OsStr::new("msav")))
        .map(|path| SaveGame {
            mtime: get_mtime(&path),
            path,
        })
        .collect();
    // sort files by modification time
    s.sort_by(|a, b| b.mtime.cmp(&a.mtime));
    s
}

/// Creates backup copy of continue.sav
fn backup() {
    let _ = copy(get_save_file(), get_save_directory().join("backup.sav"));
}

fn main() {
    let (update_sender, update_receiver): (Sender<String>, Receiver<String>) = mpsc::channel();

    update::update(update_sender);

    let mut dimensions = (500, 700);
    let mut system = support::init("FTL saves", dimensions);

    let mut savegames = get_available_saves();
    let mut save_name: ImString = im_str!("{}", "unnamed");
    // let mut message: ImString = im_str!("{}", "welcome");
    let mut message = "".to_string();

    system.imgui.style_mut().window_border_size = 0.0;
    let col_main = [0.2, 0.5, 0.8, 1.0];
    system.imgui.style_mut().colors[Button as usize] = col_main;
    system.imgui.style_mut().colors[Tab as usize] = col_main;
    system.imgui.style_mut().colors[ChildBg as usize] = col_main;
    system.imgui.style_mut().colors[TabUnfocusedActive as usize] = [0.2, 0.2, 0.2, 1.0];
    system.imgui.style_mut().colors[WindowBg as usize] = [0.2, 0.2, 0.2, 1.0];

    system.main_loop(move |_, ui| {
        if let Ok(msg) = update_receiver.try_recv() {
            message = msg;
        }

        let s = ui.io().display_size;
        dimensions.0 = s[0] as u32;
        dimensions.1 = s[1] as u32;
        Window::new(im_str!("Savegames"))
            .position([0.0, 0.0], Condition::FirstUseEver)
            .movable(false)
            .no_decoration()
            .size(
                [dimensions.0 as f32, dimensions.1 as f32],
                Condition::Always,
            )
            .build(ui, || {
                TabBar::new(im_str!("basictabbar")).build(&ui, || {
                    TabItem::new(im_str!("Save/Load")).build(&ui, || {
                        ui.text(im_str!("Enter a name to save the current game."));
                        ui.text(im_str!("This can be done at any time."));

                        ui.input_text(im_str!("Save name"), &mut save_name)
                            .resize_buffer(true)
                            .build();

                        if ui.button(&im_str!("Save \"{}\"", save_name), [-1., 0.]) {
                            let savegame = get_save_directory()
                                .join(save_name.to_string())
                                .with_extension("msav");
                            match copy(get_save_file(), &savegame) {
                                Ok(_) => {
                                    message = format!(
                                        "Saved {} successfully.",
                                        savegame
                                            .file_name()
                                            .unwrap_or(OsStr::new(""))
                                            .to_string_lossy()
                                    );
                                    savegames = get_available_saves()
                                }
                                Err(e) => message = format!("Could not save: {}", e),
                            }
                        }

                        ui.text(&im_str!("{}", message));

                        ui.dummy([0., 60.]);
                        ui.text(im_str!(
                            "Select a game to load. Make sure the game is closed."
                        ));

                        if savegames.is_empty() {
                            ui.text("There are no savegames yet. Please save one!");
                        }

                        for savegame in &savegames.clone() {
                            if ui.button(
                                &im_str!(
                                    "Restore {}",
                                    savegame.path.file_name().unwrap_or_default().to_string_lossy()
                                ),
                                [dimensions.0 as f32 - 60., 0.],
                            ) {
                                backup();
                                let _ = copy(&savegame.path, get_save_file());
                            }

                            if ui.is_item_hovered() {
                                ui.tooltip(|| {
                                    if let Ok(save) = ftldata::get_save_info(&savegame.path) {
                                        ui.text(&im_str!("{}", save));
                                        ui.text(&im_str!("{}", savegame.age()));
                                        //ui.text(&im_str!("{}", savegame.));
                                        
                                    }
                                });
                            }

                            ui.same_line(0.0);

                            if ui.button(&im_str!("DEL##{:?}", savegame.path), [40., 0.]) {
                                if remove_file(&savegame.path).is_ok() {
                                    message = format!(
                                        "Removed {}.",
                                        savegame.path
                                            .file_name()
                                            .unwrap_or(OsStr::new(""))
                                            .to_string_lossy()
                                    );
                                }
                                savegames = get_available_saves();
                            }
                        }
                    });
                    TabItem::new(im_str!("Settings")).build(&ui, || {
                        ui.text(im_str!("Nothing here yet..."));
                    });
                });
            });
    });
}
