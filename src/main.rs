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
}

// ================================================================================================

fn main() {
    // ACTIVE                                   T    T    T
    test_adjoint("+", add, ad_add, State::new([2.0, 3.0, 0.0],
                                              ["a", "b", "r"]));

    // ACTIVE                                        F    T    T
    test_adjoint("axpy", axpy, ad_axpy, State::new([1.0, 2.0, 5.0],
                                                   ["a", "x", "y"]));

    test_adjoint("linear_weight", linear_weight, ad_linear_weight,
    // ACTIVE                 T    F    T    F    T
                 State::new([0.0, 0.8, 2.3, 0.2, 3.2],
                            ["r", "A", "x", "B", "y"]));
}
