use nalgebra::{DMatrix, DVector, SVD};

// complete honesty here, i did not write this code
// as of the time of me writing this, i don't know a lot about matrix algebra.
// so i wasn't able to write my own implementation by hand to solve AX = B for quadratic regression.
pub fn quadratic_regression(
    x_values: &Vec<f64>,
    y_values: &Vec<f64>,
) -> Result<(f64, f64, f64), Box<dyn std::error::Error>> {
    let n = x_values.len();

    let x_matrix = DMatrix::from_fn(n, 3, |i, j| x_values[i].powi(2 - j as i32));
    let y_vector = DVector::from_vec(y_values.to_vec());

    let svd = SVD::new(x_matrix, true, true);
    let result = svd.solve(&y_vector, nalgebra::convert(1.0e-10))?;

    Ok((result[0], result[1], result[2]))
}
// the formula can be found here:
// https://www.scribbr.com/statistics/pearson-correlation-coefficient/
// or anywhere i guess
pub fn pearson_correlation(x: &[f64], y: &[f64]) -> f64 {
    let n = x.len() as f64;

    let sum_x: f64 = x.iter().sum();
    let sum_y: f64 = y.iter().sum();

    let sum_xy: f64 = x.iter().zip(y.iter()).map(|(&xi, &yi)| xi * yi).sum();
    let sum_x_squared: f64 = x.iter().map(|&xi| xi * xi).sum();
    let sum_y_squared: f64 = y.iter().map(|&yi| yi * yi).sum();

    let numerator = n * sum_xy - sum_x * sum_y;
    let denominator =
        ((n * sum_x_squared - sum_x.powi(2)) * (n * sum_y_squared - sum_y.powi(2))).sqrt();

    if denominator == 0.0 {
        return 0.0;
    }

    numerator / denominator
}

#[cfg(test)]
mod tests {
    // simple util function to round off some extra precision for testing
    fn round_to_decimal_digits(value: f64, decimal_digits: u32) -> f64 {
        let multiplier = 10u32.pow(decimal_digits);
        (value * multiplier as f64).round() / multiplier as f64
    }

    // example data was sourced from here
    // https://goodcalculators.com/quadratic-regression-calculator/

    #[test]
    fn quadratic_regression() {
        let x_values = vec![
            -5f64, -4f64, -3f64, -2f64, -1f64, 0f64, 1f64, 2f64, 3f64, 4f64,
        ];
        let y_values = vec![
            12.55, 15.61, 10.20, 11.77, 10.24, 9.84, 8.07, 11.63, 12.82, 15.85,
        ];
        let res = super::quadratic_regression(&x_values, &y_values).unwrap();
        let rounded = (
            round_to_decimal_digits(res.0, 4),
            round_to_decimal_digits(res.1, 4),
            round_to_decimal_digits(res.2, 4),
        );
        assert_eq!(rounded, (0.2484, 0.2837, 9.8881));
    }
    // data and results were sourced from wolframalpha:
    // https://www.wolframalpha.com/widgets/view.jsp?id=53a3838163d5fe3d2dc7a3dfd448758
    // so i hope this is right :D
    #[test]
    fn pearson_correlation() {
        let xs = [1.0, 2.0, 3.0, 4.0];
        let ys = [5.0, 9.0, 7.0, 10.0];
        let res = super::pearson_correlation(&xs, &ys);
        let rounded = round_to_decimal_digits(res, 6);
        assert_eq!(rounded, 0.756889);
    }
}
