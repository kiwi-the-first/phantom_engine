use std::{cell::RefCell, rc::Rc};

/// A single queued playback request. Pure data — the host's `AudioSystem` turns
/// these into rodio `Player`s after the script pass. No rodio types here, so this
/// (and all of `phantom_core`) stays free of the audio backend.
///
/// `data` is an *owned* copy of the encoded audio bytes. Scripts pass
/// `include_bytes!(..)` (a `&'static [u8]` living in the dylib); copying it to a
/// host-owned `Vec` means a still-playing sound survives a dylib hot-reload that
/// would otherwise free those bytes.
#[derive(Clone)]
pub struct AudioCommand {
    pub data: Vec<u8>,
    pub looping: bool,
    pub volume: f32,
}

/// Script-facing audio surface, mirroring `InputContext` / `TimeContext`.
///
/// Unlike those (which scripts only *read*), audio is a *write*: scripts hold a
/// shared `&AudioContext`, so `.play()` can't start a sound directly. Instead the
/// builder pushes an [`AudioCommand`] into an interior-mutable queue, which the
/// host drains via [`AudioContext::drain`] — the same deferred-command pattern
/// used elsewhere in the editor.
pub struct AudioContext {
    queue: Rc<RefCell<Vec<AudioCommand>>>,
    /// Entry point for the builder API: `ctx.audio.player.with_file(..).play()`.
    pub player: PlayerFactory,
}

impl Default for AudioContext {
    fn default() -> Self {
        let queue = Rc::new(RefCell::new(Vec::new()));
        Self {
            player: PlayerFactory {
                queue: Rc::clone(&queue),
            },
            queue,
        }
    }
}

impl AudioContext {
    /// Host-side: take everything scripts queued this frame, leaving the queue empty.
    pub fn drain(&self) -> Vec<AudioCommand> {
        self.queue.borrow_mut().drain(..).collect()
    }
}

/// The `player` handle on [`AudioContext`]. Begins a builder chain.
pub struct PlayerFactory {
    queue: Rc<RefCell<Vec<AudioCommand>>>,
}

impl PlayerFactory {
    /// Start a builder from encoded audio bytes — typically `include_bytes!(..)`.
    /// The bytes are copied into the command (see [`AudioCommand`]).
    pub fn with_bytes(&self, bytes: &[u8]) -> PlayerBuilder {
        PlayerBuilder {
            queue: Rc::clone(&self.queue),
            command: AudioCommand {
                data: bytes.to_vec(),
                looping: false,
                volume: 1.0,
            },
        }
    }
}

/// Configures one sound, then queues it on `.play()`. Nothing happens until
/// `.play()` is called.
#[must_use = "call .play() to actually queue the sound"]
pub struct PlayerBuilder {
    queue: Rc<RefCell<Vec<AudioCommand>>>,
    command: AudioCommand,
}

impl PlayerBuilder {
    /// Loop the sound until its `Player` is dropped (`loop` is a reserved word).
    pub fn looping(mut self) -> Self {
        self.command.looping = true;
        self
    }

    /// Linear volume multiplier (1.0 = original).
    pub fn volume(mut self, volume: f32) -> Self {
        self.command.volume = volume;
        self
    }

    /// Queue the configured sound for the host to play after the script pass.
    pub fn play(self) {
        self.queue.borrow_mut().push(self.command);
    }
}
