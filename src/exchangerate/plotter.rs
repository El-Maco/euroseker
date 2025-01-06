use std::collections::BTreeMap;

use chrono::NaiveDate;
use plotters::prelude::*;

use super::monitor::ExchangeRate;

pub fn generate_plot(prices: &[ExchangeRate], file_path: &str) -> Result<(), Box<dyn std::error::Error>> {
    let path = std::path::Path::new(file_path);
    let dir = path.parent().unwrap();
    std::fs::create_dir_all(dir).unwrap();

    let root = BitMapBackend::new(file_path, (800, 600)).into_drawing_area();
    root.fill(&WHITE)?;

    let mut date_map: BTreeMap<NaiveDate, Option<f64>> = BTreeMap::new();
    for rate in prices {
        date_map.insert(rate.date.date_naive(), Some(rate.rate));
    }

    let first_date = *date_map.keys().next().unwrap();
    let last_date = *date_map.keys().last().unwrap();

    for date in first_date.iter_days().take_while(|&d| d <= last_date) {
        if !date_map.contains_key(&date) {
            date_map.insert(date, None);
        }
    }

    let mut x_labels: Vec<String> = vec![];
    let mut data_points: Vec<(usize, f64)> = vec![];
    let mut index = 0;

    println!("date_map {:?}", date_map);
    for (date, rate) in &date_map {
        x_labels.push(date.format("%d-%m-%Y").to_string());
        if let Some(value) = rate {
            data_points.push((index, *value));
        }
        index += 1;
    }

    println!("x_labels {}: {:?}", x_labels.len(), x_labels);
    println!("data_points: {:?}", data_points);

    let max_price = prices.iter().map(|er| er.rate).fold(f64::MIN, f64::max) + 0.3;
    let min_price = prices.iter().map(|er| er.rate).fold(f64::MAX, f64::min) - 0.3;

    let mut chart = ChartBuilder::on(&root)
        .caption("Exchange Rates", ("sans-serif", 50))
        .margin(20)
        .x_label_area_size(40)
        .y_label_area_size(40)
        .build_cartesian_2d(
            0..prices.len(),
            min_price..max_price,
        )?;

    chart.configure_mesh()
        .x_labels(x_labels.len())
        .x_label_formatter(&|idx| x_labels.get(*idx).unwrap_or(&"".to_string()).clone())
        .y_desc("Exchange Rate")
        .x_desc("Date")
        .draw()?;

    chart.draw_series(LineSeries::new(
            data_points.into_iter(),
            &RED,
    ))?
        .label("Exchange Rate")
        .legend(|(x, y)| PathElement::new(vec![(x, y), (x + 20, y)], &RED));
    chart.configure_series_labels().background_style(&WHITE).draw()?;
    root.present()?;

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
            assert!(e.to_string().contains("permission denied") ||
                   e.to_string().contains("no such file or directory"));
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
