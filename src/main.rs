use css_colors::{RGB, HSL, Angle, Ratio, Color};
use std::iter;
use std::fs::{self, File};
use std::path::Path;
use std::io::Write;
use std::panic;


#[derive(Debug)]
enum ComputeError {
    EmptyInput,
    AverageOutOfRange,
    AngleOutOfRange,
    PercentageOutOfRange,
    Panic,
}

#[derive(Debug)]
struct Record {
    id: String,
    input: Vec<RGB>,
    rgb_avg: RGB,
    less_mix: RGB,
    hsl_geo: RGB,
}

impl Record {
    fn to_css(&self) -> String {
        let input: String = self.input.iter()
        .enumerate()
        .map(|(n, c)| format!(
".record-{} .input-{} {{
    background-color: {};
}}\n",
            &self.id,
            n,
            c.to_css(),
        ))
        .collect();
        let rgb_avg = format!(
".record-{} .rgb-avg {{
    background-color: {};
}}\n",
            &self.id,
            &self.rgb_avg.to_css(),
        );
        let less_mix = format!(
".record-{} .less-mix {{
    background-color: {};
}}\n",
            &self.id,
            &self.less_mix.to_css(),
        );
        let hsl_geo = format!(
".record-{} .hsl-geo {{
    background-color: {};
}}\n",
            &self.id,
            &self.hsl_geo.to_css(),
        );
        vec![input, rgb_avg, less_mix, hsl_geo]
            .into_iter()
            .collect()
    }

    fn to_html(&self) -> String {
        let input: String = self.input.iter()
            .enumerate()
            .map(|(n, _)| format!("<div class='input input-{}'></div>\n", n))
            .collect();
        format!(
"<div class='record record-{}'>
    <div class='inputs'>
    {}
    </div>
    <div class='outputs'>
        <div class='output rgb-avg'></div>
        <div class='output less-mix'></div>
        <div class='output hsl-geo'></div>
    </div>
</div>\n",
            self.id,
            input
        )
    }
}

fn rgb_avg(input: &[RGB]) -> Result<RGB, ComputeError> {
    if input.len() == 0 {
        return Err(ComputeError::EmptyInput);
    }

    let r_sum: u64 = input.iter()
        .map(|c| c.r.as_u8() as u64)
        .sum();
    let g_sum: u64 = input.iter()
        .map(|c| c.g.as_u8() as u64)
        .sum();
    let b_sum: u64 = input.iter()
        .map(|c| c.b.as_u8() as u64)
        .sum();

    let r_avg: u64 = r_sum / input.len() as u64;
    let g_avg: u64 = g_sum / input.len() as u64;
    let b_avg: u64 = b_sum / input.len() as u64;

    if r_avg > u8::max_value() as u64 {
        return Err(ComputeError::AverageOutOfRange);
    }
    if g_avg > u8::max_value() as u64 {
        return Err(ComputeError::AverageOutOfRange);
    }
    if b_avg > u8::max_value() as u64 {
        return Err(ComputeError::AverageOutOfRange);
    }

    Ok(RGB {
        r: Ratio::from_u8(r_avg as u8),
        g: Ratio::from_u8(g_avg as u8),
        b: Ratio::from_u8(b_avg as u8),
    })
}

fn less_mix(input: &[RGB]) -> Result<RGB, ComputeError> {
    if input.len() == 0 {
        return Err(ComputeError::EmptyInput);
    }

    let percent = dbg!(1.0 / input.len() as f32);

    if percent < 0.0 || percent > 1.0 {
        return Err(ComputeError::PercentageOutOfRange);
    }

    let ratio = Ratio::from_f32(percent);

    Ok(input.iter()
        .skip(1)
        .fold(input[0], |acc, c|
            acc.mix(c.clone(), ratio.clone())
            .to_rgb()
        )
    )
}

fn hsl_geo(input: &[RGB]) -> Result<RGB, ComputeError> {
    if input.len() == 0 {
        return Err(ComputeError::EmptyInput);
    }

    let s_sum: u64 = input.iter()
        .map(|c| c.clone().to_hsl().s.as_u8() as u64)
        .sum();
    let l_sum: u64 = input.iter()
        .map(|c| c.clone().to_hsl().l.as_u8() as u64)
        .sum();

    let s_avg: u64 = dbg!(s_sum / input.len() as u64);
    let l_avg: u64 = dbg!(l_sum / input.len() as u64);

    if s_avg > u8::max_value() as u64 {
        return Err(ComputeError::AverageOutOfRange);
    }
    if l_avg > u8::max_value() as u64 {
        return Err(ComputeError::AverageOutOfRange);
    }

    let x_sum: f32 = input.iter()
        .map(|c| c.clone().to_hsl().h.degrees() as f32)
        .map(|degrees| degrees.to_radians().cos())
        .sum();
    let y_sum: f32 = input.iter()
        .map(|c| c.clone().to_hsl().h.degrees() as f32)
        .map(|degrees| degrees.to_radians().sin())
        .sum();

    let x_avg = dbg!(x_sum / input.len() as f32);
    let y_avg = dbg!(y_sum / input.len() as f32);

    let mut angle = dbg!(f32::atan2(y_avg, x_avg).to_degrees() as i16);

    while angle < 0 {
        angle += 360;
    }

    if angle > 360 || angle < 0 {
        return Err(ComputeError::AngleOutOfRange);
    }

    let hue = Angle::new(angle as u16);

    Ok(HSL {
        h: hue,
        s: Ratio::from_u8(s_avg as u8),
        l: Ratio::from_u8(l_avg as u8),
    }.to_rgb())
}

fn random_color() -> RGB {
    RGB {
        r: Ratio::from_u8(rand::random::<u8>()),
        g: Ratio::from_u8(rand::random::<u8>()),
        b: Ratio::from_u8(rand::random::<u8>()),
    }
}

fn create_iter(max_len: usize, rounds: usize) -> impl Iterator<Item=(usize, usize)> {
    (2..=max_len)
        .flat_map(move |input_len| iter::repeat(input_len).zip(0..rounds))
}

fn id(input_len: usize, round: usize) -> String {
    format!("{}-{}", input_len, round)
}

fn main() -> std::io::Result<()> {
    let max_len = 5;
    let rounds = 10;
    let out_dir = Path::new("./out");
    let res_dir = Path::new("./res");

    let records: Vec<Record> = create_iter(max_len, rounds)
        .map(|(input_len, round)| {
            let input: Vec<_> = (0..input_len).map(|_| random_color()).collect();
            let id = id(input_len, round);
            let black = RGB {
                r: Ratio::from_u8(0),
                g: Ratio::from_u8(0),
                b: Ratio::from_u8(0),
            };
            let rgb_avg = panic::catch_unwind(|| rgb_avg(&input))
                .map_err(|_| ComputeError::Panic)
                .and_then(|r| r)
                .unwrap_or_else(|e| {
                    eprintln!("WARN: {:?}: rgb_avg not computable for {:?}", e, &input);
                    black.clone()
                });
            let less_mix = panic::catch_unwind(|| less_mix(&input))
                .map_err(|_| ComputeError::Panic)
                .and_then(|r| r)
                .unwrap_or_else(|e| {
                    eprintln!("WARN: {:?}: less_mix not computable for {:?}", e, &input);
                    black.clone()
                });
            let hsl_geo = panic::catch_unwind(|| hsl_geo(&input))
                .map_err(|_| ComputeError::Panic)
                .and_then(|r| r)
                .unwrap_or_else(|e| {
                    eprintln!("WARN: {:?}: hsl_geo not computable for {:?}", e, &input);
                    black.clone()
                });
            Record {
                id, input, rgb_avg, less_mix, hsl_geo
            }
        }).collect();

        let color_css: String = records.iter()
            .map(|r| r.to_css())
            .collect();

        let html_content: String = records.iter()
            .map(|r| r.to_html())
            .collect();

        let html = format!(
"<html>
 <head>
<link rel='stylesheet' type='text/css' href='index.css'>
<link rel='stylesheet' type='text/css' href='colors.css'>
</head>
<body>
{}
</body>
</html>",
            html_content
        );

    fs::create_dir_all(out_dir)?;

    fs::copy(res_dir.join("index.css"), out_dir.join("index.css"))?;

    let mut color_css_file = File::create(out_dir.join("colors.css"))?;
    color_css_file.write_all(color_css.as_bytes())?;
    drop(color_css_file);

    let mut html_file = File::create(out_dir.join("index.html"))?;
    html_file.write_all(html.as_bytes())?;
    drop(html_file);



    Ok(())
}

