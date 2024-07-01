use std::process::Command;

use log::error;

#[derive(Default)]
pub struct Killer(Vec<u32>);

impl Killer {
    pub fn add_process(&mut self, pid: u32) {
        self.0.push(pid);
    }
}

impl Drop for Killer {
    fn drop(&mut self) {
        self.0
            .iter()
            .for_each(|pid| {
                let command = Command::new("kill")
                    .args(["-s", "INT", &pid.to_string()])
                    .output();
                if let Err(e) = command {
                    error!(r#"err Command::new("kill"): {}, pid: {}"#, e, pid);
                }
            });
    }
}