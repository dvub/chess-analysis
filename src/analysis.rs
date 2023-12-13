use nalgebra::{DMatrix, DVector, SVD};

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

#[cfg(test)]
mod tests {
    fn round_to_decimal_digits(value: f64, decimal_digits: u32) -> f64 {
        let multiplier = 10u32.pow(decimal_digits);
        (value * multiplier as f64).round() / multiplier as f64
    }
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
}
