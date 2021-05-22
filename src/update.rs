use self_update::{cargo_crate_version, Status};
use std::{sync::mpsc::Sender, thread};

fn gh_update() -> Result<Status, Box<dyn std::error::Error>> {
    #[cfg(target_os = "windows")]
    let mut target = "";
    #[cfg(target_os = "linux")]
    let target = "_linux";
    #[cfg(target_os = "macos")]
    let target = "_mac";

    let status = self_update::backends::github::Update::configure()
        .repo_owner("woelper")
        .repo_name("ftl_save_manager")
        .bin_name("ftlsavemanager")
        .target(target)
        .current_version(cargo_crate_version!())
        .no_confirm(true)
        .build()?
        .update()?;
    println!("Update status: `{:?}`!", status);
    Ok(status)
}

pub fn update(sender: Sender<String>) {
    thread::spawn(move || match gh_update() {
        Ok(s) => {
            let msg = match s {
                Status::UpToDate(ver) => {
                    format!("Welcome! You are running the latest version: {}", ver)
                }
                Status::Updated(ver) => format!(
                    "Welcome! You have been updated to: {}, please restart!",
                    ver
                ),
            };
            let _ = sender.send(msg);
        }
        Err(e) => {
            eprintln!("Update error{:?}", e);
            let _ = sender.send(format!("Can't check for a new version."));
        }
    });
}
