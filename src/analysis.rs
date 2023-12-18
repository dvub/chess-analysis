use nalgebra::{DMatrix, DVector, SVD};
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
        .map(|x| (r.0 * (x * x)) + (r.1 * x) + r.2)
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
// simple util function to round off some extra precision for testing
pub fn to_precision(value: f64, decimal_digits: u32) -> f64 {
    let multiplier = 10u32.pow(decimal_digits);
    (value * multiplier as f64).round() / multiplier as f64
}

pub fn standard_deviation(data: &[f64]) -> f64 {
    let mean = data.iter().sum::<f64>() / data.len() as f64;

    let squared_diff_sum: f64 = data.iter().map(|&x| (x - mean).powi(2)).sum();
    let variance = squared_diff_sum / (data.len() - 1) as f64;

    variance.sqrt()
}

#[cfg(test)]
mod tests {
    // test results sourced from:
    // https://goodcalculators.com/quadratic-regression-calculator/
    // https://stapplet.com/quant2v.html
    use super::to_precision;

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
            to_precision(res.0, 4),
            to_precision(res.1, 4),
            to_precision(res.2, 4),
        );
        assert_eq!(rounded, (0.2484, 0.2837, 9.8881));
    }
    #[test]
    fn determination() {
        let x_values = vec![-5.0, -4.0, -3.0, -2.0, -1.0, 0.0, 1.0, 2.0, 3.0, 4.0];
        let y1 = [
            12.55, 15.61, 10.20, 11.77, 10.24, 9.84, 8.07, 11.63, 12.82, 15.85,
        ];
        let y2 = [9.0, 5.7, 6.5, 3.3, 1.9, 0.6, 1.2, 2.6, 5.3, 7.8];
        let res = super::determination(&x_values, &y1.to_vec()).unwrap();
        let res2 = super::determination(&x_values, &y2.to_vec()).unwrap();
        let rounded = to_precision(res.sqrt(), 4);
        let rounded2 = to_precision(res2, 3);

        assert_eq!(rounded, 0.7691);
        assert_eq!(rounded2, 0.889);
    }
    #[test]
    fn split_nested() {
        let data = [[0, 1, 2], [3, 4, 5], [6, 7, 8]];
        let (x_values, y_values): (Vec<i32>, Vec<i32>) = data
            .iter()
            .enumerate()
            .flat_map(|(x, row)| row.iter().map(move |&y| (x as i32, y)))
            .unzip();
        assert_eq!(x_values.len(), 9);
        assert_eq!(x_values, [0, 0, 0, 1, 1, 1, 2, 2, 2]);
        assert_eq!(y_values.len(), 9);
    }
    #[test]
    fn standard_devation() {
        let data = [10.0, 12.0, 23.0, 23.0, 16.0, 23.0, 21.0, 16.0];
        assert_eq!(super::standard_deviation(&data), 5.237229365663817);
    }
    #[test]
    fn residual_standard_deviation() {
        let x = vec![1.0, 2.0, 3.0, 4.0, 5.0, 6.0, 7.0, 8.0, 9.0, 10.0];
        let y = vec![10.0, 8.0, 6.0, 4.0, 2.0, 1.0, 3.0, 5.0, 7.0, 10.0];
        let residuals = super::generate_residuals(&x, &y).unwrap();
        let stdev = super::standard_deviation(&residuals);
        assert_eq!(stdev, 0.802);
    }
    #[test]
    fn residuals() {
        let x = vec![1.0, 2.0, 3.0, 4.0, 5.0, 6.0, 7.0, 8.0, 9.0, 10.0];
        let y = vec![10.0, 8.0, 6.0, 4.0, 2.0, 1.0, 3.0, 5.0, 7.0, 10.0];
        let mut residuals = super::generate_residuals(&x, &y)
            .unwrap()
            .iter()
            .map(|f| to_precision(*f, 3))
            .collect::<Vec<_>>();
        residuals.sort_by(|a, b| a.partial_cmp(b).unwrap());
        assert_eq!(
            residuals,
            vec![-1.339, -0.855, -0.436, -0.045, 0.018, 0.133, 0.445, 0.455, 0.664, 0.961]
        )
    }
}
