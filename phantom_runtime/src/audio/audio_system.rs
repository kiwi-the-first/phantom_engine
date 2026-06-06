use std::io::Cursor;

use phantom_core::audio::{AudioCommand, AudioContext};
use rodio::{Decoder, MixerDeviceSink, Player, mixer::Mixer};

/// Host-side audio backend, mirroring `InputSystem` / `TimeSystem`: owns the rodio
/// machinery and exposes a script-facing [`AudioContext`] (`audio_ctx`).
///
/// Scripts queue [`AudioCommand`]s through `audio_ctx`; the host calls
/// [`AudioSystem::update`] once per frame to start those sounds and reap finished
/// ones. `rodio` lives here, not in `phantom_core`, so it stays out of the game dylib.
pub struct AudioSystem {
    pub audio_ctx: AudioContext,
    /// `None` if no audio device could be opened — audio then no-ops gracefully.
    device: Option<MixerDeviceSink>,
    /// Live sounds; a `Player` must stay alive for its sound to keep playing.
    players: Vec<Player>,
}

impl Default for AudioSystem {
    fn default() -> Self {
        let device = match rodio::DeviceSinkBuilder::open_default_sink() {
            Ok(device) => Some(device),
            Err(e) => {
                log::error!("Audio disabled: failed to open default sink: {e}");
                None
            }
        };
        Self {
            audio_ctx: AudioContext::default(),
            device,
            players: Vec::new(),
        }
    }
}

impl AudioSystem {
    /// Drain everything scripts queued this frame, start each sound, then drop any
    /// `Player`s whose sound has finished (otherwise finished handles pile up).
    pub fn update(&mut self) {
        for command in self.audio_ctx.drain() {
            if let Some(device) = &self.device {
                match Self::start(device.mixer(), command) {
                    Ok(player) => self.players.push(player),
                    Err(e) => log::error!("Failed to play audio: {e}"),
                }
            }
        }
        self.players.retain(|player| !player.empty());
    }

    /// Stop everything and discard any not-yet-played commands. Dropping a
    /// `Player` stops its sound, so clearing the vec silences all playback —
    /// e.g. when leaving play mode in the editor.
    pub fn stop_all(&mut self) {
        self.players.clear();
        let _ = self.audio_ctx.drain();
    }

    fn start(mixer: &Mixer, command: AudioCommand) -> anyhow::Result<Player> {
        let cursor = Cursor::new(command.data);
        let player = Player::connect_new(mixer);
        // `looping` and one-shot decode to different source types, so append in
        // each branch rather than unifying.
        if command.looping {
            player.append(Decoder::new_looped(cursor)?);
        } else {
            player.append(Decoder::new(cursor)?);
        }
        player.set_volume(command.volume);
        Ok(player)
    }
}
