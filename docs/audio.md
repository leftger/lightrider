# Audio & Procedural Music System

**LIGHTRIDER** features a custom audio synthesizer running in a dedicated realtime thread powered by [Glicol](https://glicol.org) and `cpal`. No external pre-rendered WAV or MP3 files are used: all tracks and effects are synthesized on the fly!

---

## 1. The Title Menu Theme Song

When LIGHTRIDER launches, the main menu immediately begins playing an iconic, dedicated synthesized title theme:

* **Aesthetic**: Energetic Electro / Sci-Fi / Frutiger-Aero (inspired by Daft Punk TRON: Legacy, Wipeout HD, Mirror's Edge by Solar Fields, and futuristic mid-2000s cyber aesthetics).
* **Tempo**: Driving 128.0 BPM.
* **Key & Harmony**: D Major / Lydian (bright, optimistic, crystalline, radiant).
* **Synthesizer Layers**:
  * **Pumping French Touch Sidechain (`~menu_pump`)**: A dynamic sine envelope (`sin 2.133 >> mul 0.35 >> add 0.65`) that ducks pads, leads, and basses behind the kick.
  * **4-on-the-Floor Electro Kick (`~menu_kick`)**: Resonant bassdrum impulse (`imp 2.133 >> bd 0.075`).
  * **Cyber Backbeat Snare (`~menu_snare`)**: Crisp electronic snare on beats 2 & 4 (`imp 1.067 >> sn 0.048`).
  * **Rolling 16th-Note Hi-Hats (`~menu_hat`)**: Fast, shimmering stereo-panned hats (`imp 8.533 >> hh 0.015`).
  * **Deep Sub-Bass (`~menu_sub`)**: Clean sub sine wave rooted at D2 (73.42 Hz).
  * **Sawtooth Electro Bass (`~menu_bass`)**: Resonant filtered sawtooth bassline ducked by sidechain.
  * **Frutiger Aero Crystalline Ambient Pads (`~aero_pad0` & `~aero_pad1`)**: Lush, glassy triangle/saw chord pads with slow breathing filter sweep LFOs.
  * **Glassy Aquatic Bells (`~aero_bell`)**: High-frequency crystal chime pulses at D6 (1174.66 Hz).
  * **Soaring Sci-Fi Cyber Lead (`~aero_lead`)**: Uplifting square-wave melody at D5 (587.33 Hz) with resonant vibrato sweep.
  * **Particle Shimmer (`~aero_shimmer`)**: Airy high-frequency crystal texture at A6 (1760.00 Hz).
  * **Plate Reverb**: Vast, luminous spatial ambiance (`>> plate 0.32`).

---

## 2. Deterministic Procedural Directory Themes

Once you enter the grid, your filesystem itself acts as the musical score:
* **Path-Seeded Themes**: Every folder's path is hashed into a stable musical fingerprint — key, scale (Major, Minor, Dorian, Phrygian, Harmonic Minor, Cyberpunk, etc.), tempo, and timbre family (Glass, Pulse, Reed, Brass). Visiting the same folder always produces the same score.
* **Mode Profiles**:
  * **Menu (`ModeProfile::Menu`)**: Plays the dedicated 128 BPM Frutiger-Aero title track.
  * **Calm (`ModeProfile::Calm`)**: In 3D Explorer mode, plays a meditative, spacious ambient arrangement with warm pads, war-horn drones, and gentle pulse.
  * **Action (`ModeProfile::Action`)**: In Lightcycle mode, ramps up tempo and layers on driving 4-on-the-floor electro drums, rolling hats, sidechain pump, and aggressive basslines.
* **Spatial Proximity Voice Mixing**: Each file and folder in the active directory corresponds to a synthesizer voice on the ground plane. Moving the camera or riding the bike near blocks brings their unique harmonic tones into the mix.

---

## 3. Realtime Sound Effects

Gameplay sound effects are dynamically synthesized on dedicated Glicol reference chains:
* **Turn**: Pitched cyber blip when queuing turns.
* **Crash / Derezz**: Descending noise impact.
* **Beam Transport**: Rising frequency sweep when entering directories.
* **Portal**: Bright harmonic shimmer when returning to parents.
* **Victory**: Uplifting fanfare when clearing rings.
* **Memory Flood Wall**: Directional, 3D-panned low-frequency warning growl that accelerates and intensifies as the red wall nears.

---

## 4. Controls & Hotkeys

* **`N`**: Toggle music on/off.
* **`[`**: Decrease master volume.
* **`]`**: Increase master volume.
* In the Main Menu, audio settings can also be toggled and adjusted with the mouse or arrow keys in the **Settings** screen.

---

## 5. Linux Dependencies

On Linux distributions, Glicol relies on ALSA development libraries to interface with the audio device:

```bash
sudo apt install libasound2-dev pkg-config
```

If no audio device is available, the audio engine reports status and runs silently without crashing.
