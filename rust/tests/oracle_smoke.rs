use boyer_moore_impl::majority_candidate;
use serde_json::{json, Value};
use std::io::Write;
use std::path::Path;
use std::process::{Command, Stdio};

#[test]
fn fixed_input_matches_lean() {
  let votes = vec![1_u32, 1, 2];

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

  let lean_candidate = response["candidate"]
    .as_u64()
    .expect("candidate が非負整数ではありません");
  let rust_candidate = majority_candidate(&votes);

  println!("votes={votes:?}, Lean={lean_candidate}, rust={rust_candidate}");
  assert_eq!(lean_candidate, 1);
  assert_eq!(u64::from(rust_candidate), lean_candidate);

}
