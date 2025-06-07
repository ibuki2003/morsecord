use anyhow::Context;
use std::env;
use std::io::Read;
const SAMPLE_RATE: u32 = 48000;

fn main() -> anyhow::Result<()> {
    let mut args = env::args().skip(1);
    let text = args.next().context("missing text")?;
    let out_path = args.next().context("missing output path")?;
    let wpm: f32 = args
        .next()
        .unwrap_or_else(|| "20".to_string())
        .parse()
        .context("invalid wpm")?;
    let freq: f32 = args
        .next()
        .unwrap_or_else(|| "600".to_string())
        .parse()
        .context("invalid freq")?;

    let mut pcm = morsecord::cw_audio::CWAudioPCM::new(text, wpm, freq, SAMPLE_RATE as usize);

    let spec = hound::WavSpec {
        channels: 1,
        sample_rate: SAMPLE_RATE,
        bits_per_sample: 16,
        sample_format: hound::SampleFormat::Int,
    };
    let mut writer = hound::WavWriter::create(out_path, spec).context("failed to create wav")?;

    let mut buf = vec![0u8; 4096];
    loop {
        let n = pcm.read(&mut buf)?;
        if n == 0 {
            break;
        }
        let samples = unsafe { std::slice::from_raw_parts(buf.as_ptr() as *const f32, n / 4) };
        for &sample in samples {
            let s = (sample * i16::MAX as f32) as i16;
            writer.write_sample(s)?;
        }
    }

    writer.finalize()?;
    Ok(())
}
