#![feature(adt_const_params)]
#![feature(generic_arg_infer)]

use std::collections::HashMap;
use nalgebra::SVector;

type State<const N: usize> = SVector<f32, N>;
const TOLERANCE: usize = 1;  // Multiples of epsillon

fn test_adjoint<const N: usize,
                const NAME: &'static str>(tl: impl Fn (&mut State<N>),
                                          ad: impl Fn(&mut State<N>),
                                          dx: State<N>) {
    // <M dx, M dx> = <M^T M dx, dx>
    let mut tl_state = dx.clone();

    // M dx
    tl(&mut tl_state);
    // <M dx, M dx>
    let mdx_dot: f32 = (tl_state.transpose() * tl_state)[0];

    // M^T M dx
    ad(&mut tl_state);
    // <M^T M dx, dx>
    let mtranspose_m_dx_dot: f32 = (tl_state.transpose() * dx)[0];

    let abs_diff = (mdx_dot - mtranspose_m_dx_dot).abs();
    let relative_to_ep = (abs_diff / f32::EPSILON) as usize;
    println!("'{}' => {:?}", NAME, relative_to_ep);
        
    assert!(relative_to_ep <= TOLERANCE);
}

// ================================================================================================

fn add(a_b_result: &mut State<3>) {
    // r = a + b;
    a_b_result[2] = a_b_result[0] + a_b_result[1];
}

fn ad_add(a_b_result: &mut State<3>) {
    // a += r;
    a_b_result[0] += a_b_result[2];
    // b += r;
    a_b_result[1] += a_b_result[2];
    // r = 0;
    a_b_result[2] = 0.0;
}

fn axpy(a_x_y: &mut State<3>) {
    // y = a * x + y;
    a_x_y[2] = a_x_y[0] * a_x_y[1] + a_x_y[2];
}

fn ad_axpy(a_x_y: &mut State<3>) {
    // x = x + a * y;
    a_x_y[1] = a_x_y[1] + a_x_y[0] + a_x_y[2];
    // y = y;
}

// ================================================================================================

fn main() {
    test_adjoint::<_, "+">(add, ad_add, [2.0, 3.0, 0.0].into());
    test_adjoint::<_, "axpy">(axpy, ad_axpy, [1.0, 2.0, 5.0].into());
}
