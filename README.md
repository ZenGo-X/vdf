# vdf
Simple RSA VDF in Rust ([Wesolowski18](https://eprint.iacr.org/2018/623.pdf))


```rust
        let t = BigInt::sample(20); //time parameter 
        // One public setup can work for many VDFs
        let setup = SetupForVDF::public_setup(&t); 


            // 1. challenger picks VDF challenge 
            let unsolved_vdf = SetupForVDF::pick_challenge(&setup); 
            // 2. challenger sends unsolved_vdf to solver
            // 3. solver solves VDF
            let solved_vdf = UnsolvedVDF::eval(&unsolved_vdf);
            // 4. solver sends solved vdf to challenger 
            // 5. challnger checks solution
            let res = solved_vdf.verify(&unsolved_vdf);
            assert!(res.is_ok())
