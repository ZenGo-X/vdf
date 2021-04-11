#![allow(non_snake_case)]

use crate::utilities::ErrorReason;
use curv::arithmetic::traits::Modulo;
use curv::arithmetic::traits::Samplable;
use curv::arithmetic::{traits::*, BigInt};
use serde::{Deserialize, Serialize};
use utilities::{compute_rsa_modulus, h_g, hash_to_prime};

const BIT_LENGTH: usize = 4096;
const SEED_LENGTH: usize = 256;
pub mod utilities;

pub struct ElGamal;
pub struct ExponentElGamal;

/// Wesolowski VDF, based on https://eprint.iacr.org/2018/712.pdf.
/// Original paper: https://eprint.iacr.org/2018/623.pdf
///
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct SolvedVDF {
    vdf_instance: UnsolvedVDF,
    pub y: BigInt,
    pub pi: BigInt,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct SetupForVDF {
    pub t: BigInt,
    pub N: BigInt,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct UnsolvedVDF {
    pub x: BigInt,
    pub setup: SetupForVDF,
}

impl SetupForVDF {
    pub fn public_setup(t: &BigInt) -> Self {
        // todo: setup can also be used to define H_G. for example pick random domain separator
        let N = compute_rsa_modulus(BIT_LENGTH);
        SetupForVDF { t: t.clone(), N }
    }

    pub fn pick_challenge(setup: &SetupForVDF) -> UnsolvedVDF {
        let x = BigInt::sample(SEED_LENGTH);
        UnsolvedVDF {
            x,
            setup: setup.clone(),
        }
    }
}

impl UnsolvedVDF {
    //algorithm 3 from https://eprint.iacr.org/2018/623.pdf
    pub fn eval(unsolved_vdf: &UnsolvedVDF) -> SolvedVDF {
        let N = unsolved_vdf.setup.N.clone();
        let x = unsolved_vdf.x.clone();
        let t = unsolved_vdf.setup.t.clone();

        let g = h_g(&N, &x);
        let mut y = g.clone();
        let mut i = BigInt::zero();

        while i < t {
            y = BigInt::mod_mul(&y, &y, &N);
            i = i + BigInt::one();
        }
        let l = hash_to_prime(&unsolved_vdf.setup, &g, &y);

        //algorithm 4 from https://eprint.iacr.org/2018/623.pdf
        // long division TODO: consider alg 5 instead
        let mut i = BigInt::zero();
        let mut b: BigInt;
        let mut r = BigInt::one();
        let mut r2: BigInt;
        let two = BigInt::from(2);
        let mut pi = BigInt::one();
        let mut g_b: BigInt;

        while i < t {
            r2 = &r * &two;
            b = r2.div_floor(&l);
            r = r2.mod_floor(&l);
            g_b = BigInt::mod_pow(&g, &b, &N);
            pi = BigInt::mod_mul(&pi, &pi, &N);
            pi = BigInt::mod_mul(&pi, &g_b, &N);
            i = i + BigInt::one();
        }

        let vdf = SolvedVDF {
            vdf_instance: unsolved_vdf.clone(),
            y,
            pi,
        };
        vdf
    }
}

impl SolvedVDF {
    //algorithm 2 from https://eprint.iacr.org/2018/623.pdf
    pub fn verify(&self, unsolved_vdf: &UnsolvedVDF) -> Result<(), ErrorReason> {
        // we first check the solution received is for VDF generated by us
        if &self.vdf_instance != unsolved_vdf {
            return Err(ErrorReason::MisMatchedVDF);
        }
        let N = self.vdf_instance.setup.N.clone();
        let g = h_g(&self.vdf_instance.setup.N, &self.vdf_instance.x);

        // test that y is element in the group : https://eprint.iacr.org/2018/712.pdf 2.1 line 0
        if &self.y >= &N || &self.pi >= &N {
            return Err(ErrorReason::VDFVerifyError);
        }

        let l = hash_to_prime(&self.vdf_instance.setup, &g, &self.y);

        let r = BigInt::mod_pow(&BigInt::from(2), &self.vdf_instance.setup.t, &l);
        let pi_l = BigInt::mod_pow(&self.pi, &l, &N);
        let g_r = BigInt::mod_pow(&g, &r, &N);
        let pi_l_g_r = BigInt::mod_mul(&pi_l, &g_r, &N);

        match pi_l_g_r == self.y {
            true => return Ok(()),
            false => return Err(ErrorReason::VDFVerifyError),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::SetupForVDF;
    use super::UnsolvedVDF;
    use curv::arithmetic::traits::Samplable;
    use curv::BigInt;
    use std::time::Instant;

    #[test]
    fn test_vdf_valid_proof() {
        let t = BigInt::sample(13);
        let setup = SetupForVDF::public_setup(&t);

        let mut i = 0;
        while i < 10 {
            let unsolved_vdf = SetupForVDF::pick_challenge(&setup);
            let start = Instant::now();
            let solved_vdf = UnsolvedVDF::eval(&unsolved_vdf);
            let duration1 = start.elapsed();
            let start = Instant::now();
            // here unsolved_vdf is the version that was kept by the challenger
            let res = solved_vdf.verify(&unsolved_vdf);
            let duration2 = start.elapsed();
            i = i + 1;

            // todo: compute mean and std
            println!("eval time: {:?}", duration1);
            println!("verify time: {:?}", duration2);

            assert!(res.is_ok());
        }
    }
}
