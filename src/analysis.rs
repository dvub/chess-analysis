use nalgebra::{DMatrix, DVector, SVD};

use crate::plots::two_var::residuals;

// complete honesty here, i did not write this code
// as of the time of me writing this, i don't know a lot about matrix algebra.
// so i wasn't able to write my own implementation by hand to solve AX = B for quadratic regression.

/// Returns three coefficients
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
// TODO: implement testing for this function

pub fn generate_residuals(
    x_values: &Vec<f64>,
    y_values: &Vec<f64>,
) -> Result<Vec<f64>, Box<dyn std::error::Error>> {
    let r = quadratic_regression(x_values, y_values)?;
    let predicted_y = x_values
        .iter()
        .map(|x| r.0 * (x * x) + r.1 * x + r.2)
        .collect::<Vec<_>>();

    Ok(y_values
        .iter()
        .zip(predicted_y)
        .collect::<Vec<(_, _)>>()
        .iter()
        .map(|(actual, predicted)| **actual - *predicted)
        .collect::<Vec<_>>())
}
// you're filled with determination!!
pub fn determination(
    x_values: &Vec<f64>,
    y_values: &Vec<f64>,
) -> Result<f64, Box<dyn std::error::Error>> {
    let sse: f64 = generate_residuals(x_values, y_values)?
        .iter()
        .map(|y| y.powi(2))
        .sum();
    let mean = y_values.iter().sum::<f64>() / y_values.len() as f64;
    let sst: f64 = y_values.iter().map(|y| (*y - mean).powi(2)).sum();

    Ok(1.0 - (sse / sst))
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
    #[test]
    fn determination() {
        let x_values = vec![312.0, 13.0, 14.0, 15.0, 15.0, 22.0, 27.0];
        let y_values = vec![11.0, 13.0, 14.0, 14.0, 15.0, 16.0, 18.0];
        let res = super::determination(&x_values, &y_values).unwrap();
        let rounded = round_to_decimal_digits(res, 4);
        assert_eq!(res, 0.8368);
    }
}
