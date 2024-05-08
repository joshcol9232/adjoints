#![allow(dead_code)]

mod state;

use crate::state::State;

const TOLERANCE: usize = 1;  // Multiples of epsillon

fn test_adjoint<const N: usize>(func_name: &'static str,
                                tl: impl Fn (&mut State<N>),
                                ad: impl Fn(&mut State<N>),
                                dx: State<N>) {
    // <M dx, M dx> = <M^T M dx, dx>
    let mut tl_state = dx.clone();

    // M dx
    tl(&mut tl_state);
    // <M dx, M dx>
    let mdx_dot: f32 = (tl_state.vec().transpose() * tl_state.vec_ref())[0];

    // M^T M dx
    ad(&mut tl_state);
    // <M^T M dx, dx>
    let mtranspose_m_dx_dot: f32 = (tl_state.vec().transpose() * dx.vec_ref())[0];

    let abs_diff = (mdx_dot - mtranspose_m_dx_dot).abs();
    let relative_to_ep = (abs_diff / f32::EPSILON) as usize;
    println!("State: {}", dx);
    println!("'{}' => {:?} {:?} : {:?}", func_name, mdx_dot, mtranspose_m_dx_dot, relative_to_ep);
        
    assert!(relative_to_ep <= TOLERANCE);
}

// ================================================================================================

// a(in), b(in), r(inout)
fn add(s: &mut State<3>) {
    s["r"] = s["a"] + s["b"];
}

fn ad_add(s: &mut State<3>) {
    s["a"] += s["r"];
    s["b"] += s["r"];
    s["r"] = 0.0;
}

// x(in), a(in), y(inout)
fn axpy(s: &mut State<3>) {
    s["y"] = s["a"] * s["x"] + s["y"];
}

fn ad_axpy(s: &mut State<3>) {
    // x = x + a * y;
    s["x"] = s["x"] + s["a"] * s["y"];
    // y = y;
}

// r(inout) = A(in) x(in) + B(in) y(in)
fn linear_weight(s: &mut State<5>) {
    s["r"] = s["A"] * s["x"] + s["B"] * s["y"];
}

fn ad_linear_weight(s: &mut State<5>) {
    s["x"] = s["x"] + s["A"]*s["r"];
    s["y"] = s["y"] + s["B"]*s["r"];
    s["r"] = 0.0;
}

// ---- STENCILS

fn apply_stencil_one<const N: usize>(f: &mut State<N>, func: impl Fn(f32, f32) -> f32) {
    let mut copy = f.clone();
    // Edge case
    copy[0] = func(f[N - 1], f[1]);
    for i in 1..=N-2 {
        copy[i] = func(f[i - 1], f[i + 1]);
    }
    copy[N - 1] = func(f[N - 2], f[0]);
    *f = copy;
}

fn ad_apply_stencil_one<const N: usize>(f: &mut State<N>, ad_func: impl Fn(f32) -> (f32, f32)) {
    let copy = f.clone();

    // NOTE: Once you have done a neighbour, you're then overwriting it?
    let (mut df_l, mut df_r) = ad_func(copy[N - 1]);
    //                |                |
    f[N - 2] += df_l;  f[N - 1] = 0.0;  f[0] += df_r;

    for i in (1..=N-2).rev() {
        (df_l, df_r) = ad_func(copy[i]);

        f[i - 1] += df_l;  f[i] = 0.0;  f[i + 1] += df_r;
    }
    (df_l, df_r) = ad_func(copy[0]);
    f[N - 1] += df_l;  f[0] = 0.0;  f[1] += df_r;
}

// Goes around the loop, smoothing. f[i] = f[i - 1] + f[i + 1] / 2.
fn smooth(left: f32, right: f32) -> f32 {
    left * 0.5 + right * 0.5
}

// Stencil size of 1.
fn smooth_stencil<const N: usize>(f: &mut State<N>) {
    apply_stencil_one(f, smooth)
}

// NOTE: Same as linear weighting adjoint.
fn ad_smooth(cent: f32) -> (f32, f32) {
    (0.5 * cent, 0.5 * cent)
}

fn ad_smooth_stencil_lbl<const N: usize>(f: &mut State<N>) {
    ad_apply_stencil_one(f, ad_smooth)
}

fn ad_smooth_stencil_primal<const N: usize>(f: &mut State<N>) {
    /*
    (f[N - 2], f[0], f[N - 1]) = ad_smooth(f[N - 2], f[0], f[N - 1]);
    for i in (1..=N-2).rev() {
        (f[i - 1], f[i + 1], f[i]) = ad_smooth(f[i - 1], f[i + 1], f[i]);
    }
    (f[N - 1], f[1], f[0]) = ad_smooth(f[N - 1], f[1], f[0]);
    */
    // In this case, the code forms a symmetric matrix and so the adjoint is the same as the TL.
    smooth_stencil(f)
}

fn weight_stencil<const N: usize>(f: &mut State<N>, a: f32, b: f32) {
    apply_stencil_one(f,
                      |left, right| -> f32 {
                          left * a + right * b
                      });
}

/*
fn ad_weight_stencil<const N: usize>(f: &mut State<N>, a: f32, b: f32) {
    ad_apply_stencil_one(f,
                         |mut left, mut right, centre| -> (f32, f32, f32) {
                            left = left + a * centre;
                            right = right + b * centre;
                            (left, right, 0.0)
                         });
}
*/

// -----------------

// ================================================================================================

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_add() {
        // ACTIVE                                   T    T    T
        test_adjoint("+", add, ad_add, State::new([2.0, 3.0, 0.0],
                                             Some(["a", "b", "r"])));
    }

    #[test]
    fn test_axpy() {
        // ACTIVE                                        F    T    T
        test_adjoint("axpy", axpy, ad_axpy, State::new([1.0, 2.0, 5.0],
                                                  Some(["a", "x", "y"])));
    }

    #[test]
    fn test_linear_weight() {
        test_adjoint("linear_weight", linear_weight, ad_linear_weight,
        // ACTIVE                 T    F    T    F    T
                     State::new([0.0, 0.8, 2.3, 0.2, 3.2],
                           Some(["r", "A", "x", "B", "y"])));
    }
     
    /*
    #[test]
    fn test_stencil() {
        let stencil = (1.0, 2.0, 3.0);
        let ad_stencil_result = ad_smooth(stencil.0, stencil.1, stencil.2);
        
        let mut stencil_state = State::new([stencil.2, 0.5, stencil.0, 0.5, stencil.1],
                                  Some(["r", "A", "x", "B", "y"]));
        ad_linear_weight(&mut stencil_state);

        println!("AD_SMOOTH: {:?}, AD_LINEAR {}", ad_stencil_result, stencil_state);

        test_adjoint("smooth_stencil", smooth_stencil, ad_smooth_stencil_primal,
                     State::new([0.2, 0.5, 0.6, 1.3, 2.3], None));
    }
    */

    #[test]
    fn test_lbl_stencil() {
        test_adjoint("lbl_stencil", smooth_stencil, ad_smooth_stencil_lbl,
                     State::new([0.2, 0.5, 0.6, 1.3, 2.3], None));

    }

    /*
    #[test]
    fn test_weighted_stencil() {
        let a = 0.8;
        let b = 0.2;

        test_adjoint("weighted_stencil",
                     |f| weight_stencil(f, a, b),
                     |f| weight_stencil(f, b ,a),
                     State::new([0.2, 0.5, 0.6, 1.3, 2.3], None));
    }
    */

}

fn main() {
}
