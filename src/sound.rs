use rodio::{OutputStream, Sink};
use std::fs::File;
use std::io::BufReader;
use std::path::PathBuf;
use std::sync::mpsc::{self, Sender};
use std::thread;

pub struct SoundPlayer {
    sender: Sender<SoundCommand>,
}

enum SoundCommand {
    Play(String),
    Stop,
}

impl SoundPlayer {
    pub fn new() -> Self {
        let (sender, receiver) = mpsc::channel();

        thread::spawn(move || {
            let (_stream, stream_handle) = match OutputStream::try_default() {
                Ok(s) => s,
                Err(_) => return,
            };
            let mut current_sink: Option<Sink> = None;

            while let Ok(cmd) = receiver.recv() {
                match cmd {
                    SoundCommand::Play(sound_path) => {
                        if let Some(sink) = current_sink.take() {
                            sink.stop();
                        }

                        if let Ok(sink) = Sink::try_new(&stream_handle) {
                            if let Ok(file) = File::open(&sound_path) {
                                let reader = BufReader::new(file);
                                if let Ok(source) = rodio::Decoder::new(reader) {
                                    sink.append(source);
                                    current_sink = Some(sink);
                                }
                            }
                        }
                    }
                    SoundCommand::Stop => {
                        if let Some(sink) = current_sink.take() {
                            sink.stop();
                        }
                    }
                }
            }
        });

        Self { sender }
    }

    pub fn play(&self, sound_path: &str) {
        let _ = self.sender.send(SoundCommand::Play(sound_path.to_string()));
    }

    pub fn stop(&self) {
        let _ = self.sender.send(SoundCommand::Stop);
    }
}

pub fn get_sound_path(name: &str) -> Option<PathBuf> {
    let config_dir = directories::ProjectDirs::from("", "", "clock")
        .map(|dirs| dirs.config_dir().to_path_buf())?;

    let sound_dir = config_dir.join("sounds");

    if name == "default" {
        return Some(sound_dir.join("alarm.mp3"));
    }

    let ext = if name.ends_with(".mp3") || name.ends_with(".wav") || name.ends_with(".ogg") {
        ""
    } else {
        ".mp3"
    };

    Some(sound_dir.join(format!("{}{}", name, ext)))
}

pub fn notification(title: &str, body: &str) {
    use notify_rust::Notification;

    let _ = Notification::new()
        .appname("tock")
        .subtitle(title)
        .body(body)
        .show();
}
