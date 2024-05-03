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

// Goes around the loop, smoothing. f[i] = f[i - 1] + f[i + 1] / 2.
fn smooth(left: f32, right: f32) -> f32 {
    left * 0.5 + right * 0.5
}

// Stencil size of 1.
fn smooth_stencil<const N: usize>(f: &mut State<N>) {
    // Edge case
    f[0] = smooth(f[N - 1], f[1]);
    for i in 1..=N-2 {
        f[i] = smooth(f[i - 1], f[i + 1]);
    }
    f[N - 1] = smooth(f[N - 2], f[0]);
}

// NOTE: Same as linear weighting adjoint.
fn ad_smooth(mut left: f32, mut right: f32, mut cent: f32) -> (f32, f32 ,f32) {
    left = left + 0.5 * cent;
    right = right + 0.5 * cent;
    cent = 0.0;
    (left, right, cent)
}

fn ad_smooth_stencil<const N: usize>(f: &mut State<N>) {
    (f[N - 2], f[0], f[N - 1]) = ad_smooth(f[N - 2], f[0], f[N - 1]);
    for i in (1..=N-2).rev() {
        (f[i - 1], f[i + 1], f[i]) = ad_smooth(f[i - 1], f[i + 1], f[i]);
    }
    (f[N - 1], f[1], f[0]) = ad_smooth(f[N - 1], f[1], f[0]);
}

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
     
    #[test]
    fn test_stencil() {
        let stencil = (1.0, 2.0, 3.0);
        let ad_stencil_result = ad_smooth(stencil.0, stencil.1, stencil.2);
        
        let mut stencil_state = State::new([stencil.2, 0.5, stencil.0, 0.5, stencil.1],
                                  Some(["r", "A", "x", "B", "y"]));
        ad_linear_weight(&mut stencil_state);

        println!("AD_SMOOTH: {:?}, AD_LINEAR {:?}", ad_stencil_result, stencil_state);

        test_adjoint("smooth_stencil", smooth_stencil, ad_smooth_stencil,
                     State::new([0.2, 0.5, 0.6, 1.3, 2.3], None));
    }

}

fn main() {
}
