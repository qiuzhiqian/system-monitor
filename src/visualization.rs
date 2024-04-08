use plotters::prelude::*;

const COLORS:[RGBColor;20] = [
    RGBColor(0xd2, 0x69, 0x1e),
    RGBColor(0xff, 0x7f, 0x50),
    RGBColor(0x64, 0x95, 0xed),
    RGBColor(0xfa, 0xf0, 0xe6),
    RGBColor(0xff, 0x00, 0xff),

    RGBColor(0xfe, 0xee, 0xed),
    RGBColor(0xf0, 0x5b, 0x72),
    RGBColor(0xca, 0x86, 0x87),
    RGBColor(0xed, 0x19, 0x41),
    RGBColor(0x90, 0x5a, 0x3d),

    RGBColor(0xfa, 0xa7, 0x55),
    RGBColor(0xe0, 0x86, 0x1a),
    RGBColor(0xb7, 0xba, 0x6b),
    RGBColor(0x7f, 0xb8, 0x0e),
    RGBColor(0x19, 0xd5, 0x3f),

    RGBColor(0x50, 0xb7, 0xc1),
    RGBColor(0x44, 0x46, 0x93),
    RGBColor(0x85, 0x52, 0xa1),
    RGBColor(0xea, 0x66, 0xa6),
    RGBColor(0x7f, 0xff, 0x00),
];

fn datas(rdr: &mut csv::Reader<std::fs::File>) -> (i32,i32,i32,i32,Vec<Vec<(i32, i32)>>) {
    let mut min_x  = -1;
    let mut min_y = -1;
    let mut max_x = 0;
    let mut max_y = 0;
    let mut datas = Vec::new();
    for result in rdr.records() {
        let record = result.unwrap();
        if record.position().unwrap().line() == 1 {
            continue;
        }

        if record.len() -1 > datas.len() {
            datas.resize(record.len() -1, Vec::new());
        }
        let x = record[0].parse::<i32>().unwrap();
        for i in 1..record.len() {
            let y = record[i].parse::<i32>().unwrap();
            datas[i-1].push((record[0].parse::<i32>().unwrap(), record[i].parse::<i32>().unwrap()));

            if min_x < 0 {
                min_x = x;
            }
            if min_y < 0 {
                min_y = y;
            }
            if x > max_x {
                max_x =x;
            } else if x < min_x {
                min_x = x;
            }
    
            if y > max_y {
                max_y = y;
            } else if y < min_y {
                min_y = y;
            }
        }
    }
    
    let mut v = (max_y - min_y) / 5;
    if v <= 0 {
        v = 10;
    }
    min_y = min_y - v;
    max_y = max_y + v;
    (min_x, max_x, min_y, max_y, datas)
}

// cpufreq.csv -> cpufreq.svg
// capacity.csv -> capacity.svg
pub fn show_datas(infile: &str, outfile: &str, desc: &str) -> Result<(), Box<dyn std::error::Error>> {
    let mut rdr = csv::Reader::from_path(infile)?;
    let (min_x, max_x, min_y, max_y, datas) = datas(&mut rdr);

    let root = SVGBackend::new(outfile, (1920, 1080)).into_drawing_area();
    root.fill(&WHITE)?;
    let mut chart = ChartBuilder::on(&root)
        .caption(desc, ("sans-serif", 40))
        .set_label_area_size(LabelAreaPosition::Left, 50)
        .set_label_area_size(LabelAreaPosition::Bottom, 50)
        .build_cartesian_2d(min_x..max_x, min_y..max_y)?;

    chart.configure_mesh().draw()?;

    let mut index = 0;
    for data in datas {
        chart.draw_series(LineSeries::new(data, COLORS[index % 20]).point_size(5))?;
        index = index + 1;
    }

    root.present()?;
    
    Ok(())
}