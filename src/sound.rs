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
                Err(e) => {
                    eprintln!("Failed to initialize audio output: {}", e);
                    return;
                }
            };
            let mut current_sink: Option<Sink> = None;

            while let Ok(cmd) = receiver.recv() {
                match cmd {
                    SoundCommand::Play(sound_path) => {

                        // Stop current sound
                        if let Some(sink) = current_sink.take() {
                            sink.stop();
                        }

                        // Try to open and play the file
                        match File::open(&sound_path) {
                            Ok(file) => {
                                let reader = BufReader::new(file);
                                match rodio::Decoder::new(reader) {
                                    Ok(source) => match Sink::try_new(&stream_handle) {
                                        Ok(sink) => {
                                            sink.append(source);
                                            current_sink = Some(sink);
                                        }
                                        Err(e) => eprintln!("Failed to create sink: {}", e),
                                    },
                                    Err(e) => eprintln!("Failed to decode audio: {}", e),
                                }
                            }
                            Err(e) => eprintln!("Sound file not found: {} ({})", sound_path, e),
                        }
                    }
                    SoundCommand::Stop => {
                        if let Some(sink) = current_sink.take() {
                            sink.stop();
                            eprintln!("Sound stopped");
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

    // Check for common audio file extensions
    for ext in &["", ".mp3", ".wav", ".ogg"] {
        let path = sound_dir.join(format!("{}{}", name, ext));
        if path.exists() {
            return Some(path);
        }
    }

    // Also check absolute paths
    let path = PathBuf::from(name);
    if path.exists() {
        return Some(path);
    }

    None
}

pub fn notification(title: &str, body: &str) {
    use notify_rust::Notification;

    let _ = Notification::new()
        .summary("tock")
        .subtitle(title)
        .body(body)
        .show();
}
