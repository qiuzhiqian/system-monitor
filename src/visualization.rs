use plotters::prelude::*;

const COLORS:[RGBColor;20] = [
    RGBColor(0x7f, 0xff, 0x00),
    RGBColor(0xd2, 0x69, 0x1e),
    RGBColor(0xff, 0x7f, 0x50),
    RGBColor(0x64, 0x95, 0xed),
    RGBColor(0xfa, 0xf0, 0xe6),

    RGBColor(0xff, 0x00, 0xff),
    RGBColor(0xd2, 0x69, 0xe1),
    RGBColor(0xd2, 0x69, 0xe1),
    RGBColor(0xd2, 0x69, 0xe1),
    RGBColor(0xd2, 0x69, 0xe1),

    RGBColor(0xd2, 0x69, 0xe1),
    RGBColor(0xd2, 0x69, 0xe1),
    RGBColor(0xd2, 0x69, 0xe1),
    RGBColor(0xd2, 0x69, 0xe1),
    RGBColor(0xd2, 0x69, 0xe1),

    RGBColor(0xd2, 0x69, 0xe1),
    RGBColor(0xd2, 0x69, 0xe1),
    RGBColor(0xd2, 0x69, 0xe1),
    RGBColor(0xd2, 0x69, 0xe1),
    RGBColor(0xd2, 0x69, 0xe1),
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
            let y = record[1].parse::<i32>().unwrap();
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
    (min_x, max_x, min_y, max_y, datas)
}

pub fn show_cpu() -> Result<(), Box<dyn std::error::Error>> {
    let mut rdr = csv::Reader::from_path("cpufreq.csv").unwrap();
    let (min_x, max_x, min_y, max_y, datas) = datas(&mut rdr);

    let root = SVGBackend::new("0.svg", (1920, 1080)).into_drawing_area();
    root.fill(&WHITE)?;
    let mut chart = ChartBuilder::on(&root)
        .caption("Line Chart with CSV", ("sans-serif", 40))
        .set_label_area_size(LabelAreaPosition::Left, 50)
        .set_label_area_size(LabelAreaPosition::Bottom, 50)
        .build_cartesian_2d(min_x..max_x, min_y..max_y)?;

    chart.configure_mesh().draw()?;

    let mut index = 0;
    for data in datas {
        chart.draw_series(LineSeries::new(data, COLORS[index % 20]))?;
        index = index + 1;
    }

    root.present()?;
    
    Ok(())
}