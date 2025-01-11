use std::collections::BTreeMap;

use chrono::NaiveDate;
use gnuplot::{
    AutoOption, AxesCommon,
    DashType::Dash,
    Figure, LabelOption,
    PlotOption::{Caption, Color, LineStyle},
    Tick,
};

use super::monitor::{ExchangeRate, ExchangeRateConfig};

pub fn generate_plot(
    prices: &[ExchangeRate],
    file_path: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    let path = std::path::Path::new(file_path);
    let dir = path.parent().unwrap();
    std::fs::create_dir_all(dir).unwrap();

    let config = &ExchangeRateConfig::new();

    let mut date_map: BTreeMap<NaiveDate, f64> = BTreeMap::new();
    for rate in prices {
        date_map.insert(rate.date.date_naive(), rate.rate);
    }

    let first_date = *date_map.keys().next().unwrap();
    let last_date = *date_map.keys().last().unwrap();

    for date in first_date.iter_days().take_while(|&d| d <= last_date) {
        if !date_map.contains_key(&date) {
            date_map.insert(date, f64::NAN);
        }
    }

    let mut data_points: Vec<(String, f64)> = vec![];

    println!("date_map {:?}", date_map);
    for (date, rate) in &date_map {
        data_points.push((date.format("%d-%m-%Y").to_string(), *rate));
    }

    println!("data_points: {:?}", data_points);

    let mut fg = Figure::new();

    let x_values: Vec<usize> = data_points.iter().enumerate().map(|(i, _)| i).collect();
    let y_values: Vec<f64> = data_points.iter().map(|(_, p)| *p).collect();

    let tick_labels: Vec<String> = data_points.iter().map(|(date, _)| date.clone()).collect();

    let ticks: Vec<Tick<i32, String>> = x_values
        .iter()
        .step_by(x_values.len() / 5)
        .map(|&x| Tick::Major(x as i32, AutoOption::Fix(tick_labels[x].clone())))
        .collect::<Vec<_>>();

    let threshold_x = [0, x_values.len() - 1];
    let threshold_y = [config.threshold, config.threshold];

    fg.axes2d()
        .lines(
            &x_values,
            &y_values,
            &[Caption("Exchange Rate"), Color("blue")],
        )
        .lines(
            &threshold_x,
            &threshold_y,
            &[Caption("Limit"), Color("green"), LineStyle(Dash)],
        )
        .set_x_ticks_custom(ticks, &[], &[LabelOption::Rotate(-45.0)])
        .set_x_label("Time", &[])
        .set_y_label("Rate", &[]);

    println!("Generated plot {file_path}");
    fg.save_to_png(file_path, 800, 600).unwrap();

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::tempdir;

    // Helper function to create test data
    fn create_test_data() -> Vec<ExchangeRate> {
        vec![
            ExchangeRate::new(100.0, "2024-12-27T00:00:02Z"),
            ExchangeRate::new(150.0, "2024-12-28T00:00:02Z"),
            ExchangeRate::new(200.0, "2024-12-29T00:00:02Z"),
            ExchangeRate::new(250.0, "2024-12-30T00:00:02Z"),
        ]
    }

    // Helper function to verify file exists and has content
    fn verify_file_validity(file_path: &str) -> bool {
        if let Ok(metadata) = fs::metadata(file_path) {
            metadata.len() > 0
        } else {
            false
        }
    }

    #[test]
    fn test_successful_plot_generation() -> Result<(), Box<dyn std::error::Error>> {
        let dir = tempdir()?;
        let file_path = dir.path().join("test_plot.png");
        let prices = create_test_data();

        let result = generate_plot(&prices, file_path.to_str().unwrap());
        assert!(result.is_ok());
        assert!(verify_file_validity(file_path.to_str().unwrap()));

        Ok(())
    }

    #[test]
    fn test_empty_data() {
        let dir = tempdir().unwrap();
        let file_path = dir.path().join("empty_plot.png");
        let empty_prices: Vec<ExchangeRate> = Vec::new();

        let result = generate_plot(&empty_prices, file_path.to_str().unwrap());
        assert!(result.is_err());

        if let Err(e) = result {
            assert!(e.to_string().contains("empty"));
        }
    }

    #[test]
    fn test_invalid_file_path() {
        let prices = create_test_data();
        let result = generate_plot(&prices, "/nonexistent/directory/plot.png");

        assert!(result.is_err());
        if let Err(e) = result {
            assert!(
                e.to_string().contains("permission denied")
                    || e.to_string().contains("no such file or directory")
            );
        }
    }

    #[test]
    fn test_large_dataset() -> Result<(), Box<dyn std::error::Error>> {
        let dir = tempdir()?;
        let file_path = dir.path().join("large_plot.png");

        // Create large dataset
        let large_prices: Vec<ExchangeRate> = (1..=1000)
            .map(|i| ExchangeRate::new(i as f64, format!("Day{}", i).as_str()))
            .collect();

        let result = generate_plot(&large_prices, file_path.to_str().unwrap());
        assert!(result.is_ok());
        assert!(verify_file_validity(file_path.to_str().unwrap()));

        Ok(())
    }

    #[test]
    fn test_negative_values() -> Result<(), Box<dyn std::error::Error>> {
        let dir = tempdir()?;
        let file_path = dir.path().join("negative_plot.png");

        let negative_prices = vec![
            ExchangeRate::new(-100.0, "2024-12-26T00:00:02Z"),
            ExchangeRate::new(150.0, "2024-12-27T00:00:02Z"),
            ExchangeRate::new(-75.0, "2024-12-28T00:00:02Z"),
            ExchangeRate::new(200.0, "2024-12-29T00:00:02Z"),
        ];

        let result = generate_plot(&negative_prices, file_path.to_str().unwrap());
        assert!(result.is_ok());
        assert!(verify_file_validity(file_path.to_str().unwrap()));

        Ok(())
    }

    #[test]
    fn test_duplicate_labels() -> Result<(), Box<dyn std::error::Error>> {
        let dir = tempdir()?;
        let file_path = dir.path().join("duplicate_plot.png");

        let duplicate_prices = vec![
            ExchangeRate::new(100.0, "2024-12-26T00:00:02Z"),
            ExchangeRate::new(150.0, "2024-12-26T00:00:02Z"),
            ExchangeRate::new(200.0, "2024-12-27T00:00:02Z"),
        ];

        let result = generate_plot(&duplicate_prices, file_path.to_str().unwrap());
        assert!(result.is_ok());
        assert!(verify_file_validity(file_path.to_str().unwrap()));

        Ok(())
    }
}
