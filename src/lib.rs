use std::collections::{HashMap, HashSet};

use macros::define_model;
mod model;

pub fn add(left: u64, right: u64) -> u64 {
    left + right
}

#[define_model]
mod gaussian {
    pub struct Parameters {
        mu: Mu,
        sigma: Float,
    }

    pub struct Mu {
        mu: Float,
    }

    pub struct Data {
        x: Float,
    }

    pub fn n(sigma: Float) -> Float {
        (2.0 * 3.1415926535) * sigma
    }

    pub fn norm(sigma: Float) -> Float {
        1.0 / n(sigma)
    }

    pub fn distribution(parameters: Parameters, data: Data) -> Float {
        let norm = norm(parameters.sigma);
        let exp = (-0.5 * ((data.x - parameters.mu.mu) / parameters.sigma).powf(2.0)).exp();
        exp * norm
    }

    pub fn generation(parameters: Parameters) -> Data {
        todo!()
    }
}
//
// #[define_model]
// mod b0s_phi_mu_mu {
//     struct Parameters {
//         s: Signal,
//         b: Background,
//     }
//
//     struct Signal {
//         k1s: Float,
//         k1c: Float,
//         k2s: Float,
//         k2c: Float,
//         k3: Float,
//         k4: Float,
//         k5: Float,
//         k6s: Float,
//         k7: Float,
//         k8: Float,
//         k9: Float,
//         w1s: Float,
//         w1c: Float,
//         w2s: Float,
//         w2c: Float,
//         w3: Float,
//         w4: Float,
//         w5: Float,
//         w6s: Float,
//         w7: Float,
//         w8: Float,
//         w9: Float,
//         h1s: Float,
//         h1c: Float,
//         h2s: Float,
//         h2c: Float,
//         h3: Float,
//         h4: Float,
//         h5: Float,
//         h6s: Float,
//         h7: Float,
//         h8: Float,
//         h9: Float,
//         z1s: Float,
//         z1c: Float,
//         z2s: Float,
//         z2c: Float,
//         z3: Float,
//         z4: Float,
//         z5: Float,
//         z6s: Float,
//         z7: Float,
//         z8: Float,
//         z9: Float,
//         m: Float,
//         sigma_m: Float,
//     }
//
//     struct Background {
//         c0: Float,
//         c1: Float,
//         c2: Float,
//         k_m: Float,
//     }
//
//     struct Data {
//         ctl: Float,
//         ctk: Float,
//         phi: Float,
//         t: Float,
//         m: Float,
//     }
//
//     fn time_dependent(cosh_factor: Float, cos_factor: Float, h_i: Float, z_i: Float, x: Float, y: Float, gamma: Float, t: Float, sign: Float) -> Float {
//         let p = (y * gamma * t).exp();
//         let m = (-y * gamma * t).exp();
//         let cosh = (p + m) / 2.0;
//         let sinh = (p - m) / 2.0;
//         let cos = (x * gamma * t).cos();
//         let sin = (x * gamma * t).sin();
//         return cosh_factor * cosh - h_i * sinh + sign * (cos_factor * cos - z_i * sin);
//     }
//     pub fn distribution(p: Parameters, d: Data) -> Float {
//         let ctl2 = d.ctl * d.ctl;
//         let ctk2 = d.ctk * d.ctk;
//         let c2tl = 2.0 * ctl2 - 1.0;
//         let stk2 = 1.0 - ctk2;
//         let stl2 = 1.0 - ctl2;
//         let stl = stl2.powf(0.5);
//         let stk = stk2.powf(0.5);
//         let s2tl = 2.0 * stl * d.ctl;
//         let s2tk = 2.0 * stk * d.ctk;
//
//         return (9.0 / 64.0)
//             * (time_dependent(p.k1s, p.w1s, p.h1s, p.z1s, p.x, p.y, p.gamma, d.t, 1.0) * stk2
//                 + time_dependent(p.k1c, p.w1c, p.h1c, p.z1c, p.x, p.y, p.gamma, d.t, 1.0) * ctk2
//                 + time_dependent(p.k2s, p.w2s, p.h2s, p.z2s, p.x, p.y, p.gamma, d.t, 1.0) * stk2 * c2tl
//                 + time_dependent(p.k2c, p.w2c, p.h2c, p.z2c, p.x, p.y, p.gamma, d.t, 1.0) * ctk2 * c2tl
//                 + time_dependent(p.k3, p.w3, p.h3, p.z3, p.x, p.y, p.gamma, d.t, 1.0) * stk2 * stl2 * (2.0 * d.phi).cos()
//                 + time_dependent(p.k4, p.w4, p.h4, p.z4, p.x, p.y, p.gamma, d.t, 1.0) * s2tk * s2tl * d.phi.cos()
//                 + time_dependent(p.w5, p.k5, p.h5, p.z5, p.x, p.y, p.gamma, d.t, 1.0) * s2tl * stl * d.phi.cos()
//                 + time_dependent(p.w6s, p.k6s, p.h6s, p.z6s, p.x, p.y, p.gamma, d.t, 1.0) * stk2 * ctl2
//                 + time_dependent(p.k7, p.w7, p.h7, p.z7, p.x, p.y, p.gamma, d.t, 1.0) * s2tk * stl * d.phi.sin()
//                 + time_dependent(p.w8, p.k8, p.h8, p.z8, p.x, p.y, p.gamma, d.t, 1.0) * s2tk * s2tl * d.phi.sin()
//                 + time_dependent(p.w9, p.k9, p.h9, p.z9, p.x, p.y, p.gamma, d.t, 1.0) * stk2 * stl2 * (2.0 * d.phi).sin());
//     }
//
//     pub fn generation(parameters: Parameters) -> Data {
//         Data { ctl: 0.0, ctk: 0.0, phi: 0.0, t: 0.0, m: 0.0 }
//     }
// }
//
//
//
//
//
// #[derive(Debug, Eq, PartialEq, Hash)]
// struct N {
//     num: usize,
// }
//
// pub struct G {
//     nodes: HashSet<Box<N>>,
// }
//
// pub fn test_g(n: usize) -> usize {
//     let mut g = G { nodes: HashSet::new() };
//     let mut tests = Vec::new();
//     for i in 0..n {
//         let node = N { num: i };
//         let b = Box::new(node);
//         let pointer = b.as_ref() as *const N;
//
//         g.nodes.insert(b);
//         tests.push(pointer);
//     }
//     let mut failures = Vec::new();
//     for i in 0..n {
//         let p = tests[i];
//         if unsafe { &*p }.num != i {
//             failures.push(i);
//             println!("Failed on {}, found value {}", i, unsafe { &*p }.num);
//         }
//     }
//     failures.len()
// }
//
// #[cfg(test)]
// mod tests {
//     use super::*;
//
//     #[test]
//     fn it_works() {
//         let result = test_g(100_000_000);
//         println!("{}", result);
//     }
// }
