use boyer_moore_impl::majority_candidate;
use serde_json::{json, Value};
use std::io::Write;
use std::path::Path;
use std::process::{Command, Stdio};
use proptest::prelude::*;
use proptest::test_runner::FileFailurePersistence;


fn lean_candidate(votes: &[u32]) -> u32 {

  let root = Path::new(env!("CARGO_MANIFEST_DIR"))
    .parent()
    .unwrap();
  let oracle_path = root.join(format!(
    ".lake/build/bin/bm_oracle{}",
    std::env::consts::EXE_SUFFIX
  ));

  let mut child = Command::new(&oracle_path)
    .stdin(Stdio::piped())
    .stdout(Stdio::piped())
    .stderr(Stdio::piped())
    .spawn()
    .expect("Lean オラクルを起動できません。lake build bm_oracle を確認してください");

  let request = json!({ "id": 1, "votes": votes});
  {
    let mut stdin = child.stdin.take().expect("stdin を取得できません");
    writeln!(stdin, "{request}").expect("リクエストを送信できません")
  } // stdin を閉じて EOF を送り Lean 側の入力ループを終了させる

  let output = child.wait_with_output().expect("Lean の終了を待てません");
  assert!(
    output.status.success(),
    "Lean オラクルが異常終了しました：{}",
    String::from_utf8_lossy(&output.stderr)
  );

  let response: Value = serde_json::from_slice(&output.stdout)
    .expect("Lean の応答が JSON ではありません");
  assert_eq!(response["id"], json!(1));
  assert!(response.get("error").is_none(), "Lean のエラー: {response}");

  let candidate = response["candidate"]
    .as_u64()
    .expect("candidate が非負整数ではありません");

  u32::try_from(candidate).expect("candidate が u32 の範囲外です")
}

#[test]
fn fixed_inputs_match_lean() {
  let cases: Vec<(&str, Vec<u32>, u32)> = vec![
    ("empty", vec![], 0),
    ("single_zero", vec![0], 0),
    ("single", vec![7], 7),
    ("all_equal", vec![7, 7, 7, 7], 7),
    ("majority_at_start", vec![1, 1, 2], 1),
    ("majority_at_end", vec![1, 2, 2], 2),
    ("majority_after_cancellation", vec![1, 2, 3, 3, 3], 3),
    ("alternating_majority", vec![1, 2, 1, 2, 1], 1),
    ("tie", vec![1, 2], 1),
    ("alternating_tie", vec![1, 2, 1, 2], 1),
    ("all_distinct", vec![1, 2, 3, 4], 3),
    ("max_u32", vec![u32::MAX, 0, u32::MAX], u32::MAX),
    ("zero_majority", vec![0, u32::MAX, 0], 0),
    ("max_length_all_equal", vec![7; 4096], 7),
    ("max_length_tie", vec![1, 2].repeat(2048), 1),  
  ];

  for (name, votes, expected) in &cases {
    let lean = lean_candidate(votes);
    let rust = majority_candidate(votes);

    println!(
      "{name}: len={}, expected={expected}, Lean={lean}, Rust={rust}",
      votes.len()
    );

    assert_eq!(lean, *expected, "{name}: Lean の結果が期待値と異なります");
    assert_eq!(rust, lean, "{name}: Rust と Leanの結果が異なります");
  }
}

fn inputs_with_majority () -> impl Strategy<Value = (u32, Vec<u32>)> {
  (
    0_u32..4,
    proptest::collection::vec(0_u32..4, 0..=32),

  )
    .prop_flat_map(|(majority, extra_votes)| {
      let mut votes = vec![majority; extra_votes.len() + 1];
      votes.extend(extra_votes);

      (Just(majority), Just(votes).prop_shuffle())
    })
}




proptest! {
  #![proptest_config(ProptestConfig {
    cases: 100,
    failure_persistence: Some(Box::new(
      FileFailurePersistence::WithSource("proptest-regressions")
    )),
    ..ProptestConfig::default()
  })]

  #[test]
  fn random_inputs_match_lean(
    votes in proptest::collection::vec(0_u32..4, 0..=64)
  ) {
    let lean = lean_candidate(&votes);
    let rust = majority_candidate(&votes);

    prop_assert_eq!(rust, lean, "votes = {:?}", votes);
  }

  #[test]
  fn random_majority_matches_lean(
    (majority, votes) in inputs_with_majority()
  ) {
    let count = votes.iter().filter(|&&v| v == majority).count();
    prop_assert!(
      count > votes.len() / 2,
      "多数派が成立していません: majority={}, votes={:?}",
      majority, votes
    );

    let lean = lean_candidate(&votes);
    let rust = majority_candidate(&votes);

    prop_assert_eq!(lean, majority, "Lean: votes = {:?}", votes);
    prop_assert_eq!(rust, majority, "Rust: votes = {:?}", votes);
  }

}