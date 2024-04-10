//use plotters::prelude::*;
use charts_rs::{Box, LineChart, Series};
use std::io::Write;

// cpufreq.csv -> cpufreq.svg
// capacity.csv -> capacity.svg
pub fn show_datas(infile: &str, outfile: &str, desc: &str) -> std::io::Result<()> {
    let mut rdr = csv::Reader::from_path(infile)?;
    let mut series_list = Vec::new();
    let mut x_axis_data = Vec::new();
    let mut record_index = 0;
    for result in rdr.records() {
        let record = result.unwrap();
        if record_index == 0 {
            println!("find head.");
            let mut index = 0;
            for r in record.into_iter() {
                if index > 0 {
                    let tag = r.to_string();
                    series_list.push(Series::new(tag, Vec::new()));
                    println!("insert head.");
                }
                index = index + 1;
            }
        }

        for i in 0..record.len() {
            if i == 0 {
                x_axis_data.push( record[i].to_string());
            } else {
                series_list[i - 1].data.push(record[i].parse::<i32>().unwrap() as f32);
                series_list[i - 1].label_show = true
            }
        }

        record_index = record_index + 1;
    }
    let mut line_chart = LineChart::new(series_list, x_axis_data);
    line_chart.title_text = "Stacked Area Chart".to_string();
    line_chart.sub_title_text = "Hello World".to_string();
    line_chart.legend_margin = Some(Box {
        top: 50.0,
        bottom: 10.0,
        ..Default::default()
    });
    line_chart.width = 1920.0;
    line_chart.height = 1080.0;

    if let Ok(svg_data) = line_chart.svg() {
        let mut buffer = std::fs::File::create(outfile)?;
        buffer.write_all(svg_data.as_bytes())?;
    }
    
    Ok(())
}