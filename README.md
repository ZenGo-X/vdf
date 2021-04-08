# vdf
Simple RSA VDF in Rust


```rust
        let t = BigInt::sample(20); //time parameter 
        // One public setup can work for many VDFs
        let setup = SetupForVDF::public_setup(&t); 


            // challenger picks VDF challenge 
            let unsolved_vdf = SetupForVDF::pick_challenge(&setup); 
            // solver solves VDF
            let vdf_out_proof = UnsolvedVDF::eval(&unsolved_vdf);
            // challnger check solution
            let res = vdf_out_proof.verify(&unsolved_vdf);
            assert!(res.is_ok())
