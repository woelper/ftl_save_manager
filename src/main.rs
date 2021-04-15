use imgui::*;
use std::{fs::copy, path::PathBuf};

mod support;

/// Get the FTL save directory for all platforms
fn get_save_directory() -> PathBuf {
    #[cfg(target_os = "linux")]
    {
        dirs::data_dir().unwrap_or_default().join("FasterThanLight")
    }
    #[cfg(target_os = "windows")]
    {
        dirs::data_dir().unwrap_or_default().join("FasterThanLight")
    }
    #[cfg(target_os = "macos")]
    {
        dirs::data_dir().unwrap_or_default().join("FasterThanLight")
    }
}

/// Get the FTL save file
fn get_save_file() -> PathBuf {
    get_save_directory().join("continue.sav")
}

fn get_available_saves() -> Vec<PathBuf> {
    std::fs::read_dir(get_save_directory())
        .unwrap()
        .filter_map(|e| e.ok())
        .map(|f| f.path().to_path_buf())
        .filter(|f| f.extension() == Some(std::ffi::OsStr::new("msav")))
        .collect()
}

/// Creates backup copy of continue.sav
fn backup() {
    let _ = copy(get_save_file(), get_save_directory().join("backup.sav"));
}


fn main() {
    dbg!(get_save_directory());
    let system = support::init(file!());

    let mut savegames = get_available_saves();
    let mut save_name: ImString = im_str!("{}", "unnamed");

    system.main_loop(move |_, ui| {
        Window::new(im_str!("Savegames"))
            .size([300.0, 110.0], Condition::FirstUseEver)
            .build(ui, || {
                ui.text(im_str!("Available savegames"));
                ui.separator();

                for savegame in &savegames {

                    if 
                    ui.button(
                        &im_str!(
                            "Restore {}",
                            savegame.file_name().unwrap_or_default().to_string_lossy()
                        ),
                        [0., 0.],
                    ) {
                        backup();
                        let _ = copy(savegame, get_save_file());
                    }
                }
            });

        Window::new(im_str!("Save active game"))
            .size([300.0, 210.0], Condition::FirstUseEver)
            .build(ui, || {
                ui.input_text(im_str!("Save name"), &mut save_name).build();

                if ui.button(&im_str!("Save {}", save_name), [0., 0.]) {
                    match copy(
                        get_save_file(),
                        get_save_directory()
                            .join(save_name.to_string())
                            .with_extension("msav"),
                    ) {
                        Ok(_) => savegames = get_available_saves(),
                        Err(e) => eprintln!("{:?}", e),
                    }
                }
            });
    });
}
