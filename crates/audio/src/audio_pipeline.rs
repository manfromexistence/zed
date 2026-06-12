use anyhow::{Context as _, Result};
use collections::HashMap;
use cpal::{
    DeviceDescription, DeviceId, default_host,
    traits::{DeviceTrait, HostTrait},
};
use gpui::{App, AsyncApp, BorrowAppContext, Global};

pub(super) use cpal::Sample;

use rodio::{Decoder, DeviceSinkBuilder, MixerDeviceSink, Source, mixer::Mixer, source::Buffered};
use settings::Settings;
use std::{
    io::Cursor,
    path::Path,
    sync::{
        Arc,
        atomic::{AtomicBool, Ordering},
    },
    time::Instant,
};
use util::ResultExt;

mod echo_canceller;
use echo_canceller::EchoCanceller;
mod rodio_ext;
pub use crate::audio_settings::AudioSettings;
pub use rodio_ext::RodioExt;

use crate::audio_settings::LIVE_SETTINGS;

use crate::{DxSoundEvent, DxSoundPolicy, Sound};

use super::{CHANNEL_COUNT, SAMPLE_RATE};
pub const BUFFER_SIZE: usize = // echo canceller and livekit want 10ms of audio
    (SAMPLE_RATE.get() as usize / 100) * CHANNEL_COUNT.get() as usize;

pub fn init(cx: &mut App) {
    LIVE_SETTINGS.initialize(cx);
}

// TODO(jk): this is currently cached only once - we should observe and react instead
pub fn ensure_devices_initialized(cx: &mut App) {
    if cx.has_global::<AvailableAudioDevices>() {
        return;
    }
    cx.default_global::<AvailableAudioDevices>();
    let task = cx
        .background_executor()
        .spawn(async move { get_available_audio_devices() });
    cx.spawn(async move |cx: &mut AsyncApp| {
        let devices = task.await;
        cx.update(|cx| cx.set_global(AvailableAudioDevices(devices)));
        cx.refresh();
    })
    .detach();
}

#[derive(Default)]
pub struct Audio {
    output: Option<(MixerDeviceSink, Mixer)>,
    pub echo_canceller: EchoCanceller,
    source_cache: HashMap<Sound, Buffered<Decoder<Cursor<Vec<u8>>>>>,
    dx_sound_last_played: HashMap<DxSoundEvent, Instant>,
}

impl Global for Audio {}

#[derive(Clone, Debug)]
pub struct AudioPlaybackHandle {
    state: Arc<AudioPlaybackState>,
}

#[derive(Debug)]
struct AudioPlaybackState {
    canceled: AtomicBool,
    completed: AtomicBool,
}

struct TrackedAudioSource<S> {
    inner: S,
    state: Arc<AudioPlaybackState>,
}

impl AudioPlaybackHandle {
    pub fn cancel(&self) {
        self.state.canceled.store(true, Ordering::Relaxed);
    }

    pub fn is_complete(&self) -> bool {
        self.state.completed.load(Ordering::Relaxed)
    }
}

impl AudioPlaybackState {
    fn new() -> Arc<Self> {
        Arc::new(Self {
            canceled: AtomicBool::new(false),
            completed: AtomicBool::new(false),
        })
    }
}

impl<S> Iterator for TrackedAudioSource<S>
where
    S: Source,
{
    type Item = rodio::Sample;

    fn next(&mut self) -> Option<Self::Item> {
        if self.state.canceled.load(Ordering::Relaxed) {
            self.state.completed.store(true, Ordering::Relaxed);
            return None;
        }

        let sample = self.inner.next();
        if sample.is_none() {
            self.state.completed.store(true, Ordering::Relaxed);
        }
        sample
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        self.inner.size_hint()
    }
}

impl<S> Source for TrackedAudioSource<S>
where
    S: Source,
{
    fn current_span_len(&self) -> Option<usize> {
        self.inner.current_span_len()
    }

    fn channels(&self) -> rodio::ChannelCount {
        self.inner.channels()
    }

    fn sample_rate(&self) -> rodio::SampleRate {
        self.inner.sample_rate()
    }

    fn total_duration(&self) -> Option<std::time::Duration> {
        self.inner.total_duration()
    }
}

impl<S> Drop for TrackedAudioSource<S> {
    fn drop(&mut self) {
        self.state.completed.store(true, Ordering::Relaxed);
    }
}

impl Audio {
    fn ensure_output_exists(&mut self, output_audio_device: Option<DeviceId>) -> Result<&Mixer> {
        if self.output.is_none() {
            #[cfg(debug_assertions)]
            log::debug!(
                "Audio does not sound correct without optimizations. Use a release build to debug audio issues"
            );

            let (output_handle, output_mixer) =
                open_output_stream(output_audio_device, self.echo_canceller.clone())?;
            self.output = Some((output_handle, output_mixer));
        }

        Ok(self
            .output
            .as_ref()
            .map(|(_, mixer)| mixer)
            .expect("we only get here if opening the outputstream succeeded"))
    }

    pub fn play_sound(sound: Sound, cx: &mut App) {
        let output_audio_device = AudioSettings::get_global(cx).output_audio_device.clone();
        cx.update_default_global(|this: &mut Self, cx| {
            let source = this.sound_source(sound, cx).log_err()?;
            let output_mixer = this
                .ensure_output_exists(output_audio_device)
                .context("Could not get output mixer")
                .log_err()?;

            output_mixer.add(source);
            Some(())
        });
    }

    pub fn play_dx_sound(event: DxSoundEvent, cx: &mut App) {
        let (output_audio_device, dx_interaction_sounds, dx_sounds) = {
            let settings = AudioSettings::get_global(cx);
            (
                settings.output_audio_device.clone(),
                settings.dx_interaction_sounds,
                settings.dx_sounds,
            )
        };

        if !dx_sounds {
            return;
        }

        if event.policy() == DxSoundPolicy::ExplicitOptIn && !dx_interaction_sounds {
            return;
        }

        cx.update_default_global(|this: &mut Self, cx| {
            if !this.should_play_dx_sound(event) {
                return Some(());
            }

            let source = this.sound_source(event.sound(), cx).log_err()?;
            let output_mixer = this
                .ensure_output_exists(output_audio_device)
                .context("Could not get output mixer")
                .log_err()?;

            output_mixer.add(source.amplify(event.gain()));
            Some(())
        });
    }

    fn should_play_dx_sound(&mut self, event: DxSoundEvent) -> bool {
        let now = Instant::now();
        if let Some(last_played) = self.dx_sound_last_played.get(&event)
            && now.duration_since(*last_played) < event.cooldown()
        {
            return false;
        }

        self.dx_sound_last_played.insert(event, now);
        true
    }

    pub fn play_wav_file(path: &Path, cx: &mut App) -> Result<()> {
        Self::play_wav_file_tracked(path, cx).map(|_| ())
    }

    pub fn play_wav_file_tracked(path: &Path, cx: &mut App) -> Result<AudioPlaybackHandle> {
        let output_audio_device = AudioSettings::get_global(cx).output_audio_device.clone();
        let bytes = std::fs::read(path)
            .with_context(|| format!("Could not read WAV file {}", path.display()))?;

        cx.update_default_global(|this: &mut Self, _cx| {
            let source = Decoder::new(Cursor::new(bytes))?;
            let state = AudioPlaybackState::new();
            let handle = AudioPlaybackHandle {
                state: Arc::clone(&state),
            };
            let source = TrackedAudioSource {
                inner: source,
                state,
            };
            let output_mixer = this
                .ensure_output_exists(output_audio_device)
                .context("Could not get output mixer")?;

            // TTS (Kokoro read-aloud) volume is set to 5% to keep it non-intrusive.
            output_mixer.add(source.amplify(0.05));
            Ok(handle)
        })
    }

    pub fn end_call(cx: &mut App) {
        cx.update_default_global(|this: &mut Self, _cx| {
            this.output.take();
        });
    }

    fn sound_source(&mut self, sound: Sound, cx: &App) -> Result<impl Source + use<>> {
        if let Some(wav) = self.source_cache.get(&sound) {
            return Ok(wav.clone());
        }

        let path = format!("sounds/{}.wav", sound.file());
        let bytes = cx
            .asset_source()
            .load(&path)?
            .map(anyhow::Ok)
            .with_context(|| format!("No asset available for path {path}"))??
            .into_owned();
        let cursor = Cursor::new(bytes);
        let source = Decoder::new(cursor)?.buffered();

        self.source_cache.insert(sound, source.clone());

        Ok(source)
    }
}

pub fn open_input_stream(
    device_id: Option<DeviceId>,
) -> anyhow::Result<rodio::microphone::Microphone> {
    let builder = rodio::microphone::MicrophoneBuilder::new();
    let builder = if let Some(id) = device_id {
        // TODO(jk): upstream patch
        // if let Some(input_device) = default_host().device_by_id(id) {
        //     builder.device(input_device);
        // }
        let mut found = None;
        for input in rodio::microphone::available_inputs()? {
            if input.clone().into_inner().id()? == id {
                found = Some(builder.device(input));
                break;
            }
        }
        found.unwrap_or_else(|| builder.default_device())?
    } else {
        builder.default_device()?
    };
    let stream = builder
        .default_config()?
        .prefer_sample_rates([
            SAMPLE_RATE,
            SAMPLE_RATE.saturating_mul(rodio::nz!(2)),
            SAMPLE_RATE.saturating_mul(rodio::nz!(3)),
            SAMPLE_RATE.saturating_mul(rodio::nz!(4)),
        ])
        .prefer_channel_counts([rodio::nz!(1), rodio::nz!(2), rodio::nz!(3), rodio::nz!(4)])
        .prefer_buffer_sizes(512..)
        .open_stream()?;
    log::info!("Opened microphone: {:?}", stream.config());
    Ok(stream)
}

pub fn resolve_device(device_id: Option<&DeviceId>, input: bool) -> anyhow::Result<cpal::Device> {
    if let Some(id) = device_id {
        if let Some(device) = default_host().device_by_id(id) {
            return Ok(device);
        }
        log::warn!("Selected audio device not found, falling back to default");
    }
    if input {
        default_host()
            .default_input_device()
            .context("no audio input device available")
    } else {
        default_host()
            .default_output_device()
            .context("no audio output device available")
    }
}

pub fn open_test_output(device_id: Option<DeviceId>) -> anyhow::Result<MixerDeviceSink> {
    let device = resolve_device(device_id.as_ref(), false)?;
    DeviceSinkBuilder::from_device(device)?
        .open_stream()
        .context("Could not open output stream")
}

pub fn open_output_stream(
    device_id: Option<DeviceId>,
    mut echo_canceller: EchoCanceller,
) -> anyhow::Result<(MixerDeviceSink, Mixer)> {
    let device = resolve_device(device_id.as_ref(), false)?;
    let mut output_handle = DeviceSinkBuilder::from_device(device)?
        .open_stream()
        .context("Could not open output stream")?;
    output_handle.log_on_drop(false);
    log::info!("Output stream: {:?}", output_handle);

    let (output_mixer, source) = rodio::mixer::mixer(CHANNEL_COUNT, SAMPLE_RATE);
    // otherwise the mixer ends as it's empty
    output_mixer.add(rodio::source::Zero::new(CHANNEL_COUNT, SAMPLE_RATE));
    let echo_cancelling_source = source // apply echo cancellation just before output
        .inspect_buffer::<BUFFER_SIZE, _>(move |buffer| {
            let mut buf: [i16; _] = buffer.map(|s| s.to_sample());
            echo_canceller.process_reverse_stream(&mut buf)
        });
    output_handle.mixer().add(echo_cancelling_source);

    Ok((output_handle, output_mixer))
}

#[derive(Clone, Debug)]
pub struct AudioDeviceInfo {
    pub id: DeviceId,
    pub desc: DeviceDescription,
}

impl AudioDeviceInfo {
    pub fn matches_input(&self, is_input: bool) -> bool {
        if is_input {
            self.desc.supports_input()
        } else {
            self.desc.supports_output()
        }
    }

    pub fn matches(&self, id: &DeviceId, is_input: bool) -> bool {
        &self.id == id && self.matches_input(is_input)
    }
}

impl std::fmt::Display for AudioDeviceInfo {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} ({})", self.desc.name(), self.id)
    }
}

fn get_available_audio_devices() -> Vec<AudioDeviceInfo> {
    let Some(devices) = default_host().devices().ok() else {
        return Vec::new();
    };
    devices
        .filter_map(|device| {
            let id = device.id().ok()?;
            let desc = device.description().ok()?;
            Some(AudioDeviceInfo { id, desc })
        })
        .collect()
}

#[derive(Default, Clone, Debug)]
pub struct AvailableAudioDevices(pub Vec<AudioDeviceInfo>);

impl Global for AvailableAudioDevices {}
