use core::f64::consts::PI;
use core::ops::{Add, Div, Mul, Sub};
use js_sys::Float64Array;
use wasm_bindgen::prelude::*;

#[wasm_bindgen]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum FilterType {
    LowPass,
    HighPass,
    BandPass,
}

#[derive(Clone, Copy, Debug, Default)]
struct C64 {
    re: f64,
    im: f64,
}

impl C64 {
    #[inline(always)]
    fn new(re: f64, im: f64) -> Self {
        Self { re, im }
    }

    #[inline(always)]
    fn abs(self) -> f64 {
        (self.re * self.re + self.im * self.im).sqrt()
    }

    #[inline(always)]
    fn scale(self, k: f64) -> Self {
        Self::new(self.re * k, self.im * k)
    }
}

impl Add for C64 {
    type Output = Self;
    #[inline(always)]
    fn add(self, rhs: Self) -> Self::Output {
        Self::new(self.re + rhs.re, self.im + rhs.im)
    }
}

impl Sub for C64 {
    type Output = Self;
    #[inline(always)]
    fn sub(self, rhs: Self) -> Self::Output {
        Self::new(self.re - rhs.re, self.im - rhs.im)
    }
}

impl Mul for C64 {
    type Output = Self;
    #[inline(always)]
    fn mul(self, rhs: Self) -> Self::Output {
        Self::new(
            self.re * rhs.re - self.im * rhs.im,
            self.re * rhs.im + self.im * rhs.re,
        )
    }
}

impl Div for C64 {
    type Output = Self;
    #[inline(always)]
    fn div(self, rhs: Self) -> Self::Output {
        let den = rhs.re * rhs.re + rhs.im * rhs.im;
        Self::new(
            (self.re * rhs.re + self.im * rhs.im) / den,
            (self.im * rhs.re - self.re * rhs.im) / den,
        )
    }
}

#[derive(Clone, Debug)]
struct Biquad {
    b0: f64,
    b1: f64,
    b2: f64,
    a1: f64,
    a2: f64,
    z1: f64,
    z2: f64,
}

impl Biquad {
    #[inline(always)]
    fn new(b0: f64, b1: f64, b2: f64, a1: f64, a2: f64) -> Self {
        Self {
            b0,
            b1,
            b2,
            a1,
            a2,
            z1: 0.0,
            z2: 0.0,
        }
    }

    #[inline(always)]
    fn process_sample(&mut self, x: f64) -> f64 {
        // Direct Form II Transposed
        let y = self.b0 * x + self.z1;
        self.z1 = self.b1 * x - self.a1 * y + self.z2;
        self.z2 = self.b2 * x - self.a2 * y;
        y
    }

    #[inline(always)]
    fn reset_state(&mut self) {
        self.z1 = 0.0;
        self.z2 = 0.0;
    }

    #[inline(always)]
    fn response(&self, omega: f64) -> C64 {
        let x = C64::new(omega.cos(), -omega.sin());
        let x2 = x * x;

        let num = C64::new(self.b0, 0.0) + x.scale(self.b1) + x2.scale(self.b2);
        let den = C64::new(1.0, 0.0) + x.scale(self.a1) + x2.scale(self.a2);

        num / den
    }
}

#[wasm_bindgen]
pub struct FilterEngine {
    sample_rate: f64,
    cutoff: f64,
    order: u32,
    filter_type: FilterType,
    sections: Vec<Biquad>,
}

#[wasm_bindgen]
impl FilterEngine {
    #[wasm_bindgen(constructor)]
    pub fn new() -> Self {
        Self {
            sample_rate: 48_000.0,
            cutoff: 1_000.0,
            order: 4,
            filter_type: FilterType::LowPass,
            sections: Vec::new(),
        }
    }

    pub fn compute_coefficients(
        &mut self,
        sample_rate: f64,
        cutoff: f64,
        order: u32,
        filter_type: FilterType,
    ) {
        assert!(sample_rate.is_finite() && sample_rate > 0.0, "sample_rate must be > 0");
        assert!(cutoff.is_finite() && cutoff > 0.0, "cutoff must be > 0");
        assert!(order >= 1, "order must be >= 1");

        let nyquist = sample_rate * 0.5;
        assert!(cutoff < nyquist, "cutoff must be below Nyquist");

        self.sample_rate = sample_rate;
        self.cutoff = cutoff;
        self.order = order;
        self.filter_type = filter_type;
        self.sections.clear();

        match filter_type {
            FilterType::LowPass => {
                self.sections = build_butterworth_lowpass(sample_rate, cutoff, order);
                normalize_cascade(&mut self.sections, 0.0);
            }
            FilterType::HighPass => {
                self.sections = build_butterworth_highpass(sample_rate, cutoff, order);
                normalize_cascade(&mut self.sections, PI);
            }
            FilterType::BandPass => {
                assert!(order >= 2, "BandPass needs order >= 2");
                self.sections = build_butterworth_bandpass(sample_rate, cutoff, order);
                normalize_cascade(&mut self.sections, 2.0 * PI * cutoff / sample_rate);
            }
        }

        for s in &mut self.sections {
            s.reset_state();
        }
    }

    pub fn process_batch(&mut self, data: Float64Array) -> Float64Array {
        let mut buf = data.to_vec();
        self.process_in_place(&mut buf);

        let out = Float64Array::new_with_length(buf.len() as u32);
        out.copy_from(&buf);
        out
    }

    pub fn reset_state(&mut self) {
        for s in &mut self.sections {
            s.reset_state();
        }
    }

    pub fn sample_rate(&self) -> f64 {
        self.sample_rate
    }

    pub fn cutoff(&self) -> f64 {
        self.cutoff
    }

    pub fn order(&self) -> u32 {
        self.order
    }

    pub fn filter_type(&self) -> FilterType {
        self.filter_type
    }
}

impl FilterEngine {
    #[inline(always)]
    fn process_in_place(&mut self, data: &mut [f64]) {
        if self.sections.is_empty() {
            return;
        }

        for x in data.iter_mut() {
            let mut v = *x;
            for sec in &mut self.sections {
                v = sec.process_sample(v);
            }
            *x = v;
        }
    }
}

fn normalize_cascade(sections: &mut [Biquad], omega: f64) {
    if sections.is_empty() {
        return;
    }

    let mut h = C64::new(1.0, 0.0);
    for s in sections.iter() {
        h = h * s.response(omega);
    }

    let mag = h.abs();
    if mag > 0.0 && mag.is_finite() {
        let g = 1.0 / mag;
        let first = &mut sections[0];
        first.b0 *= g;
        first.b1 *= g;
        first.b2 *= g;
    }
}

#[inline(always)]
fn bilinear_biquad(
    b0: f64,
    b1: f64,
    b2: f64,
    a0: f64,
    a1: f64,
    a2: f64,
    sample_rate: f64,
) -> Biquad {
    let k = 2.0 * sample_rate;
    let kk = k * k;

    let a0d = a2 * kk + a1 * k + a0;
    let a1d = 2.0 * a0 - 2.0 * a2 * kk;
    let a2d = a2 * kk - a1 * k + a0;

    let b0d = b2 * kk + b1 * k + b0;
    let b1d = 2.0 * b0 - 2.0 * b2 * kk;
    let b2d = b2 * kk - b1 * k + b0;

    Biquad::new(b0d / a0d, b1d / a0d, b2d / a0d, a1d / a0d, a2d / a0d)
}

#[inline(always)]
fn bilinear_first_order_lowpass(wc: f64, sample_rate: f64) -> Biquad {
    let k = 2.0 * sample_rate;
    let d = k + wc;
    Biquad::new(wc / d, wc / d, 0.0, (wc - k) / d, 0.0)
}

#[inline(always)]
fn bilinear_first_order_highpass(wc: f64, sample_rate: f64) -> Biquad {
    let k = 2.0 * sample_rate;
    let d = k + wc;
    Biquad::new(k / d, -k / d, 0.0, (wc - k) / d, 0.0)
}

fn build_butterworth_lowpass(sample_rate: f64, cutoff: f64, order: u32) -> Vec<Biquad> {
    let wc = 2.0 * sample_rate * (PI * cutoff / sample_rate).tan();
    let mut sections = Vec::with_capacity(((order as usize) + 1) / 2);

    let pairs = order / 2;
    for k in 0..pairs {
        let theta = PI * (2.0 * k as f64 + order as f64 - 1.0) / (2.0 * order as f64);
        let re = theta.cos();

        let sec = bilinear_biquad(
            wc * wc,
            0.0,
            0.0,
            wc * wc,
            -2.0 * re * wc,
            1.0,
            sample_rate,
        );
        sections.push(sec);
    }

    if order % 2 == 1 {
        sections.push(bilinear_first_order_lowpass(wc, sample_rate));
    }

    sections
}

fn build_butterworth_highpass(sample_rate: f64, cutoff: f64, order: u32) -> Vec<Biquad> {
    let wc = 2.0 * sample_rate * (PI * cutoff / sample_rate).tan();
    let mut sections = Vec::with_capacity(((order as usize) + 1) / 2);

    let pairs = order / 2;
    for k in 0..pairs {
        let theta = PI * (2.0 * k as f64 + order as f64 - 1.0) / (2.0 * order as f64);
        let re = theta.cos();

        let sec = bilinear_biquad(
            0.0,
            0.0,
            1.0,
            wc * wc,
            -2.0 * re * wc,
            1.0,
            sample_rate,
        );
        sections.push(sec);
    }

    if order % 2 == 1 {
        sections.push(bilinear_first_order_highpass(wc, sample_rate));
    }

    sections
}

fn build_butterworth_bandpass(sample_rate: f64, center: f64, order: u32) -> Vec<Biquad> {
    // Default one-octave window around center.
    let low = (center / 2.0_f64.sqrt()).max(1.0e-9);
    let high = (center * 2.0_f64.sqrt()).min(sample_rate * 0.499_999);

    assert!(low < high, "band-pass edges collapsed");

    let hp_order = order / 2;
    let lp_order = order - hp_order;

    let mut sections = Vec::with_capacity(((order as usize) + 1) / 2 + 1);
    sections.extend(build_butterworth_highpass(sample_rate, low, hp_order.max(1)));
    sections.extend(build_butterworth_lowpass(sample_rate, high, lp_order.max(1)));

    sections
}