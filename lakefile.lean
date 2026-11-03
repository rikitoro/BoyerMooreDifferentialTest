import Lake
open Lake DSL

package "BoyerMooreDifferentialTest" where
  version := v!"0.1.0"

require BoyerMooreVerification from git
  "https://github.com/rikitoro/BoyerMooreVerification.git" @
  "b2238d8f5b04b5856fcff318f94d17452672c10b"

@[default_target]
lean_exe bm_oracle where
  root := `Oracle.Main
