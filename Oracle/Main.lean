import Lean
import BMModel.Algorithm

/-! # Boyer-Moore differential-testing oracle
-/

namespace BoyerMoore.Oracle

/-- Keep the first spike's input domain shared with the Rust test runner. -/
def maxVotes : Nat := 4096

def maxVoteValue : Nat := 4294967295

structure Request where
  id : Nat
  votes : List Nat
  deriving Lean.FromJson

/-- Invalid requests still receive one response; the process can continue. -/
def errorResponse (id : Lean.Json) (message : String) : Lean.Json :=
  Lean.Json.mkObj [
    ("id", id),
    ("error", Lean.Json.str message)
  ]

/-- Parse one request, validate its domain, and execute the verified function. -/
def hadleLine (line : String) : Lean.Json :=
  match Lean.Json.parse line with
  | .error message =>
    errorResponse Lean.Json.null s!"invalid JSON: {message}"
  | .ok json =>
    let id := match json.getObjValAs? Nat "id" with
      | .ok n => Lean.toJson n
      | .error _ => Lean.Json.null
    match Lean.fromJson? (α := Request) json with
    | .error message =>
      errorResponse id s!"invalid request: {message}"
    | .ok request =>
      if request.votes.length > maxVotes then
        errorResponse id s!"votes must contain at most {maxVotes} elements"
      else if request.votes.any (fun v => decide (v > maxVoteValue)) then
        errorResponse id s!"each vote must be at most {maxVoteValue}"
      else
        let candidate := BoyerMoore.majorityCandidate request.votes
        Lean.Json.mkObj [
          ("id", Lean.toJson request.id),
          ("candidate", Lean.toJson candidate)
        ]

end BoyerMoore.Oracle

/-- Keep stdout exclusively for JSON Lines and flush each response. -/
def main : IO Unit := do
  let stdin ← IO.getStdin
  let stdout ← IO.getStdout
  while true do
    let line ← stdin.getLine
    if line.isEmpty then
      break
    let response := BoyerMoore.Oracle.hadleLine line
    stdout.putStrLn response.compress
    stdout.flush
