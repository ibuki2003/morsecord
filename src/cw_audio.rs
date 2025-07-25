use songbird::input::{reader::MediaSource, Input};

pub struct CWAudioPCM {
    epos: usize,                // current position in the events
    spos: usize,                // current position in a event
    events: Vec<(usize, bool)>, // (length in samples, ON/OFF)

    omega: f32,   // angular frequency
    srate: usize, // sample rate
}

const ENV_MS: f32 = 10.0; // envelope length in ms

impl CWAudioPCM {
    pub fn new(str: String, wpm: f32, freq: f32, srate: usize) -> Self {
        let dot_length = (crate::morse::dot_time(wpm).as_secs_f32()
            * songbird::constants::SAMPLE_RATE_RAW as f32) as usize;

        let mut events = Vec::new();

        events.push((dot_length * 2, false)); // first pause

        for c in crate::morse::get_morse_str(str) {
            if c.0 == 0 {
                // space
                events.push((dot_length * 4, false));
            } else {
                let (l, b) = c;
                for i in (0..l).rev() {
                    let k = (b & (1 << i)) != 0;
                    events.push((dot_length * (if k { 3 } else { 1 }), true));
                    events.push((dot_length, false));
                }
                events.push((dot_length * 2, false));
            }
        }

        Self {
            epos: 0,
            spos: 0,
            events,

            omega: 2.0 * std::f32::consts::PI * freq / srate as f32,
            srate,
        }
    }

    pub fn get_duration(s: &str, wpm: f32) -> std::time::Duration {
        let mut length = 2;

        for c in s.chars() {
            if c == ' ' {
                length += 4;
                continue;
            }

            let (n, b) = crate::morse::get_morse(c);
            for i in 0..n {
                if b & (1 << i) != 0 {
                    length += 4;
                } else {
                    length += 2;
                }
            }
            length += 2;
        }

        crate::morse::dot_time(wpm) * length
    }

    pub fn to_input(self) -> Input {
        Input::float_pcm(
            false,
            songbird::input::reader::Reader::Extension(std::boxed::Box::new(self)),
        )
    }
}

impl MediaSource for CWAudioPCM {
    fn is_seekable(&self) -> bool {
        false
    }
    fn byte_len(&self) -> Option<u64> {
        None
    }
}

impl std::io::Read for CWAudioPCM {
    fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
        let head = buf.as_ptr();
        let (_, mut s, _) = unsafe { buf.align_to_mut::<f32>() };

        // envelope length in samples
        let env_len: usize = ((ENV_MS / 1000.0) * self.srate as f32) as usize;

        while self.epos + 1 < self.events.len() && !s.is_empty() {
            let (length, on) = self.events[self.epos];
            let t = length - self.spos;

            let c = std::cmp::min(s.len(), t);
            let of = s.len() <= c;

            if on {
                let spos = self.spos;
                s[..c].iter_mut().enumerate().for_each(|(i, x)| {
                    let pos = spos + i;
                    // Envelope gain (0.0~1.0), using cosine shape
                    let gain = if pos < env_len {
                        // Fade in (cosine)
                        let theta = (pos as f32 / env_len as f32) * std::f32::consts::PI;
                        0.5 * (1.0 - (theta).cos())
                    } else if pos + env_len > length {
                        // Fade out (cosine)
                        let rem = length.saturating_sub(pos);
                        let theta = (rem as f32 / env_len as f32) * std::f32::consts::PI;
                        0.5 * (1.0 - (theta).cos())
                    } else {
                        1.0
                    };
                    *x = gain * (self.omega * (self.spos + i) as f32).sin();
                });
            } else {
                s[..c].iter_mut().for_each(|x| *x = 0.);
            }
            s = &mut s[c..];
            self.spos += c;
            if !of {
                self.epos += 1;
                self.spos = 0;
            }

            if self.spos < t {
                break;
            }
        }
        Ok(s.as_ptr() as usize - head as usize)
    }
}

impl std::io::Seek for CWAudioPCM {
    fn seek(&mut self, _pos: std::io::SeekFrom) -> std::io::Result<u64> {
        unreachable!();
    }
}
