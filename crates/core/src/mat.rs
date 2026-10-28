//! Small vector and matrix helpers for the element and solver code: 3-vectors and 6×6 matrices.

pub type V = [f64; 3];
pub type M6 = [[f64; 6]; 6];

pub fn add(a: V, b: V) -> V {
    [a[0] + b[0], a[1] + b[1], a[2] + b[2]]
}
pub fn sub(a: V, b: V) -> V {
    [a[0] - b[0], a[1] - b[1], a[2] - b[2]]
}
pub fn scale(a: V, s: f64) -> V {
    [a[0] * s, a[1] * s, a[2] * s]
}
pub fn dot(a: V, b: V) -> f64 {
    a[0] * b[0] + a[1] * b[1] + a[2] * b[2]
}
pub fn cross(a: V, b: V) -> V {
    [a[1] * b[2] - a[2] * b[1], a[2] * b[0] - a[0] * b[2], a[0] * b[1] - a[1] * b[0]]
}
pub fn norm(a: V) -> f64 {
    dot(a, a).sqrt()
}
pub fn unit(a: V) -> V {
    let n = norm(a);
    if n > 0.0 { scale(a, 1.0 / n) } else { [0.0; 3] }
}
pub fn dist(a: V, b: V) -> f64 {
    norm(sub(a, b))
}

/// Local axes of a pipe direction `t`: x along the pipe, z = x × Y (horizontal), y = z × x
/// (upwards); a vertical pipe takes z = x × X instead.
pub fn axes(t: V) -> (V, V, V) {
    let x = unit(t);
    let z0 = cross(x, [0.0, 1.0, 0.0]);
    let z = if norm(z0) < 1e-6 { unit(cross(x, [1.0, 0.0, 0.0])) } else { unit(z0) };
    (x, cross(z, x), z)
}

pub fn zero6() -> M6 {
    [[0.0; 6]; 6]
}

/// a · b for 6×6 matrices.
pub fn mul6(a: &M6, b: &M6) -> M6 {
    let mut c = zero6();
    for i in 0..6 {
        for k in 0..6 {
            if a[i][k] != 0.0 {
                for j in 0..6 {
                    c[i][j] += a[i][k] * b[k][j];
                }
            }
        }
    }
    c
}
pub fn t6(a: &M6) -> M6 {
    let mut c = zero6();
    for i in 0..6 {
        for j in 0..6 {
            c[i][j] = a[j][i];
        }
    }
    c
}
pub fn mv6(a: &M6, v: &[f64; 6]) -> [f64; 6] {
    let mut o = [0.0; 6];
    for i in 0..6 {
        for j in 0..6 {
            o[i] += a[i][j] * v[j];
        }
    }
    o
}

/// The inverse of a symmetric positive definite 6×6 matrix (Gauss–Jordan with partial pivoting).
pub fn inv6(a: &M6) -> Option<M6> {
    let mut m = *a;
    let mut inv = zero6();
    for i in 0..6 {
        inv[i][i] = 1.0;
    }
    for c in 0..6 {
        let p = (c..6).max_by(|&x, &y| m[x][c].abs().total_cmp(&m[y][c].abs()))?;
        if !(m[p][c].abs() > 0.0) || !m[p][c].is_finite() {
            return None;
        }
        m.swap(c, p);
        inv.swap(c, p);
        let d = m[c][c];
        for j in 0..6 {
            m[c][j] /= d;
            inv[c][j] /= d;
        }
        for r in 0..6 {
            if r != c && m[r][c] != 0.0 {
                let f = m[r][c];
                for j in 0..6 {
                    m[r][j] -= f * m[c][j];
                    inv[r][j] -= f * inv[c][j];
                }
            }
        }
    }
    Some(inv)
}

/// [[I, 0], [skew(r), I]]: a force and moment at a point moved to a point `r` behind it
/// (F stays, M gains r × F).
pub fn shift(r: V) -> M6 {
    let mut h = zero6();
    for i in 0..6 {
        h[i][i] = 1.0;
    }
    // skew(r) F = r × F
    h[3][1] = -r[2];
    h[3][2] = r[1];
    h[4][0] = r[2];
    h[4][2] = -r[0];
    h[5][0] = -r[1];
    h[5][1] = r[0];
    h
}

/// Gauss–Legendre points and weights on [0, 1] (4 points: exact for cubics).
pub const GAUSS4: [(f64, f64); 4] = [
    (0.069_431_844_202_973_71, 0.173_927_422_568_726_93),
    (0.330_009_478_207_571_9, 0.326_072_577_431_273_07),
    (0.669_990_521_792_428_1, 0.326_072_577_431_273_07),
    (0.930_568_155_797_026_3, 0.173_927_422_568_726_93),
];

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn inverse_and_shift() {
        let mut a = zero6();
        for i in 0..6 {
            for j in 0..6 {
                a[i][j] = if i == j { 4.0 + i as f64 } else { 1.0 / (1.0 + (i + j) as f64) };
            }
        }
        let b = mul6(&a, &inv6(&a).unwrap());
        for i in 0..6 {
            for j in 0..6 {
                assert!((b[i][j] - if i == j { 1.0 } else { 0.0 }).abs() < 1e-12);
            }
        }
        // A force of 1 along Y at r = (2, 0, 0): moment about the origin is r × F = (0, 0, 2).
        let p = mv6(&shift([2.0, 0.0, 0.0]), &[0.0, 1.0, 0.0, 0.0, 0.0, 0.0]);
        assert_eq!(p, [0.0, 1.0, 0.0, 0.0, 0.0, 2.0]);
        let (x, y, z) = axes([1.0, 0.0, 0.0]);
        assert!(dot(x, y).abs() < 1e-15 && (y[1] - 1.0).abs() < 1e-15 && (z[2] - 1.0).abs() < 1e-15);
        let s: f64 = GAUSS4.iter().map(|g| g.1 * g.0.powi(3)).sum();
        assert!((s - 0.25).abs() < 1e-15);
    }
}
