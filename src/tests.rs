use std::collections::HashMap;
use std::fs;
use std::io::{BufWriter, Cursor, Read};
use std::num::Wrapping;
use ark_bn254::{Bn254, FrParameters};
use ark_ff::{Fp256, ToBytes};
use num_bigint::BigInt;
use rand::{Rng, thread_rng};

use crate::bellman_ce::bn256::{Bn256, Fr};
use crate::circom_circuit::CircomCircuit;
use crate::{plonk, reader};
use crate::bellman_ce::{Circuit, ConstraintSystem, Engine, Field, SynthesisError};
use crate::reader::{load_witness_from_array, load_witness_from_bin_file};
use crate::recursive::fe_to_biguint;
use crate::witness::witness_calculator::WitnessCalculator;

const CIRCUIT_FILE: &'static str = concat!(env!("CARGO_MANIFEST_DIR"), "/test/circuits/simple/circuit.r1cs.json");
const WITNESS_FILE: &'static str = concat!(env!("CARGO_MANIFEST_DIR"), "/test/circuits/simple/witness.json");
const VK_FILE: &'static str = concat!(env!("CARGO_MANIFEST_DIR"), "/test/circuits/simple/vk.bin");
const PROOF_FILE: &'static str = concat!(env!("CARGO_MANIFEST_DIR"), "/test/circuits/simple/proof.bin");
const MONOMIAL_KEY_FILE: &'static str = concat!(env!("CARGO_MANIFEST_DIR"), "/keys/setup/setup_2^10.key");
const MONOMIAL_KEY_FILE2: &'static str = concat!(env!("CARGO_MANIFEST_DIR"), "/keys/setup/setup_2^20.key");
const DEFAULT_TRANSCRIPT: &'static str = "keccak";

const CIRCUIT_ANALYZE_RESULT: &'static str = r#"{"num_inputs":2,"num_aux":2,"num_variables":4,"num_constraints":2,"num_nontrivial_constraints":2,"num_gates":3,"num_hints":2,"constraint_stats":[{"name":"0","num_gates":1},{"name":"1","num_gates":2}]}"#;

#[test]
fn test_analyze() {
    let circuit = CircomCircuit {
        r1cs: reader::load_r1cs(CIRCUIT_FILE),
        witness: None,
        wire_mapping: None,
        aux_offset: plonk::AUX_OFFSET,
    };

    let result = crate::plonk::analyse(circuit).unwrap();

    assert_eq!(CIRCUIT_ANALYZE_RESULT, serde_json::to_string(&result).unwrap());
}

#[test]
fn test_export_verification_key() {
    let circuit = CircomCircuit {
        r1cs: reader::load_r1cs(CIRCUIT_FILE),
        witness: None,
        wire_mapping: None,
        aux_offset: plonk::AUX_OFFSET,
    };

    let setup = plonk::SetupForProver::prepare_setup_for_prover(circuit, reader::load_key_monomial_form(MONOMIAL_KEY_FILE), None)
        .expect("prepare err");
    let vk = setup.make_verification_key().unwrap();
    let mut buf = vec![];
    vk.write(&mut buf).unwrap();
    let check_vk = fs::read(VK_FILE).unwrap();
    assert_eq!(check_vk, buf);
}

#[test]
fn test_prove() {
    let circuit = CircomCircuit {
        r1cs: reader::load_r1cs(CIRCUIT_FILE),
        witness: Some(reader::load_witness_from_file::<Bn256>(WITNESS_FILE)),
        wire_mapping: None,
        aux_offset: plonk::AUX_OFFSET,
    };

    let setup = plonk::SetupForProver::prepare_setup_for_prover(
        circuit.clone(),
        reader::load_key_monomial_form(MONOMIAL_KEY_FILE),
        reader::maybe_load_key_lagrange_form(None),
    )
        .unwrap();

    assert!(setup.validate_witness(circuit.clone()).is_ok());

    let _ = setup.get_srs_lagrange_form_from_monomial_form();

    let proof = setup.prove(circuit, DEFAULT_TRANSCRIPT).unwrap();
    let mut buf = vec![];
    proof.write(&mut buf).unwrap();
    let check_proof = fs::read(PROOF_FILE).unwrap();
    assert_eq!(check_proof, buf);
}

#[test]
fn test_verify() {
    let vk = reader::load_verification_key::<Bn256>(VK_FILE);

    let proof = reader::load_proof::<Bn256>(PROOF_FILE);
    assert!(plonk::verify(&vk, &proof, DEFAULT_TRANSCRIPT).expect("fail to verify proof"));
}

#[test]
fn test_prove2() {
    let cir_file = "/Users/lvcong/rust/plonkit/test/circuits/complex/circuit.r1cs";
    let wit_file = "/Users/lvcong/rust/plonkit/test/circuits/complex/witness.wtns";
    let vk_file = "/Users/lvcong/rust/plonkit/test/circuits/complex/vk.bin";
    let circuit = CircomCircuit {
        r1cs: reader::load_r1cs(cir_file),
        witness: Some(reader::load_witness_from_file::<Bn256>(wit_file)),
        wire_mapping: None,
        aux_offset: plonk::AUX_OFFSET,
    };

    let setup = plonk::SetupForProver::prepare_setup_for_prover(
        circuit.clone(),
        reader::load_key_monomial_form(MONOMIAL_KEY_FILE2),
        reader::maybe_load_key_lagrange_form(None),
    )
        .unwrap();

    assert!(setup.validate_witness(circuit.clone()).is_ok());

    let _ = setup.get_srs_lagrange_form_from_monomial_form();

    let proof = setup.prove(circuit, DEFAULT_TRANSCRIPT).unwrap();

    let vk = reader::load_verification_key::<Bn256>(vk_file);
    let mut proof_bytes = vec![];
    proof.write(&mut proof_bytes).unwrap();

    let proof = reader::load_proof_from_bytes::<Bn256>(proof_bytes);

    println!("\n\n\n\n proof信息为: {:?} \n\n\n", proof);
    assert!(plonk::verify(&vk, &proof, DEFAULT_TRANSCRIPT).expect("fail to verify proof"));
}


#[test]
fn test_prove3() {
    let input_json_file = "";
    let wasm_file = "";
    let mut wtns = WitnessCalculator::new(wasm_file).unwrap();
    let mut inputs: HashMap<String, Vec<BigInt>> = HashMap::new();
}


#[test]
fn test_calculate_witness() {
    let path = "/Users/lvcong/rust/plonkit/test/circoms/mycircuit.wasm";
    let mut wtns = WitnessCalculator::new(path).unwrap();
    let mut inputs: HashMap<String, Vec<BigInt>> = HashMap::new();

    {
        let values = inputs.entry("a".to_string()).or_insert_with(Vec::new);
        values.push(1.into());
    }

    {
        let values = inputs.entry("b".to_string()).or_insert_with(Vec::new);
        values.push(2.into());
    }

    let cir_file = "/Users/lvcong/rust/plonkit/test/circuits/complex/circuit.r1cs";
    let wit_file = "/Users/lvcong/rust/plonkit/test/circuits/complex/witness.wtns";

    let data = wtns.calculate_witness_element_to_bytes::<Bn254, _>(inputs, false).unwrap();
    println!("{:?}", data);
    let circuit = CircomCircuit {
        r1cs: reader::load_r1cs(cir_file),
        witness: Some(reader::load_witness_from_array::<Bn256>(data).expect("fail")),
        wire_mapping: None,
        aux_offset: plonk::AUX_OFFSET,
    };

    let setup = plonk::SetupForProver::prepare_setup_for_prover(
        circuit.clone(),
        reader::load_key_monomial_form(MONOMIAL_KEY_FILE),
        reader::maybe_load_key_lagrange_form(None),
    )
        .unwrap();

    assert!(setup.validate_witness(circuit.clone()).is_ok());
}

#[derive(Clone)]
struct MySillyCircuit<E: Engine> {
    a: Option<E::Fr>,
    b: Option<E::Fr>,
}

impl<E: Engine> Circuit<E> for MySillyCircuit<E> {
    fn synthesize<CS: ConstraintSystem<E>>(
        self,
        cs: &mut CS,
    ) -> Result<(), SynthesisError>
    {
        let a = cs.alloc(|| "a", || self.a.ok_or(SynthesisError::AssignmentMissing))?;
        let b = cs.alloc(|| "b", || self.b.ok_or(SynthesisError::AssignmentMissing))?;
        let c = cs.alloc_input(|| "c", || {
            let mut a = self.a.ok_or(SynthesisError::AssignmentMissing)?;
            let b = self.b.ok_or(SynthesisError::AssignmentMissing)?;

            a.mul_assign(&b);
            Ok(a)
        })?;

        cs.enforce(
            || "a*b=c",
            |lc| lc + a,
            |lc| lc + b,
            |lc| lc + c,
        );

        Ok(())
    }
}

#[test]
pub fn test_simple() {
    let circuit = MySillyCircuit { a: None, b: None };
    let setup = plonk::SetupForProver::prepare_setup_for_prover(
        circuit.clone(),
        reader::load_key_monomial_form(MONOMIAL_KEY_FILE2),
        reader::maybe_load_key_lagrange_form(None),
    )
        .unwrap();

    let rng = &mut thread_rng();

    let a = Fr::one();
    let b = Fr::one();
    let mut c = a;
    c.mul_assign(&b);


    let _ = setup.get_srs_lagrange_form_from_monomial_form();

    let circuit = MySillyCircuit { a: Some(a), b: Some(b) };
    assert!(setup.validate_witness(circuit.clone()).is_ok());
    let proof = setup.prove(circuit, DEFAULT_TRANSCRIPT).unwrap();

    let vk = setup.make_verification_key().expect("fail");
    // let vk = reader::load_verification_key::<Bn256>(vk_file);
    let mut proof_bytes = vec![];
    proof.write(&mut proof_bytes).unwrap();

    let proof = reader::load_proof_from_bytes::<Bn256>(proof_bytes);

    println!("\n\n\n\n proof信息为: {:?} \n\n\n", proof);
    assert!(plonk::verify(&vk, &proof, DEFAULT_TRANSCRIPT).expect("fail to verify proof"));
}