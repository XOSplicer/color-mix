use css_colors::{RGB, HSL, Angle, Ratio, Color};
use std::iter;

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
".record-{}.input-{} {{
    background-color: {};
}}\n",
            &self.id,
            n,
            c.to_css(),
        ))
        .collect();
        let rgb_avg = format!(
".record-{}.rgb-avg {{
    background-color: {};
}}\n",
            &self.id,
            &self.rgb_avg.to_css(),
        );
        let less_mix = format!(
".record-{}.less-mix {{
    background-color: {};
}}\n",
            &self.id,
            &self.less_mix.to_css(),
        );
        let hsl_geo = format!(
".record-{}.hsl-geo {{
    background-color: {};
}}\n",
            &self.id,
            &self.hsl_geo.to_css(),
        );
        vec![input, rgb_avg, less_mix, hsl_geo]
            .into_iter()
            .collect()
    }
}

fn rgb_avg(input: &[RGB]) -> Option<RGB> {
    if input.len() == 0 {
        return None;
    }
    let r_sum: u64 = input.iter().map(|c| c.r.as_u8() as u64).sum();
    let g_sum: u64 = input.iter().map(|c| c.g.as_u8() as u64).sum();
    let b_sum: u64 = input.iter().map(|c| c.b.as_u8() as u64).sum();
    let r_avg: u64 = r_sum / input.len() as u64;
    let g_avg: u64 = g_sum / input.len() as u64;
    let b_avg: u64 = b_sum / input.len() as u64;
    if r_avg > u8::max_value() as u64 {
        return None;
    }
    if g_avg > u8::max_value() as u64 {
        return None;
    }
    if b_avg > u8::max_value() as u64 {
        return None;
    }
    Some(RGB {
        r: Ratio::from_u8(r_avg as u8),
        g: Ratio::from_u8(g_avg as u8),
        b: Ratio::from_u8(b_avg as u8),
    })
}

fn less_mix(input: &[RGB]) -> Option<RGB> {
    if input.len() == 0 {
        return None;
    }
    let percent = Ratio::from_percentage((100.0 / input.len() as f32) as u8);
    Some(input.iter()
        .skip(1)
        .fold(input[0], |acc, c|
            acc.mix(c.clone(), percent.clone())
            .to_rgb()
        )
    )
}

fn hsl_geo(input: &[RGB]) -> Option<RGB> {
    if input.len() == 0 {
        return None;
    }
    let s_sum: u64 = input.iter().map(|c| c.clone().to_hsl().s.as_u8() as u64).sum();
    let l_sum: u64 = input.iter().map(|c| c.clone().to_hsl().l.as_u8() as u64).sum();
    let s_avg: u64 = s_sum / input.len() as u64;
    let l_avg: u64 = l_sum / input.len() as u64;
    if s_avg > 100 as u64 {
        return None;
    }
    if l_avg > 100 as u64 {
        return None;
    }
    let hue = Angle::new(0); // TODO: impl geometric
    //unimplemented!()
    Some(HSL {
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

fn main() {
    let max_len = 6;
    let rounds = 5;
    let css: String = (2..=max_len)
        .flat_map(|input_len| iter::repeat(input_len).zip(0..rounds))
        .map(|(input_len, round)| {
            let input: Vec<_> = (0..input_len).map(|_| random_color()).collect();
            let id = format!("{}-{}", input_len, round);
            let black = RGB {
                r: Ratio::from_u8(0),
                g: Ratio::from_u8(0),
                b: Ratio::from_u8(0),
            };
            let rgb_avg = rgb_avg(&input).unwrap_or_else(|| black.clone());
            let less_mix = less_mix(&input).unwrap_or_else(|| black.clone());
            let hsl_geo = hsl_geo(&input).unwrap_or_else(|| black.clone());
            let record = Record {
                id, input, rgb_avg, less_mix, hsl_geo
            };
            record.to_css()
        }).collect();
    println!("{}", css);
}
