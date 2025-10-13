use eframe::egui;
use egui_plot::{HLine, Legend, Line, Plot, PlotPoint, PlotPoints, Text as PlotText, VLine};
use egui::{RichText, Color32};
use hound::{SampleFormat, WavReader};
use rodio::{buffer::SamplesBuffer, OutputStream, OutputStreamHandle, Sink};
use rustfft::{num_complex::Complex, FftPlanner};
use std::{env, error::Error, f32::consts::PI, path::Path, time::Instant};
use std::cell::Cell;
use std::fs::File;
use std::io::Write;
use eframe::egui::Align;
use eframe::emath::{Align2, Vec2b};

fn main() -> Result<(), Box<dyn Error>> {
    let win_size: usize = 2048;
    let hop_size: usize = 512;

    let (mono, sample_rate) = read_wav_mono_f32()?;
    let sr_in = sample_rate as f32;
    let duration = mono.len() as f32 / sr_in;

    let (track, sampled_track, global_peak) = dominant_frequency_track(&mono, sr_in, win_size, hop_size)?;
    let fmax = sr_in / 2.0;

    // 准备 App 状态
    let file_name = "input.wav".to_string();

    let sr_out = 44_100u32;

    let dumped_data : Vec<u8> = track
        .iter()
        .flat_map(|(t, f)| (*t as f64).to_le_bytes().into_iter().chain((*f as f64).to_le_bytes()))
        .collect();
    {
        let mut file = File::create("f:\\track.bin").unwrap();
        file.write(&dumped_data).unwrap();
    }

    let app = App::new(
        duration as f64,
        fmax as f64,
        track
            .iter()
            .map(|(t, f)| [*t as f64, *f as f64])
            .collect(),
        sampled_track
            .iter()
            .map(|(t, freqs)| (*t as f64, [freqs[0] as f64, freqs[1] as f64, freqs[2] as f64]))
            .collect(),
        equal_temperament_marks(20.0, fmax as f32)
            .into_iter()
            .map(|(f, name, midi)| (f as f64, name, midi))
            .collect(),
        global_peak.map(|(t, f, m)| (t as f64, f as f64, m)),
        sr_out,
        duration as f64,
    );

    let native_options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([1280.0, 800.0])
            .with_title("主频轨迹浏览器（支持滚轮缩放/平移）"),
        ..Default::default()
    };

    eframe::run_native(
        "主频轨迹浏览器",
        native_options,
        Box::new(|cc| {
            Ok(Box::new(app))
        }),
    )?;

    Ok(())
}

// ========================== 数据处理 ==========================

fn read_wav_mono_f32() -> Result<(Vec<f32>, u32), Box<dyn Error>> {
    let mut reader = WavReader::new(std::io::Cursor::new(include_bytes!("f:\\output.wav")))?;
    let spec = reader.spec();
    let sr = spec.sample_rate;
    let ch = spec.channels as usize;

    let mono: Vec<f32> = match (spec.sample_format, spec.bits_per_sample) {
        (SampleFormat::Int, 16) => {
            let samples: Result<Vec<i16>, _> = reader.samples::<i16>().collect();
            mixdown_to_mono_i16(&samples?, ch)
        }
        (SampleFormat::Float, 32) => {
            let samples: Result<Vec<f32>, _> = reader.samples::<f32>().collect();
            mixdown_to_mono_f32(&samples?, ch)
        }
        _ => return Err("仅支持 16-bit PCM 或 32-bit float 的 WAV".into()),
    };

    Ok((mono, sr))
}

fn mixdown_to_mono_i16(samples: &[i16], channels: usize) -> Vec<f32> {
    if channels == 1 {
        samples.iter().map(|&s| s as f32 / 32768.0).collect()
    } else {
        let mut mono = Vec::with_capacity(samples.len() / channels);
        for frame in samples.chunks_exact(channels) {
            let mut acc = 0.0f32;
            for &s in frame {
                acc += s as f32 / 32768.0;
            }
            mono.push(acc / channels as f32);
        }
        mono
    }
}

fn mixdown_to_mono_f32(samples: &[f32], channels: usize) -> Vec<f32> {
    if channels == 1 {
        samples.to_vec()
    } else {
        let mut mono = Vec::with_capacity(samples.len() / channels);
        for frame in samples.chunks_exact(channels) {
            let mut acc = 0.0f32;
            for &s in frame {
                acc += s;
            }
            mono.push(acc / channels as f32);
        }
        mono
    }
}

fn dominant_frequency_track(
    mono: &[f32],
    sr: f32,
    win_size: usize,
    hop_size: usize,
) -> Result<(Vec<(f32, f32)>, Vec<(f32, [f32; 3])>, Option<(f32, f32, f32)>), Box<dyn Error>> {
    let mut planner = FftPlanner::<f32>::new();
    let fft = planner.plan_fft_forward(win_size);
    let hann: Vec<f32> = (0..win_size)
        .map(|n| 0.5 * (1.0 - (2.0 * PI * n as f32 / (win_size as f32 - 1.0)).cos()))
        .collect();

    let mut track = Vec::<(f32, f32)>::new();
    let mut sampled_track = Vec::<(f32, [f32; 3])>::new();
    let mut global_peak: Option<(f32, f32, f32)> = None;

    let nyquist = sr / 2.0;
    let mut start = 0usize;

    while start + win_size <= mono.len() {
        let mut buf: Vec<Complex<f32>> = (0..win_size)
            .map(|i| Complex {
                re: mono[start + i] * hann[i],
                im: 0.0,
            })
            .collect();

        fft.process(&mut buf);

        let half = win_size / 2;

        let mut peaks: Vec<(usize, f32)> = (0..half)
            .map(|k| {
                let c = buf[k];
                let mag2 = c.re * c.re + c.im * c.im;
                (k, mag2)
            })
            .collect();

        peaks.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));

        let top3_freqs: [f32; 3] = [
            (peaks[0].0 as f32 * sr / win_size as f32).clamp(0.0, nyquist),
            if peaks.len() > 1 {
                (peaks[1].0 as f32 * sr / win_size as f32).clamp(0.0, nyquist)
            } else {
                0.0
            },
            if peaks.len() > 2 {
                (peaks[2].0 as f32 * sr / win_size as f32).clamp(0.0, nyquist)
            } else {
                0.0
            },
        ];

        let t = start as f32 / sr;
        let f_max = top3_freqs[0];
        let max_mag2 = peaks[0].1;

        track.push((t, f_max));
        sampled_track.push((t, top3_freqs));

        if max_mag2 > global_peak.map(|(_, _, m)| m).unwrap_or(-1.0) {
            global_peak = Some((t, f_max, max_mag2));
        }

        start += hop_size;
    }

    Ok((track, sampled_track, global_peak))
}

fn equal_temperament_marks(fmin: f32, fmax: f32) -> Vec<(f32, String, i32)> {
    let names = ["C","C#","D","D#","E","F","F#","G","G#","A","A#","B"];
    let mut v = Vec::new();
    for midi in 0..=127 {
        let f = 440.0 * 2f32.powf((midi as f32 - 69.0) / 12.0);
        if f >= fmin && f <= fmax {
            let pc = (midi % 12) as usize;
            let octave = (midi / 12) - 1;
            let name = format!("{}{}", names[pc], octave);
            v.push((f, name, midi));
        }
    }
    v
}
// ========================== GUI 应用 ==========================

#[derive(Debug, Clone, Copy, PartialEq)]
enum PlaybackTrack {
    Max,
    Sample1,
    Sample2,
    Sample3,
    BeatNotes,  // 新增：节拍音符
}

impl PlaybackTrack {
    fn label(&self) -> &str {
        match self {
            PlaybackTrack::Max => "最大值（红线）",
            PlaybackTrack::Sample1 => "采样频率 #1",
            PlaybackTrack::Sample2 => "采样频率 #2",
            PlaybackTrack::Sample3 => "采样频率 #3",
            PlaybackTrack::BeatNotes => "节拍音符",  // 新增
        }
    }
}

struct App {
    duration: f64,
    fmax: f64,
    track: Vec<[f64; 2]>,
    sampled_track: Vec<(f64, [f64; 3])>,
    global_peak: Option<(f64, f64, f32)>,
    note_marks: Vec<(f64, String, i32)>,

    show_note_lines: bool,
    show_sampled_freqs: bool,
    dense_threshold: usize,
    time_bounds: (Cell<f64>, Cell<f64>),
    freq_bounds: (Cell<f64>, Cell<f64>),

    bpm: f64,
    show_beat_lines: bool,
    beats_per_bar: usize,
    show_beat_notes: bool,  // 新增：是否显示节拍音符标注

    selected_track: PlaybackTrack,
    sr_out: u32,
    stream: Option<OutputStream>,
    handle: Option<OutputStreamHandle>,
    sink: Option<Sink>,
    playing: bool,
    play_start_time: Option<Instant>,
    play_position: f64,
}

impl App {
    fn new(
        duration: f64,
        fmax: f64,
        track: Vec<[f64; 2]>,
        sampled_track: Vec<(f64, [f64; 3])>,
        note_marks: Vec<(f64, String, i32)>,
        global_peak: Option<(f64, f64, f32)>,
        sr_out: u32,
        _total_duration: f64,
    ) -> Self {
        Self {
            duration,
            fmax,
            time_bounds: (Cell::new(0.0), Cell::new(duration.max(1e-6))),
            freq_bounds: (Cell::new(0.0), Cell::new(fmax.max(1.0))),
            track,
            sampled_track,
            global_peak,
            note_marks,
            show_note_lines: true,
            show_sampled_freqs: true,
            dense_threshold: 36,
            bpm: 120.0,
            show_beat_lines: true,
            beats_per_bar: 4,
            show_beat_notes: true,  // 默认显示节拍音符
            selected_track: PlaybackTrack::Max,
            sr_out,
            stream: None,
            handle: None,
            sink: None,
            playing: false,
            play_start_time: None,
            play_position: 0.0,
        }
    }

    // 分析每个节拍的主导音符
    fn analyze_beat_notes(&self) -> Vec<(f64, String, f64, bool)> {
        if self.bpm <= 0.0 {
            return Vec::new();
        }

        let beat_duration = 60.0 / self.bpm;
        let mut beat_notes = Vec::new();

        let num_beats = (self.duration / beat_duration).ceil() as usize;

        for beat_idx in 0..num_beats {
            let beat_start = beat_idx as f64 * beat_duration;
            let beat_end = beat_start + beat_duration;

            if beat_start > self.duration {
                break;
            }

            // 收集这个节拍内的所有频率数据
            let mut freqs_in_beat = Vec::new();
            for (t, freqs) in &self.sampled_track {
                if *t >= beat_start && *t < beat_end && freqs[0] > 20.0 {
                    freqs_in_beat.push(freqs[0]);
                }
            }

            // 如果节拍内没有足够数据，尝试从 track 中获取
            if freqs_in_beat.is_empty() {
                for point in &self.track {
                    if point[0] >= beat_start && point[0] < beat_end && point[1] > 20.0 {
                        freqs_in_beat.push(point[1]);
                    }
                }
            }

            if !freqs_in_beat.is_empty() {
                // 计算中位数频率（比平均值更稳定）
                freqs_in_beat.sort_by(|a, b| a.partial_cmp(b).unwrap());
                let median_freq = freqs_in_beat[freqs_in_beat.len() / 2];

                let (note_name, note_freq) = nearest_note(median_freq);
                let is_bar_start = beat_idx % self.beats_per_bar == 0;

                beat_notes.push((beat_start, note_name, note_freq, is_bar_start));
            }
        }

        beat_notes
    }


    fn draw_plot(&self, ui: &mut egui::Ui) {
        let plot = Plot::new("dominant_freq_plot")
            .legend(Legend::default())
            .allow_scroll(false)
            .allow_zoom(true)
            .allow_boxed_zoom(true)
            .allow_drag(true)
            .default_x_bounds(self.time_bounds.0.get(), self.time_bounds.1.get())
            .default_y_bounds(self.freq_bounds.0.get(), self.freq_bounds.1.get())
            .auto_bounds(Vec2b::new(true, true))
            // .include_x(self.time_bounds.0)
            // .include_x(self.time_bounds.1)
            // .include_y(self.freq_bounds.0)
            // .include_y(self.freq_bounds.1)
            .label_formatter(|name, value| {
                if !name.is_empty() {
                    format!("{name}\n时间: {:.3}s\n频率: {:.1}Hz", value.x, value.y)
                } else {
                    format!("时间: {:.3}s\n频率: {:.1}Hz", value.x, value.y)
                }
            });

        plot.show(ui, |plot_ui| {
            // 节拍线和音符标注
            if self.show_beat_lines && self.bpm > 0.0 {
                let beat_duration = 60.0 / self.bpm;
                let bounds = plot_ui.plot_bounds();
                let y_span = self.freq_bounds.1.get() - self.freq_bounds.0.get();

                let start_beat = (bounds.min()[0] / beat_duration).floor() as i32;
                let end_beat = (bounds.max()[0] / beat_duration).ceil() as i32;

                for beat_num in start_beat..=end_beat {
                    if beat_num < 0 {
                        continue;
                    }

                    let beat_time = beat_num as f64 * beat_duration;

                    if beat_time > self.duration {
                        break;
                    }

                    let is_bar_start = beat_num as usize % self.beats_per_bar == 0;

                    // 绘制节拍线
                    let (color, width) = if is_bar_start {
                        (Color32::from_rgb(0, 100, 200), 2.0)
                    } else {
                        (Color32::from_rgba_unmultiplied(100, 150, 255, 150), 1.0)
                    };

                    let beat_line = VLine::new("beat_time", beat_time)
                        .color(color)
                        .width(width);
                    plot_ui.vline(beat_line);

                    // 绘制节拍音符标注框
                    if self.show_beat_notes {
                        // if let Some((_, note_name, note_freq, is_strong)) = beat_notes.iter().take(1)
                        //     .find(|(t, _, _, _)| (*t - beat_time).abs() < beat_duration * 0.1) {

                            // 矩形位置：在图表顶部
                            let rect_y_center = bounds.max()[1] - 0.05 * y_span;
                            let rect_height = 0.08 * y_span;
                            let rect_width = beat_duration * 0.8;

                            // 矩形的四个角
                            let rect_x_min = beat_time;
                            let rect_x_max = beat_time + rect_width;
                            let rect_y_min = rect_y_center - rect_height / 2.0;
                            let rect_y_max = rect_y_center + rect_height / 2.0;

                            let border_width = 1f32;

                            let rect_x_min = rect_x_min.min(self.time_bounds.1.get());
                            let rect_x_max = rect_x_max.max(self.time_bounds.0.get());
                            let rect_y_max = rect_y_max.max(self.freq_bounds.0.get()) - border_width as f64;


                            // 上边框
                            plot_ui.line(Line::new("PlotPoints", PlotPoints::from_iter(vec![
                                [rect_x_min, rect_y_max],
                                [rect_x_max, rect_y_max],
                            ])).width(border_width));


                        // }
                    }

                }
            }

        });
    }
}

fn nearest_note(freq: f64) -> (String, f64) {
    if freq <= 0.0 {
        return ("N/A".into(), 0.0);
    }
    let midi = (69.0 + 12.0 * (freq / 440.0).log2()).round();
    let midi_i = midi.clamp(0.0, 127.0) as i32;
    let names = ["C","C#","D","D#","E","F","F#","G","G#","A","A#","B"];
    let pc = (midi_i % 12) as usize;
    let octave = (midi_i / 12) - 1;
    let name = format!("{}{}", names[pc], octave);
    let f = 440.0 * 2f64.powf((midi_i as f64 - 69.0) / 12.0);
    (name, f)
}

impl eframe::App for App {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {

        egui::CentralPanel::default().show(ctx, |ui| {
            self.draw_plot(ui);
        });
    }
}